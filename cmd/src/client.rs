use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use kaminari::opt;
use kaminari::AsyncConnect;
use kaminari::nop::NopConnect;
use kaminari::ws::WsConnect;
use kaminari::tls::TlsConnect;

use kaminari_cmd::{Endpoint, parse_cmd, parse_env};


#[tokio::main]
async fn main() -> Result<()> {
    let (Endpoint{local, remote}, options) = parse_env().or_else(|_| parse_cmd())?;

    let ws = opt::get_ws_conf(&options);
    let tls = opt::get_tls_client_conf(&options);
    
    eprintln!("ws: {:?}", &ws);
    eprintln!("tls: {:?}", &tls);

    let lis = TcpListener::bind(local).await?;

    while let Ok((stream, _)) = lis.accept().await {
        match (ws.clone(), tls.clone()) {
            (None, None) => tokio::spawn(relay(stream, remote, NopConnect{})),
            (Some(ws), None) => tokio::spawn(relay(stream, remote, WsConnect::new(
                NopConnect{}, ws
            ))),
            (None, Some(tls)) =>  tokio::spawn(relay(stream, remote, TlsConnect::new(
                NopConnect{}, tls
            ))),
            (Some(ws), Some(tls)) => tokio::spawn(relay(stream, remote, WsConnect::new(TlsConnect::new(
                NopConnect{}, tls
            ), ws)))
        };
    }

    Ok(())
}

async fn relay<T>(mut local: TcpStream, remote: SocketAddr, client: T) -> Result<()> 
where T:AsyncConnect<TcpStream>{
    let remote = TcpStream::connect(remote).await?;

    let mut remote = client.connect(remote).await?;

    tokio::io::copy_bidirectional(&mut local, &mut remote).await?;

    Ok(())
}
