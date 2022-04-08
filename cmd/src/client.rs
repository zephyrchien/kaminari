use std::io::Result;
use tokio::net::{TcpListener, TcpStream};

const LOCAL: &str = "127.0.0.1:10000";
const REMOTE: &str = "127.0.0.1:20000";
const PATH: &str = "/ws";
const HOST: &str = "www.example.com";

use kaminari::AsyncConnect;
use kaminari::mix::{MixConnect, MixClientConf};
use kaminari::ws::WsConf;
use kaminari::tls::TlsClientConf;

#[tokio::main]
async fn main() {
    let lis = TcpListener::bind(LOCAL).await.unwrap();

    while let Ok((stream, _)) = lis.accept().await {
        tokio::spawn(relay(stream));
    }
}

async fn relay(mut local: TcpStream) -> Result<()> {
    let remote = TcpStream::connect(REMOTE).await?;
    let client = MixConnect::new(MixClientConf {
        ws: Some(WsConf {
            host: String::from(HOST),
            path: String::from(PATH),
        }),
        tls: Some(TlsClientConf {
            sni: String::from("some"),
            insecure: true,
        }),
    });
    let mut remote = client.connect(remote).await?;

    tokio::io::copy_bidirectional(&mut local, &mut remote).await?;

    Ok(())
}
