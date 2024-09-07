#![allow(clippy::nonminimal_bool)]
#![macro_use]

#[cfg(feature = "ws")]
use super::ws::WsConf;

#[cfg(feature = "tls")]
use super::tls::TlsClientConf;

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

#[cfg(feature = "ws")]
pub fn get_ws_conf(s: &str) -> Option<WsConf> {
    let it = s.split(';').map(|x| x.trim());

    if !has_opt!(it.clone(), "ws") {
        return None;
    }

    let host = get_opt!(it.clone(), "host");
    let path = get_opt!(it.clone(), "path");

    if let (Some(host), Some(path)) = (host, path) {
        Some(WsConf {
            host: String::from(host),
            path: String::from(path),
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
