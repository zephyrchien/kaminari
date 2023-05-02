#![allow(clippy::nonminimal_bool)]
#![macro_use]

#[cfg(feature = "ws")]
use super::ws::WsConf;

#[cfg(feature = "tls")]
use super::tls::{TlsClientConf, TlsServerConf};

#[macro_export]
macro_rules! has_opt {
    ($it: expr, $name: expr) => {
        $it.find(|&kv| kv == $name).is_some()
    };
    ($s: expr => $name: expr) => {
        $crate::has_opt!($s.split(';').map(|x| x.trim()), $name)
    };
}

#[macro_export]
macro_rules! get_opt {
    ($it: expr, $name: expr) => {
        $it.find(|kv| kv.starts_with($name))
            .and_then(|kv| kv.split_once("="))
            .map(|(_, v)| v.trim())
            .and_then(|v| if v.is_empty() { None } else { Some(v) })
    };
    ($s: expr => $name: expr) => {
        $crate::get_opt!($s.split(';').map(|x| x.trim()), $name)
    };
}

pub use has_opt;
pub use get_opt;

#[cfg(feature = "uot")]
pub fn get_uot_conf(s: &str) -> Option<()> {
    let it = s.split(';').map(|x| x.trim());

    if !has_opt!(it.clone(), "uot") {
        return None;
    }

    Some(())
}

#[cfg(feature = "ws")]
pub fn get_ws_conf(s: &str) -> Option<WsConf> {
    let it = s.split(';').map(|x| x.trim());

    if !has_opt!(it.clone(), "ws") {
        return None;
    }

    let mask_mode = get_opt!(it.clone(), "mask");

    let host = get_opt!(it.clone(), "host");
    let path = get_opt!(it.clone(), "path");

    if let (Some(host), Some(path)) = (host, path) {
        Some(WsConf {
            host: String::from(host),
            path: String::from(path),
            mask_mode: mask_mode.into(),
        })
    } else {
        panic!("ws: require host and path")
    }
}

#[cfg(feature = "tls")]
pub fn get_tls_client_conf(s: &str) -> Option<TlsClientConf> {
    let it = s.split(';').map(|x| x.trim());

    if !has_opt!(it.clone(), "tls") {
        return None;
    }

    let sni = get_opt!(it.clone(), "sni");
    let alpn = get_opt!(it.clone(), "alpn");
    let insecure = has_opt!(it.clone(), "insecure");
    let early_data = has_opt!(it.clone(), "0rtt");

    if let Some(sni) = sni {
        let alpn = alpn.map_or(Vec::new(), |s| {
            s.split(',')
                .map(str::trim)
                .map(Vec::from)
                .filter(|v| !v.is_empty())
                .collect()
        });
        Some(TlsClientConf {
            sni: String::from(sni),
            alpn,
            insecure,
            early_data,
        })
    } else {
        panic!("tls: require sni")
    }
}

#[cfg(feature = "tls")]
pub fn get_tls_server_conf(s: &str) -> Option<TlsServerConf> {
    let it = s.split(';').map(|x| x.trim());

    if !has_opt!(it.clone(), "tls") {
        return None;
    }

    let crt = get_opt!(it.clone(), "cert");
    let key = get_opt!(it.clone(), "key");
    let ocsp = get_opt!(it.clone(), "ocsp");
    let server_name = get_opt!(it.clone(), "servername");

    if crt.is_some() && key.is_some() || server_name.is_some() {
        Some(TlsServerConf {
            crt: crt.map_or(String::new(), String::from),
            key: key.map_or(String::new(), String::from),
            ocsp: ocsp.map_or(String::new(), String::from),
            server_name: server_name.map_or(String::new(), String::from),
        })
    } else {
        panic!("tls: require cert and key or servername")
    }
}

#[cfg(test)]
#[cfg(any(feature = "ws", feature = "tls"))]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "ws")]
    fn ws_conf() {
        use crate::ws::MaskMode;
        macro_rules! y {
            ( $( ($s:expr, $host: expr, $path: expr, $mask: expr); )+ )=> {
                $(
                    assert_eq!(get_ws_conf($s), Some(WsConf{
                        host: String::from($host),
                        path: String::from($path),
                        mask_mode: $mask,
                    }));
                )+
            }
        }

        y![
            ("ws;host=a.b.c;path=/", "a.b.c", "/", MaskMode::Skip);
            ("ws;host=a.b.c;path=/abc;mask=standard", "a.b.c", "/abc", MaskMode::Standard);
            ("ws;path=/abc;host=a.b.c;mask=fixed", "a.b.c", "/abc", MaskMode::Fixed);
            ("ws;path=/abc;host=a.b.c;", "a.b.c", "/abc", MaskMode::Skip);
        ];
    }

    #[test]
    #[should_panic]
    #[cfg(feature = "ws")]
    fn ws_conf_err() {
        macro_rules! n {
            ( $( $s: expr, )+ ) => {{
                $(
                    assert_eq!(get_ws_conf($s), None);
                )+
            }}
        }

        n![
            "ws",
            "ws;",
            "ws;host",
            "ws;host=",
            "ws;host=;",
            "ws;host=a.b.c;",
            "ws;host=a.b.c;path",
            "ws;host=a.b.c;path=",
            "ws;host=a.b.c;path=;",
        ];
    }

    #[test]
    #[cfg(feature = "tls")]
    fn tls_client_conf() {
        macro_rules! y {
            ( $( ($s:expr, $sni: expr, $alpn: expr, $insecure: expr, $early_data: expr); )+ )=> {
                $(
                    assert_eq!(get_tls_client_conf($s), Some(TlsClientConf{
                        sni: String::from($sni),
                        alpn: $alpn.split(',').map(str::trim).map(Vec::from)
                        .filter(|v|!v.is_empty()).collect(),
                        insecure: $insecure,
                        early_data: $early_data,
                    }));
                )+
            }
        }

        y![
            ("tls;sni=a.b.c", "a.b.c", "", false, false);
            ("tls;sni=a.b.c;alpn=h2", "a.b.c", "h2", false, false);
            ("tls;sni=a.b.c;alpn=http/1.1;insecure", "a.b.c", "http/1.1", true, false);
            ("tls;sni=a.b.c;alpn=h2,http/1.1;insecure;", "a.b.c", "h2,http/1.1", true, false);
            ("tls;sni=a.b.c;alpn=h2,http/1.1;insecure;0rtt", "a.b.c", "h2,http/1.1", true, true);
            ("tls;sni=a.b.c;alpn=h2,http/1.1;insecure;0rtt;" ,"a.b.c", "h2,http/1.1", true, true);
        ];
    }

    #[test]
    #[should_panic]
    #[cfg(feature = "tls")]
    fn tls_client_err() {
        macro_rules! n {
            ( $( $s: expr, )+ ) => {{
                $(
                    assert_eq!(get_tls_client_conf($s), None);
                )+
            }}
        }

        n!["", "tls", "tls;", "tls;sni", "tls;sni=", "tls;sni=;",];
    }

    #[test]
    #[cfg(feature = "tls")]
    fn tls_server_conf() {
        macro_rules! y {
            ( $( ($s:expr, $key: expr, $crt: expr, $server_name: expr); )+ )=> {
                $(
                    assert_eq!(get_tls_server_conf($s), Some(TlsServerConf{
                        key: String::from($key),
                        crt: String::from($crt),
                        ocsp: String::new(),
                        server_name: String::from($server_name),
                    }));
                )+
            }
        }

        y![
            ("tls;key=/a;cert=/b", "/a", "/b", "");
            ("tls;key=/a;cert=/b;", "/a", "/b", "");
            ("tls;key=/a;cert=/b;servername=;", "/a", "/b", "");

            ("tls;servername=a.b.c", "", "", "a.b.c");
            ("tls;servername=a.b.c;", "", "", "a.b.c");
            ("tls;key=;cert=;servername=a.b.c", "", "", "a.b.c");

            // this is expected
            ("tls;key=/a;cert=/b;servername=a.b.c;", "/a", "/b", "a.b.c");
        ];
    }

    #[test]
    #[should_panic]
    #[cfg(feature = "tls")]
    fn tls_server_err() {
        macro_rules! n {
            ( $( $s: expr, )+ ) => {{
                $(
                    assert_eq!(get_tls_server_conf($s), None);
                )+
            }}
        }

        n![
            "",
            "tls",
            "tls;",
            "tls;key",
            "tls;key=",
            "tls;key=;",
            "tls;key=/a;",
            "tls;key=/a;cert",
            "tls;key=/a;cert=",
            "tls;key=/a;cert=;",
        ];
    }
}
