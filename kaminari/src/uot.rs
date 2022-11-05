pub use udpflow::{set_timeout, get_timeout};
pub use udpflow::{UdpSocket, UdpListener};
pub use udpflow::{UdpStreamLocal, UdpStreamRemote};
pub use udpflow::UotStream;

use std::io::Result;
use std::future::Future;
use std::fmt::{Display, Formatter};

use super::{IOStream, AsyncAccept, AsyncConnect};

#[derive(Debug, Clone)]
pub struct UotConnect<T> {
    conn: T,
}

#[derive(Debug, Clone)]
pub struct UotAccept<T> {
    lis: T,
}

impl<T> UotConnect<T> {
    #[inline]
    pub const fn new(conn: T) -> Self { Self { conn } }
}

impl<T> UotAccept<T> {
    #[inline]
    pub const fn new(lis: T) -> Self { Self { lis } }
}

impl<T> Display for UotConnect<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "[uot]{}", self.conn) }
}

impl<T> Display for UotAccept<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "[uot]{}", self.lis) }
}

impl<T, S> AsyncConnect<S> for UotConnect<T>
where
    S: IOStream,
    T: AsyncConnect<S>,
{
    type Stream = UotStream<T::Stream>;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn connect<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::ConnectFut<'a> {
        async move {
            let stream = self.conn.connect(stream, buf).await?;
            Ok(UotStream::new(stream))
        }
    }
}

impl<S, T> AsyncAccept<S> for UotAccept<T>
where
    S: IOStream,
    T: AsyncAccept<S>,
{
    type Stream = UotStream<T::Stream>;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn accept<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::AcceptFut<'a> {
        async move {
            let stream = self.lis.accept(stream, buf).await?;
            Ok(UotStream::new(stream))
        }
    }
}
