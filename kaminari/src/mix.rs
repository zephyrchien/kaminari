use std::io::Result;
use std::future::Future;

use super::{IOStream, AsyncAccept, AsyncConnect};
use super::nop::{NopAccept, NopConnect};
use super::ws::{WsConf, WsAccept, WsConnect};
use super::tls::{TlsClientConf, TlsServerConf, TlsAccept, TlsConnect};

// ========== client ==========
pub struct MixClientConf {
    pub ws: Option<WsConf>,
    pub tls: Option<TlsClientConf>,
}

pub enum MixConnect {
    Plain(NopConnect),
    Ws(WsConnect<NopConnect>),
    Tls(TlsConnect<NopConnect>),
    Wss(WsConnect<TlsConnect<NopConnect>>),
}

impl MixConnect {
    pub fn new(conf: MixClientConf) -> Self {
        use MixConnect::*;
        let MixClientConf { ws, tls } = conf;
        match (ws, tls) {
            (None, None) => Plain(NopConnect {}),
            (Some(ws), None) => Ws(WsConnect::new(NopConnect {}, ws)),
            (None, Some(tls)) => Tls(TlsConnect::new(NopConnect {}, tls)),
            (Some(ws), Some(tls)) => Wss(WsConnect::new(TlsConnect::new(NopConnect {}, tls), ws)),
        }
    }
}

impl<'a, S: IOStream> AsyncConnect<'a, S> for MixConnect {
    type Stream = stream::MixClientStream<S>;

    type ConnectFut = impl Future<Output = Result<Self::Stream>>;

    fn connect(&'a self, stream: S) -> Self::ConnectFut {
        use MixConnect::*;
        use stream::MixClientStream as MixS;

        async move {
            match self {
                Plain(cc) => cc.connect(stream).await.map(MixS::Plain),
                Ws(cc) => cc.connect(stream).await.map(MixS::Ws),
                Tls(cc) => cc.connect(stream).await.map(MixS::Tls),
                Wss(cc) => cc.connect(stream).await.map(MixS::Wss),
            }
        }
    }
}

// ========== server ==========
pub struct MixServerConf {
    pub ws: Option<WsConf>,
    pub tls: Option<TlsServerConf>,
}

pub enum MixAccept {
    Plain(NopAccept),
    Ws(WsAccept<NopAccept>),
    Tls(TlsAccept<NopAccept>),
    Wss(WsAccept<TlsAccept<NopAccept>>),
}

impl MixAccept {
    pub fn new(conf: MixServerConf) -> Self {
        use MixAccept::*;
        let MixServerConf { ws, tls } = conf;
        match (ws, tls) {
            (None, None) => Plain(NopAccept {}),
            (Some(ws), None) => Ws(WsAccept::new(NopAccept {}, ws)),
            (None, Some(tls)) => Tls(TlsAccept::new(NopAccept {}, tls)),
            (Some(ws), Some(tls)) => Wss(WsAccept::new(TlsAccept::new(NopAccept {}, tls), ws)),
        }
    }
}

impl<'a, S: IOStream> AsyncAccept<'a, S> for MixAccept {
    type Stream = stream::MixServerStream<S>;

    type AcceptFut = impl Future<Output = Result<Self::Stream>>;

    fn accept(&'a self, stream: S) -> Self::AcceptFut {
        use MixAccept::*;
        use stream::MixServerStream as MixS;

        async move {
            match self {
                Plain(ac) => ac.accept(stream).await.map(MixS::Plain),
                Ws(ac) => ac.accept(stream).await.map(MixS::Ws),
                Tls(ac) => ac.accept(stream).await.map(MixS::Tls),
                Wss(ac) => ac.accept(stream).await.map(MixS::Wss),
            }
        }
    }
}

// ========== stream ==========
mod stream {
    use std::io::Result;
    use std::pin::Pin;
    use std::task::{Poll, Context};
    use tokio::io::{ReadBuf, AsyncRead, AsyncWrite};
    use crate::ws::{WsClientStream, WsServerStream};
    use crate::tls::{TlsClientStream, TlsServerStream};

    pub enum MixClientStream<T> {
        Plain(T),
        Ws(WsClientStream<T>),
        Tls(TlsClientStream<T>),
        Wss(WsClientStream<TlsClientStream<T>>),
    }

    pub enum MixServerStream<T> {
        Plain(T),
        Ws(WsServerStream<T>),
        Tls(TlsServerStream<T>),
        Wss(WsServerStream<TlsServerStream<T>>),
    }

    macro_rules! call_each {
        ($this: ident || $( $name: ident, )+ || $func: ident, $cx: ident, $buf: ident) => {
            match $this.get_mut() {
                $(
                    $name(x) => Pin::new(x).$func($cx, $buf),
                )+
            }
        };
        ($this: ident || $( $name: ident, )+ || $func: ident, $cx: ident) => {
            match $this.get_mut() {
                $(
                    $name(x) => Pin::new(x).$func($cx),
                )+
            }
        };
    }

    macro_rules! impl_async_read {
        ($stream: ident) => {
            impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for $stream<T> {
                fn poll_read(
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    buf: &mut ReadBuf<'_>,
                ) -> Poll<Result<()>> {
                    use $stream::*;
                    call_each!(self || Plain, Ws, Tls, Wss, || poll_read, cx, buf)
                }
            }
        };
    }

    macro_rules! impl_async_write {
        ($stream: ident) => {
            impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for $stream<T> {
                fn poll_write(
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    buf: &[u8],
                ) -> Poll<Result<usize>> {
                    use $stream::*;
                    call_each!(self || Plain, Ws, Tls, Wss, || poll_write, cx, buf)
                }

                fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
                    use $stream::*;
                    call_each!(self || Plain, Ws, Tls, Wss, || poll_flush, cx)
                }

                fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
                    use $stream::*;
                    call_each!(self || Plain, Ws, Tls, Wss, || poll_shutdown, cx)
                }
            }
        };
    }

    impl_async_read!(MixClientStream);
    impl_async_write!(MixClientStream);
    impl_async_read!(MixServerStream);
    impl_async_write!(MixServerStream);
}
