use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use kaminari::opt;
use kaminari::AsyncConnect;
use kaminari::nop::NopConnect;
use kaminari::ws::WsConnect;
use kaminari::tls::TlsConnect;
use kaminari::trick::Ref;

use kaminari_cmd::{Endpoint, parse_cmd, parse_env};

#[tokio::main]
async fn main() -> Result<()> {
    let (Endpoint { local, remote }, options) = parse_env().or_else(|_| parse_cmd())?;

    let ws = opt::get_ws_conf(&options);
    let tls = opt::get_tls_client_conf(&options);

    eprintln!("ws: {:?}", &ws);
    eprintln!("tls: {:?}", &tls);

    let lis = TcpListener::bind(local).await?;

    macro_rules! run {
        ($cc: expr) => {
            while let Ok((stream, _)) = lis.accept().await {
                tokio::spawn(relay(stream, remote, $cc));
            }
        };
    }

    match (ws, tls) {
        (None, None) => {
            let client = NopConnect {};
            run!(Ref::new(&client));
        }
        (Some(ws), None) => {
            let client = WsConnect::new(NopConnect {}, ws);
            run!(Ref::new(&client));
        }
        (None, Some(tls)) => {
            let client = TlsConnect::new(NopConnect {}, tls);
            run!(Ref::new(&client));
        }
        (Some(ws), Some(tls)) => {
            let client = WsConnect::new(TlsConnect::new(NopConnect {}, tls), ws);
            run!(Ref::new(&client));
        }
    };

    Ok(())
}

async fn relay<T>(mut local: TcpStream, remote: SocketAddr, client: Ref<T>) -> Result<()>
where
    T: AsyncConnect<TcpStream>,
{
    let remote = TcpStream::connect(remote).await?;

    let mut remote = client.connect(remote).await?;

    tokio::io::copy_bidirectional(&mut local, &mut remote).await?;

    Ok(())
}
