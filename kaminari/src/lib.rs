#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use std::io::Result;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait IOStream: AsyncRead + AsyncWrite + Unpin + 'static {}

impl<T> IOStream for T where T: AsyncRead + AsyncWrite + Unpin + 'static {}

pub trait AsyncConnect<'a, S: IOStream> {
    type Stream: IOStream;
    type ConnectFut: Future<Output = Result<Self::Stream>>;
    fn connect(&'a self, stream: S) -> Self::ConnectFut;
}

pub trait AsyncAccept<'a, S: IOStream> {
    type Stream: IOStream;
    type AcceptFut: Future<Output = Result<Self::Stream>>;
    fn accept(&'a self, stream: S) -> Self::AcceptFut;
}

pub mod ws;
pub mod nop;
pub mod tls;
pub mod mix;
pub mod opt;
