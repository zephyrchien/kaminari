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
    fn connect<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::ConnectFut<'a>;
}

pub trait AsyncAccept<S: IOStream> {
    type Stream: IOStream;
    type AcceptFut<'a>: Future<Output = Result<Self::Stream>>
    where
        Self: 'a;
    fn accept<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::AcceptFut<'a>;
}

pub mod nop;
pub mod opt;
pub mod trick;

#[cfg(feature = "ws")]
pub mod ws;

#[cfg(feature = "tls")]
pub mod tls;

#[cfg(feature = "uot")]
pub mod uot;

#[cfg(feature = "mix")]
pub mod mix;
