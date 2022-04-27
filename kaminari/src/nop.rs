use std::io::Result;
use std::future::Future;
use std::fmt::{Display, Formatter};

use super::{IOStream, AsyncAccept, AsyncConnect};

#[derive(Debug, Clone, Copy)]
pub struct NopConnect {}

#[derive(Debug, Clone, Copy)]
pub struct NopAccept {}

impl Display for NopConnect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "[plain]") }
}

impl Display for NopAccept {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "[plain]") }
}

impl<S> AsyncConnect<S> for NopConnect
where
    S: IOStream,
{
    type Stream = S;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn connect(&self, stream: S, _: &mut [u8]) -> Self::ConnectFut<'_> { async move { Ok(stream) } }
}

impl<S> AsyncAccept<S> for NopAccept
where
    S: IOStream,
{
    type Stream = S;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn accept(&self, stream: S, _: &mut [u8]) -> Self::AcceptFut<'_> { async move { Ok(stream) } }
}
