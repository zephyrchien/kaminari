use std::io::Result;
use tokio::net::{TcpListener, TcpStream};

const LOCAL: &str = "127.0.0.1:20000";
const REMOTE: &str = "127.0.0.1:30000";
const PATH: &str = "/ws";
const HOST: &str = "www.example.com";

use kaminari::mix::{MixAccept, MixServerConf};
use kaminari::AsyncAccept;
use kaminari::ws::WsConf;
use kaminari::tls::TlsServerConf;

#[tokio::main]
async fn main() {
    let lis = TcpListener::bind(LOCAL).await.unwrap();

    while let Ok((stream, _)) = lis.accept().await {
        tokio::spawn(relay(stream));
    }
}

async fn relay(local: TcpStream) -> Result<()> {
    let server = MixAccept::new(MixServerConf {
        ws: Some(WsConf {
            host: String::from(HOST),
            path: String::from(PATH),
        }),
        tls: Some(TlsServerConf {
            crt: String::new(),
            key: String::new(),
            server_name: String::from("some"),
        }),
    });
    let mut local = server.accept(local).await?;
    let mut remote = TcpStream::connect(REMOTE).await?;

    tokio::io::copy_bidirectional(&mut local, &mut remote).await?;

    Ok(())
}
