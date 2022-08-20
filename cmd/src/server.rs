use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use realm_io::{CopyBuffer, bidi_copy_buf};

use kaminari::opt;
use kaminari::trick::Ref;
use kaminari::AsyncAccept;
use kaminari::nop::NopAccept;
use kaminari::ws::WsAccept;
#[cfg(any(feature="tls-rustls", feature="tls-openssl"))]
use kaminari::tls::TlsAccept;

use kaminari_cmd::{Endpoint, parse_cmd, parse_env};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (Endpoint { local, remote }, options) = parse_env()
        .map(|(Endpoint { local, remote }, opt)| {
            (
                Endpoint {
                    local: remote,
                    remote: local,
                },
                opt,
            )
        })
        .or_else(|_| parse_cmd())?;

    let ws = opt::get_ws_conf(&options);

    #[cfg(any(feature="tls-rustls", feature="tls-openssl"))]
    let tls = opt::get_tls_server_conf(&options);

    eprintln!("listen: {}", &local);
    eprintln!("remote: {}", &remote);

    if let Some(ws) = &ws {
        eprintln!("ws: {}", ws)
    }

    #[cfg(any(feature="tls-rustls", feature="tls-openssl"))]
    if let Some(tls) = &tls {
        eprintln!("tls: {}", &tls);
    }

    let lis = TcpListener::bind(local).await?;

    macro_rules! run {
        ($ac: expr) => {
            println!("accept: {}", $ac.as_ref());
            loop {
                match lis.accept().await {
                    Ok((stream, _)) => {
                        tokio::spawn(relay(stream, remote, $ac));
                    }
                    Err(e) => {
                        eprintln!("accept error: {}", e);
                        break;
                    }
                }
            }
        };
    }

    #[cfg(any(feature="tls-rustls", feature="tls-openssl"))]
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

    #[cfg(not(any(feature="tls-rustls", feature="tls-openssl")))]
    if let Some(ws) = ws {
        let server = WsAccept::new(NopAccept {}, ws);
        run!(Ref::new(&server));
    } else {
        let server = NopAccept {};
        run!(Ref::new(&server));
    }

    Ok(())
}

async fn relay<T: AsyncAccept<TcpStream>>(
    local: TcpStream,
    remote: SocketAddr,
    server: Ref<T>,
) -> std::io::Result<()> {
    let mut buf1 = vec![0u8; 0x2000];
    let buf2 = vec![0u8; 0x2000];

    let mut local = server.accept(local, &mut buf1).await?;
    let mut remote = TcpStream::connect(remote).await?;

    let buf1 = CopyBuffer::new(buf1.into_boxed_slice());
    let buf2 = CopyBuffer::new(buf2.into_boxed_slice());

    bidi_copy_buf(&mut local, &mut remote, buf1, buf2).await
}
