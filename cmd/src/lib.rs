use std::env;
use std::net::{SocketAddr, ToSocketAddrs};
use anyhow::Result;

pub struct Endpoint {
    pub local: SocketAddr,
    pub remote: SocketAddr,
}

pub fn parse_env() -> Result<(Endpoint, String)> {
    let local_host = env::var("SS_LOCAL_HOST")?;
    let local_port = env::var("SS_LOCAL_PORT")?;
    let remote_host = env::var("SS_REMOTE_HOST")?;
    let remote_port = env::var("SS_REMOTE_PORT")?;
    let plugin_opts = env::var("SS_PLUGIN_OPTIONS")?;

    let local = format!("{}:{}", local_host, local_port)
        .to_socket_addrs()?
        .next()
        .unwrap();

    let remote = format!("{}:{}", remote_host, remote_port)
        .to_socket_addrs()?
        .next()
        .unwrap();

    Ok((Endpoint { local, remote }, plugin_opts))
}

pub fn parse_cmd() -> Result<(Endpoint, String)> {
    let args: Vec<String> = env::args().collect();

    anyhow::ensure!(args.len() == 4, "usage: <local> <remote> <options>");

    let local = args[1].to_socket_addrs()?.next().unwrap();
    let remote = args[2].to_socket_addrs()?.next().unwrap();
    let plugin_opts = args[3].clone();

    Ok((Endpoint { local, remote }, plugin_opts))
}
