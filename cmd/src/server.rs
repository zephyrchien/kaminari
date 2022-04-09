use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use kaminari::opt;
use kaminari::AsyncAccept;
use kaminari::nop::NopAccept;
use kaminari::ws::WsAccept;
use kaminari::tls::TlsAccept;

use cmd::{Endpoint, parse_cmd, parse_env};


#[tokio::main]
async fn main() -> Result<()>{
    let (Endpoint{local, remote}, options) = parse_env().or_else(|_| parse_cmd())?;

    let ws = opt::get_ws_conf(&options);
    let tls = opt::get_tls_server_conf(&options);

    eprintln!("ws: {:?}", &ws);
    eprintln!("tls: {:?}", &tls);

    let lis = TcpListener::bind(local).await?;

    while let Ok((stream, _)) = lis.accept().await {
        match (ws.clone(), tls.clone()) {
            (None, None) => tokio::spawn(relay(stream, remote, NopAccept{})),
            (Some(ws), None) => tokio::spawn(relay(stream, remote, WsAccept::new(
                NopAccept{}, ws
            ))),
            (None, Some(tls)) =>  tokio::spawn(relay(stream, remote, TlsAccept::new(
                NopAccept{}, tls
            ))),
            (Some(ws), Some(tls)) => tokio::spawn(relay(stream, remote, WsAccept::new(TlsAccept::new(
                NopAccept{}, tls
            ), ws)))
        };
    }

    Ok(())
}

async fn relay<T:AsyncAccept<TcpStream>>(local: TcpStream, remote: SocketAddr, server: T) -> Result<()> {
    let mut local = server.accept(local).await?;

    let mut remote = TcpStream::connect(remote).await?;

    tokio::io::copy_bidirectional(&mut local, &mut remote).await?;

    Ok(())
}
