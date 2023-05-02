use std::io::Result;
use std::future::Future;
use std::marker::PhantomData;
use std::fmt::{Display, Formatter};

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
pub enum MaskMode {
    Skip,
    Fixed,
    Standard,
}

impl From<Option<&str>> for MaskMode {
    fn from(item: Option<&str>) -> Self {
        match item {
            Some(item) => match item {
                "skip" => Self::Skip,
                "fixed" => Self::Fixed,
                "standard" => Self::Standard,
                _ => panic!("{item} mask mode is not supported."),
            },
            None => Self::Skip,
        }
    }
}

impl Display for MaskMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Skip => "skip",
                Self::Fixed => "fixed",
                Self::Standard => "standard",
            }
        )
    }
}

impl Default for MaskMode {
    fn default() -> Self {
        Self::Skip
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsConf {
    pub host: String,
    pub path: String,
    pub mask_mode: MaskMode,
}

impl Display for WsConf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "host: {}, path: {}, mask: {}",
            self.host, self.path, self.mask_mode
        )
    }
}

// =========== client ==========
#[derive(Debug, Clone, Copy)]
pub struct Simple {}

#[derive(Debug, Clone, Copy)]
pub struct Standard {}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct WsConnect<T, M = Simple> {
    conn: T,
    conf: WsConf,
    _marker: PhantomData<M>,
}

impl<T, M> Display for WsConnect<T, M>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ws]{}", self.conn)
    }
}

impl<T, M: Mode> WsConnect<T, M> {
    #[inline]
    pub const fn new(conn: T, conf: WsConf) -> Self {
        Self {
            conn,
            conf,
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

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn connect<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::ConnectFut<'a> {
        async move {
            let stream = self.conn.connect(stream, buf).await?;
            let stream = Endpoint::<_, M::ClientType>::connect_async(
                stream,
                buf,
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

impl<T> Display for WsAccept<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ws]{}", self.lis)
    }
}

impl<T> WsAccept<T> {
    #[inline]
    pub const fn new(lis: T, conf: WsConf) -> Self {
        Self { lis, conf }
    }
}

impl<S, T> AsyncAccept<S> for WsAccept<T>
where
    S: IOStream,
    T: AsyncAccept<S>,
{
    type Stream = WsServerStream<T::Stream>;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn accept<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::AcceptFut<'a> {
        async move {
            let stream = self.lis.accept(stream, buf).await?;

            let stream =
                Endpoint::<_, Server>::accept_async(stream, buf, &self.conf.host, &self.conf.path)
                    .await?
                    .guard();

            Ok(stream)
        }
    }
}
