use std::io::Result;
use std::future::Future;

use super::{IOStream, AsyncAccept, AsyncConnect};

#[derive(Clone, Copy)]
pub struct NopConnect {}

#[derive(Clone, Copy)]
pub struct NopAccept {}

impl<S> AsyncConnect<S> for NopConnect
where
    S: IOStream,
{
    type Stream = S;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn connect(&self, stream: S) -> Self::ConnectFut<'_> { async move { Ok(stream) } }
}

impl<S> AsyncAccept<S> for NopAccept
where
    S: IOStream,
{
    type Stream = S;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> where Self:'a;

    fn accept(&self, stream: S) -> Self::AcceptFut<'_> { async move { Ok(stream) } }
}
