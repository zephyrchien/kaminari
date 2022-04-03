use std::io::Result;
use tokio::net::{TcpListener, TcpStream};

use lightws::endpoint::Endpoint;
use lightws::role::Server;

const LOCAL: &str = "127.0.0.1:20000";
const REMOTE: &str = "127.0.0.1:30000";
const PATH: &str = "/ws";
const HOST: &str = "www.example.com";

#[tokio::main]
async fn main() {
    let lis = TcpListener::bind(LOCAL).await.unwrap();

    while let Ok((stream, _)) = lis.accept().await {
        tokio::spawn(relay(stream));
    }
}

async fn relay(local: TcpStream) -> Result<()> {
    let mut buf = [0; 256];
    let mut local = Endpoint::<_, Server>::accept_async(local, &mut buf, HOST, PATH)
        .await?
        .guard();
    let mut remote = TcpStream::connect(REMOTE).await?;

    tokio::io::copy_bidirectional(&mut local, &mut remote).await?;

    Ok(())
}
