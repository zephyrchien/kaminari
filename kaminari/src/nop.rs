use std::io::Result;
use std::future::Future;

use super::{IOStream, AsyncAccept, AsyncConnect};

pub struct NopConnect {}

pub struct NopAccept {}

impl<'a, S> AsyncConnect<'a, S> for NopConnect
where
    S: IOStream,
{
    type Stream = S;

    type ConnectFut = impl Future<Output = Result<Self::Stream>>;

    fn connect(&'a self, stream: S) -> Self::ConnectFut { async move { Ok(stream) } }
}

impl<'a, S> AsyncAccept<'a, S> for NopAccept
where
    S: IOStream,
{
    type Stream = S;

    type AcceptFut = impl Future<Output = Result<Self::Stream>>;

    fn accept(&'a self, stream: S) -> Self::AcceptFut { async move { Ok(stream) } }
}
