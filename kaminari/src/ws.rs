use std::io::Result;
use std::future::Future;

use super::{IOStream, AsyncAccept, AsyncConnect};

use lightws::endpoint::Endpoint;
use lightws::role::{Client, Server};
use lightws::stream::{Guarded, Stream};

pub type WsClientStream<T> = Stream<T, Client, Guarded>;
pub type WsServerStream<T> = Stream<T, Server, Guarded>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsConf {
    pub host: String,
    pub path: String,
}

// =========== client ==========
#[derive(Clone)]
pub struct WsConnect<T> {
    conn: T,
    conf: WsConf,
}

impl<T> WsConnect<T> {
    pub const fn new(conn: T, conf: WsConf) -> Self { Self { conn, conf } }
}

impl<S, T> AsyncConnect<S> for WsConnect<T>
where
    S: IOStream,
    T: AsyncConnect<S>,
{
    type Stream = WsClientStream<T::Stream>;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn connect(&self, stream: S) -> Self::ConnectFut<'_> {
        async move {
            let mut buf = [0u8; 256];
            let stream = self.conn.connect(stream).await?;
            let stream = Endpoint::<_, Client>::connect_async(
                stream,
                &mut buf,
                &self.conf.host,
                &self.conf.path,
            )
            .await?
            .guard();

            Ok(stream)
        }
    }
}

// ========== server ==========
#[derive(Debug, Clone)]
pub struct WsAccept<T> {
    lis: T,
    conf: WsConf,
}

impl<T> WsAccept<T> {
    pub const fn new(lis: T, conf: WsConf) -> Self { Self { lis, conf } }
}

impl<S, T> AsyncAccept<S> for WsAccept<T>
where
    S: IOStream,
    T: AsyncAccept<S>,
{
    type Stream = WsServerStream<T::Stream>;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn accept(&self, stream: S) -> Self::AcceptFut<'_> {
        async move {
            let mut buf = [0u8; 512];
            let stream = self.lis.accept(stream).await?;

            let stream = Endpoint::<_, Server>::accept_async(
                stream,
                &mut buf,
                &self.conf.host,
                &self.conf.path,
            )
            .await?
            .guard();

            Ok(stream)
        }
    }
}
