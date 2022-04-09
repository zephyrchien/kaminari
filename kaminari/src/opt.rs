use super::ws::WsConf;
use super::tls::{TlsClientConf, TlsServerConf};

macro_rules! has {
    ($it: expr, $name: expr) => {
        $it.find(|&kv| kv == $name).is_some()
    };
}

macro_rules! get {
    ($it: expr, $name: expr) => {
        $it.find(|kv| kv.starts_with($name))
            .and_then(|kv| kv.split_once("="))
            .map(|(_, v)| v.trim())
            .and_then(|v| if v.is_empty() { None } else { Some(v) })
    };
}

pub fn get_ws_conf(s: &str) -> Option<WsConf> {
    let it = s.split(';').map(|x| x.trim());

    if !has!(it.clone(), "ws") {
        return None;
    }

    let host = get!(it.clone(), "host");
    let path = get!(it.clone(), "path");

    if let (Some(host), Some(path)) = (host, path) {
        Some(WsConf {
            host: String::from(host),
            path: String::from(path),
        })
    } else {
        None
    }
}

pub fn get_tls_client_conf(s: &str) -> Option<TlsClientConf> {
    let it = s.split(';').map(|x| x.trim());

    if !has!(it.clone(), "tls") {
        return None;
    }

    let sni = get!(it.clone(), "sni");
    let insecure = has!(it.clone(), "insecure");
    let early_data = has!(it.clone(), "0rtt");

    sni.map(|sni| TlsClientConf {
        sni: String::from(sni),
        insecure,
        early_data,
    })
}

pub fn get_tls_server_conf(s: &str) -> Option<TlsServerConf> {
    let it = s.split(';').map(|x| x.trim());

    if !has!(it.clone(), "tls") {
        return None;
    }

    let crt = get!(it.clone(), "cert");
    let key = get!(it.clone(), "key");
    let server_name = get!(it.clone(), "servername");

    if crt.is_some() && key.is_some() || server_name.is_some() {
        Some(TlsServerConf {
            crt: crt.map_or(String::new(), String::from),
            key: key.map_or(String::new(), String::from),
            server_name: server_name.map_or(String::new(), String::from),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ws_conf() {
        macro_rules! n {
            ( $( $s: expr, )+ ) => {{
                $(
                    assert_eq!(get_ws_conf($s), None);
                )+
            }}
        }

        macro_rules! y {
            ( $( ($s:expr, $host: expr, $path: expr); )+ )=> {
                $(
                    assert_eq!(get_ws_conf($s), Some(WsConf{
                        host: String::from($host),
                        path: String::from($path),
                    }));
                )+
            }
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

        y![
            ("ws;host=a.b.c;path=/", "a.b.c", "/");
            ("ws;host=a.b.c;path=/abc", "a.b.c", "/abc");
            ("ws;path=/abc;host=a.b.c", "a.b.c", "/abc");
            ("ws;path=/abc;host=a.b.c;", "a.b.c", "/abc");
        ];
    }

    #[test]
    fn tls_client_conf() {
        macro_rules! n {
            ( $( $s: expr, )+ ) => {{
                $(
                    assert_eq!(get_tls_client_conf($s), None);
                )+
            }}
        }

        macro_rules! y {
            ( $( ($s:expr, $sni: expr, $insecure: expr, $early_data: expr); )+ )=> {
                $(
                    assert_eq!(get_tls_client_conf($s), Some(TlsClientConf{
                        sni: String::from($sni),
                        insecure: $insecure,
                        early_data: $early_data,
                    }));
                )+
            }
        }

        n!["", "tls", "tls;", "tls;sni", "tls;sni=", "tls;sni=;",];

        y![
            ("tls;sni=a.b.c", "a.b.c", false, false);
            ("tls;sni=a.b.c;", "a.b.c", false, false);
            ("tls;sni=a.b.c;insecure", "a.b.c", true, false);
            ("tls;sni=a.b.c;insecure;", "a.b.c", true, false);
            ("tls;sni=a.b.c;insecure;0rtt", "a.b.c", true, true);
            ("tls;sni=a.b.c;insecure;0rtt;", "a.b.c", true, true);
        ];
    }

    #[test]
    fn tls_server_conf() {
        macro_rules! n {
            ( $( $s: expr, )+ ) => {{
                $(
                    assert_eq!(get_tls_server_conf($s), None);
                )+
            }}
        }

        macro_rules! y {
            ( $( ($s:expr, $key: expr, $crt: expr, $server_name: expr); )+ )=> {
                $(
                    assert_eq!(get_tls_server_conf($s), Some(TlsServerConf{
                        key: String::from($key),
                        crt: String::from($crt),
                        server_name: String::from($server_name),
                    }));
                )+
            }
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
}
