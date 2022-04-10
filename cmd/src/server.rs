use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use kaminari::opt;
use kaminari::AsyncAccept;
use kaminari::nop::NopAccept;
use kaminari::ws::WsAccept;
use kaminari::tls::TlsAccept;
use kaminari::trick::Ref;

use kaminari_cmd::{Endpoint, parse_cmd, parse_env};

#[tokio::main]
async fn main() -> Result<()> {
    let (Endpoint { local, remote }, options) = parse_env().or_else(|_| parse_cmd())?;

    let ws = opt::get_ws_conf(&options);
    let tls = opt::get_tls_server_conf(&options);

    eprintln!("ws: {:?}", &ws);
    eprintln!("tls: {:?}", &tls);

    let lis = TcpListener::bind(local).await?;

    macro_rules! run {
        ($ac: expr) => {
            while let Ok((stream, _)) = lis.accept().await {
                tokio::spawn(relay(stream, remote, $ac));
            }
        };
    }

    match (ws, tls) {
        (None, None) => {
            let server = NopAccept {};
            run!(Ref::new(&server));
        }
        (Some(ws), None) => {
            let server = WsAccept::new(NopAccept {}, ws);
            run!(Ref::new(&server));
        }
        (None, Some(tls)) => {
            let server = TlsAccept::new(NopAccept {}, tls);
            run!(Ref::new(&server));
        }
        (Some(ws), Some(tls)) => {
            let server = WsAccept::new(TlsAccept::new(NopAccept {}, tls), ws);
            run!(Ref::new(&server));
        }
    };

    Ok(())
}

async fn relay<T: AsyncAccept<TcpStream>>(
    local: TcpStream,
    remote: SocketAddr,
    server: Ref<T>,
) -> Result<()> {
    let mut local = server.accept(local).await?;

    let mut remote = TcpStream::connect(remote).await?;

    tokio::io::copy_bidirectional(&mut local, &mut remote).await?;

    Ok(())
}
