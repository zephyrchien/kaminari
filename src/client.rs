use std::io::Result;
use tokio::net::{TcpListener, TcpStream};

use lightws::endpoint::Endpoint;
use lightws::role::Client;

const LOCAL: &str = "127.0.0.1:10000";
const REMOTE: &str = "127.0.0.1:20000";
const PATH: &str = "/ws";
const HOST: &str = "www.example.com";

#[tokio::main]
async fn main() {
    let lis = TcpListener::bind(LOCAL).await.unwrap();

    while let Ok((stream, _)) = lis.accept().await {
        tokio::spawn(relay(stream));
    }
}

async fn relay(mut local: TcpStream) -> Result<()> {
    let mut buf = [0; 256];
    let remote = TcpStream::connect(REMOTE).await?;
    let mut remote = Endpoint::<_, Client>::connect_async(remote, &mut buf, HOST, PATH)
        .await?
        .guard();

    tokio::io::copy_bidirectional(&mut local, &mut remote).await?;

    Ok(())
}
