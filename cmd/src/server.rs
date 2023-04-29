use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use realm_io::{CopyBuffer, bidi_copy_buf, buf_size};

use kaminari::opt;
use kaminari::AsyncAccept;
use kaminari::uot::UotAccept;
use kaminari::mix::{MixAccept, MixServerConf};
use tokio::net::UdpSocket;
use udpflow::UdpStreamRemote;

use kaminari_cmd::{Endpoint, parse_cmd, parse_env, UDP_MAX_BUF_LENGTH};

#[derive(Clone)]
enum Streamer {
    TcpStream(SocketAddr),
    UdpStream(SocketAddr),
}

#[tokio::main]
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
    let tls = opt::get_tls_server_conf(&options);

    eprintln!("listen: {}", &local);
    eprintln!("remote: {}", &remote);

    if let Some(ws) = &ws {
        eprintln!("ws: {ws}")
    }

    if let Some(tls) = &tls {
        eprintln!("tls: {}", &tls);
    }

    let uot = opt::get_uot_conf(&options);
    if uot.is_some() {
        eprintln!("UDP over TCP enabled.");
    }

    let acceptor = MixAccept::new_shared(MixServerConf { ws, tls });

    let uot = opt::get_uot_conf(&options);

    let remote = match uot {
        Some(_) => Streamer::UdpStream(remote),
        None => Streamer::TcpStream(remote),
    };

    let lis = TcpListener::bind(local).await?;

    #[cfg(all(unix, not(target_os = "android")))]
    let _ = realm_syscall::bump_nofile_limit();

    println!("accept: {}", &acceptor);
    loop {
        match lis.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(relay(stream, remote.clone(), acceptor.clone()));
            }
            Err(e) => {
                eprintln!("accept error: {e}");
                break;
            }
        }
    }

    Ok(())
}

async fn relay<T>(local: TcpStream, remote: Streamer, server: T) -> std::io::Result<()>
where
    T: AsyncAccept<TcpStream>,
{
    match remote {
        Streamer::TcpStream(remote) => {
            let mut buf1 = vec![0u8; buf_size()];
            let buf2 = vec![0u8; buf_size()];
            let mut local = server.accept(local, &mut buf1).await?;
            let mut remote = TcpStream::connect(remote).await?;

            let buf1 = CopyBuffer::new(buf1);
            let buf2 = CopyBuffer::new(buf2);

            bidi_copy_buf(&mut local, &mut remote, buf1, buf2).await
        }
        Streamer::UdpStream(remote) => {
            println!("{} -> {remote}", local.peer_addr()?);
            let mut buf1 = vec![0u8; UDP_MAX_BUF_LENGTH];
            let buf2 = vec![0u8; UDP_MAX_BUF_LENGTH];
            let server = UotAccept::new(server);
            let mut local = server.accept(local, &mut buf1).await?;

            let socket = UdpSocket::bind("127.0.0.1:0").await?;
            let mut remote = UdpStreamRemote::new(socket, remote);

            let buf1 = CopyBuffer::new(buf1.into_boxed_slice());
            let buf2 = CopyBuffer::new(buf2.into_boxed_slice());

            bidi_copy_buf(&mut local, &mut remote, buf1, buf2).await.map(|_| ())
        }
    }
}
