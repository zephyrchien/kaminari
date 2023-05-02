use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use realm_io::{CopyBuffer, bidi_copy_buf, buf_size};

use kaminari::opt;
use kaminari::AsyncConnect;
use kaminari::uot::UotConnect;
use kaminari::mix::{MixConnect, MixClientConf};
use tokio::net::UdpSocket;
use udpflow::{UdpListener, UdpStreamLocal};
use kaminari_cmd::{Endpoint, parse_cmd, parse_env, UDP_MAX_BUF_LENGTH};

enum Listener {
    TcpListener(TcpListener),
    UdpListener(UdpListener),
}

#[tokio::main]
async fn main() -> Result<()> {
    let (Endpoint { local, remote }, options) = parse_env().or_else(|_| parse_cmd())?;

    let ws = opt::get_ws_conf(&options);
    let tls = opt::get_tls_client_conf(&options);

    eprintln!("listen: {}", &local);
    eprintln!("remote: {}", &remote);

    if let Some(ref ws) = ws {
        eprintln!("ws: {ws}")
    }

    if let Some(ref tls) = tls {
        eprintln!("tls: {}", &tls);
    }

    let connector = MixConnect::new_shared(MixClientConf { ws, tls });

    let uot = opt::get_uot_conf(&options);
    if uot.is_some() {
        eprintln!("UDP over TCP enabled.");
    }

    let lis = match uot {
        Some(_) => {
            let socket = UdpSocket::bind(local).await.unwrap();
            Listener::UdpListener(UdpListener::new(socket))
        }
        None => Listener::TcpListener(TcpListener::bind(local).await.unwrap()),
    };

    #[cfg(all(unix, not(target_os = "android")))]
    let _ = realm_syscall::bump_nofile_limit();

    // let connector = Ref::new(&connector);
    println!("connect: {}", &connector);
    loop {
        match lis {
            Listener::TcpListener(ref lis) => match lis.accept().await {
                Ok((stream, _)) => {
                    tokio::spawn(relay_tcp(stream, remote, connector.clone()));
                }
                Err(e) => {
                    eprintln!("accept error: {e}");
                    break;
                }
            },
            Listener::UdpListener(ref lis) => {
                let mut buf = vec![0u8; UDP_MAX_BUF_LENGTH];
                match lis.accept(&mut buf).await {
                    Ok((stream, _)) => {
                        tokio::spawn(relay_uot(stream, remote, connector.clone(), buf));
                    }
                    Err(e) => {
                        eprintln!("accept error: {e}");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn relay_tcp<T>(mut local: TcpStream, remote: SocketAddr, client: T) -> std::io::Result<()>
where
    T: AsyncConnect<TcpStream>,
{
    let mut buf1 = vec![0u8; buf_size()];
    let buf2 = vec![0u8; buf_size()];

    let remote = TcpStream::connect(remote).await?;
    let mut remote = client.connect(remote, &mut buf1).await?;

    let buf1 = CopyBuffer::new(buf1.into_boxed_slice());
    let buf2 = CopyBuffer::new(buf2.into_boxed_slice());

    bidi_copy_buf(&mut local, &mut remote, buf1, buf2)
        .await
        .map(|_| ())
}

async fn relay_uot<T>(
    mut local: UdpStreamLocal,
    remote: SocketAddr,
    client: T,
    mut buf1: Vec<u8>,
) -> std::io::Result<()>
where
    T: AsyncConnect<TcpStream>,
{
    println!("{} -> {remote}", local.peer_addr());
    let buf2 = vec![0u8; UDP_MAX_BUF_LENGTH];

    let remote = TcpStream::connect(remote).await?;
    let client = UotConnect::new(client);
    let mut remote = client.connect(remote, &mut buf1).await?;

    let buf1 = CopyBuffer::new(buf1.into_boxed_slice());
    let buf2 = CopyBuffer::new(buf2.into_boxed_slice());

    bidi_copy_buf(&mut local, &mut remote, buf1, buf2)
        .await
        .map(|_| ())
}
