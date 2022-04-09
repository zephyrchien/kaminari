#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

use std::io::Result;
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait IOStream: AsyncRead + AsyncWrite + Unpin + 'static {}

impl<T> IOStream for T where T: AsyncRead + AsyncWrite + Unpin + 'static {}

pub trait AsyncConnect<S: IOStream> {
    type Stream: IOStream;
    type ConnectFut<'a>: Future<Output = Result<Self::Stream>>
    where
        Self: 'a;
    fn connect(&self, stream: S) -> Self::ConnectFut<'_>;
}

pub trait AsyncAccept<S: IOStream> {
    type Stream: IOStream;
    type AcceptFut<'a>: Future<Output = Result<Self::Stream>>
    where
        Self: 'a;
    fn accept(&self, stream: S) -> Self::AcceptFut<'_>;
}

pub mod ws;
pub mod nop;
pub mod tls;
pub mod mix;
pub mod opt;
