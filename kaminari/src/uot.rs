pub use udpflow::{set_timeout, get_timeout};
pub use udpflow::{UdpSocket, UdpListener};
pub use udpflow::{UdpStreamLocal, UdpStreamRemote};
pub use udpflow::UotStream;

use std::io::Result;
use std::future::Future;
use std::fmt::{Display, Formatter};

use super::{IOStream, AsyncAccept, AsyncConnect};

#[derive(Debug, Clone, Copy)]
pub struct UotConnect {}

#[derive(Debug, Clone, Copy)]
pub struct UotAccept {}

impl Display for UotConnect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "[uot]") }
}

impl Display for UotAccept {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "[uot]") }
}

impl<S> AsyncConnect<S> for UotConnect
where
    S: IOStream,
{
    type Stream = UotStream<S>;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn connect(&self, stream: S, _: &mut [u8]) -> Self::ConnectFut<'_> {
        async move { Ok(UotStream::new(stream)) }
    }
}

impl<S> AsyncAccept<S> for UotAccept
where
    S: IOStream,
{
    type Stream = UotStream<S>;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn accept(&self, stream: S, _: &mut [u8]) -> Self::AcceptFut<'_> {
        async move { Ok(UotStream::new(stream)) }
    }
}
