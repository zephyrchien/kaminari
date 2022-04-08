use super::ws::WsConf;
use super::tls::{TlsClientConf, TlsServerConf};

macro_rules! has {
    ($it: expr, $name: expr) => {
        $it.find(|kv| kv.starts_with($name)).is_some()
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
            "",
            "host",
            "host=",
            "host=;",
            "host=a.b.c;",
            "host=a.b.c;path",
            "host=a.b.c;path=",
            "host=a.b.c;path=;",
        ];

        y![
            ("host=a.b.c;path=/", "a.b.c", "/");
            ("host=a.b.c;path=/abc", "a.b.c", "/abc");
            ("path=/abc;host=a.b.c", "a.b.c", "/abc");
            ("path=/abc;host=a.b.c;", "a.b.c", "/abc");
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

        n!["", "sni", "sni=", "sni=;",];

        y![
            ("sni=a.b.c", "a.b.c", false, false);
            ("sni=a.b.c;", "a.b.c", false, false);
            ("sni=a.b.c;insecure", "a.b.c", true, false);
            ("sni=a.b.c;insecure;", "a.b.c", true, false);
            ("sni=a.b.c;insecure;0rtt", "a.b.c", true, true);
            ("sni=a.b.c;insecure;0rtt;", "a.b.c", true, true);
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
            "key",
            "key=",
            "key=;",
            "key=/a;",
            "key=/a;cert",
            "key=/a;cert=",
            "key=/a;cert=;",
        ];

        y![
            ("key=/a;cert=/b", "/a", "/b", "");
            ("key=/a;cert=/b;", "/a", "/b", "");
            ("key=/a;cert=/b;servername=;", "/a", "/b", "");

            ("servername=a.b.c", "", "", "a.b.c");
            ("servername=a.b.c;", "", "", "a.b.c");
            ("key=;cert=;servername=a.b.c", "", "", "a.b.c");

            // this is expected
            ("key=/a;cert=/b;servername=a.b.c;", "/a", "/b", "a.b.c");
        ];
    }
}
