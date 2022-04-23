use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use realm_io::{CopyBuffer, bidi_copy_buf};

use kaminari::opt;
use kaminari::AsyncConnect;
use kaminari::nop::NopConnect;
use kaminari::ws::WsConnect;
use kaminari::tls::TlsConnect;
use kaminari::trick::Ref;

use kaminari_cmd::{Endpoint, parse_cmd, parse_env};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let (Endpoint { local, remote }, options) = parse_env().or_else(|_| parse_cmd())?;

    let ws = opt::get_ws_conf(&options);
    let tls = opt::get_tls_client_conf(&options);

    eprintln!("listen: {}", &local);
    eprintln!("remote: {}", &remote);
    eprintln!("ws: {:?}", &ws);
    eprintln!("tls: {:?}", &tls);

    let lis = TcpListener::bind(local).await?;

    macro_rules! run {
        ($cc: expr) => {
            loop {
                match lis.accept().await {
                    Ok((stream, _)) => {
                        tokio::spawn(relay(stream, remote, $cc));
                    }
                    Err(e) => eprintln!("accept error: {}", e),
                }
            }
        };
    }

    macro_rules! run_ws_each {
        ($client: expr) => {
            let ws_mask_mode = opt::get_opt!(&options => "mask");
            match ws_mask_mode {
                Some("standard") => {
                    eprintln!("mask: standard");
                    let client = $client.standard();
                    run!(Ref::new(&client));
                },
                Some("fixed") => {
                    let client = $client.fixed();
                    eprintln!("mask: fixed");
                    run!(Ref::new(&client));
                },
                _ => {
                    eprintln!("mask: skip");
                    run!(Ref::new(&$client));
                }
            };
        }
    }

    match (ws, tls) {
        (None, None) => {
            let client = NopConnect {};
            run!(Ref::new(&client));
        }
        (Some(ws), None) => {
            let client = WsConnect::new(NopConnect {}, ws);
            run_ws_each!(client);
        }
        (None, Some(tls)) => {
            let client = TlsConnect::new(NopConnect {}, tls);
            run!(Ref::new(&client));
        }
        (Some(ws), Some(tls)) => {
            let client = WsConnect::new(TlsConnect::new(NopConnect {}, tls), ws);
            run_ws_each!(client);
        }
    };
}

async fn relay<T>(mut local: TcpStream, remote: SocketAddr, client: Ref<T>) -> std::io::Result<()>
where
    T: AsyncConnect<TcpStream>,
{
    let mut buf1 = vec![0u8; 0x2000];
    let buf2 = vec![0u8; 0x2000];

    let remote = TcpStream::connect(remote).await?;
    let mut remote = client.connect(remote, &mut buf1).await?;

    let buf1 = CopyBuffer::new(buf1.into_boxed_slice());
    let buf2 = CopyBuffer::new(buf2.into_boxed_slice());

    let (res, _, _) = bidi_copy_buf(&mut local, &mut remote, buf1, buf2).await;

    res
}
