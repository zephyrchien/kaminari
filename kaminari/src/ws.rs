use std::io::Result;
use std::future::Future;
use std::marker::PhantomData;

use super::{IOStream, AsyncAccept, AsyncConnect};

use lightws::endpoint::Endpoint;
use lightws::role::{Server, Client, StandardClient, FixedMaskClient, ClientRole};
use lightws::stream::{Guarded, Stream};

pub(crate) type WsStream<T, R> = Stream<T, R, Guarded>;
pub type WsServerStream<T> = WsStream<T, Server>;
pub type WsClientStream<T> = WsStream<T, Client>;
pub type WsStandardClientStream<T> = WsStream<T, StandardClient>;
pub type WsFixedClientStream<T> = WsStream<T, FixedMaskClient>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsConf {
    pub host: String,
    pub path: String,
}

// =========== client ==========
#[derive(Clone, Copy)]
pub struct Simple {}

#[derive(Clone, Copy)]
pub struct Standard {}

#[derive(Clone, Copy)]
pub struct Fixed {}

pub trait Mode {
    type ClientType: ClientRole;
}

impl Mode for Simple {
    type ClientType = Client;
}

impl Mode for Standard {
    type ClientType = StandardClient;
}

impl Mode for Fixed {
    type ClientType = FixedMaskClient;
}

#[derive(Clone)]
pub struct WsConnect<T, M = Simple> {
    conn: T,
    conf: WsConf,
    _marker: PhantomData<M>,
}

impl<T> WsConnect<T> {
    #[inline]
    pub const fn new(conn: T, conf: WsConf) -> Self {
        Self {
            conn,
            conf,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn standard(self) -> WsConnect<T, Standard> {
        WsConnect {
            conn: self.conn,
            conf: self.conf,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn fixed(self) -> WsConnect<T, Fixed> {
        WsConnect {
            conn: self.conn,
            conf: self.conf,
            _marker: PhantomData,
        }
    }
}

impl<S, T, M: Mode> AsyncConnect<S> for WsConnect<T, M>
where
    S: IOStream,
    T: AsyncConnect<S>,
    M::ClientType: Unpin + 'static,
{
    type Stream = Stream<T::Stream, M::ClientType, Guarded>;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn connect(&self, stream: S) -> Self::ConnectFut<'_> {
        async move {
            let mut buf = [0u8; 256];
            let stream = self.conn.connect(stream).await?;
            let stream = Endpoint::<_, M::ClientType>::connect_async(
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
    #[inline]
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
