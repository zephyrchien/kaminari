use std::io::Result;
use std::future::Future;
use std::fmt::{Display, Formatter};

use crate::ws;

use super::{IOStream, AsyncAccept, AsyncConnect};
use super::nop::{NopAccept, NopConnect};
use super::ws::{WsConf, WsAccept, WsConnect};
use super::tls::{TlsClientConf, TlsServerConf, TlsAccept, TlsConnect};

// ========== client ==========
#[derive(Debug, Clone)]
pub struct MixClientConf {
    pub ws: Option<WsConf>,
    pub tls: Option<TlsClientConf>,
}

#[derive(Debug, Clone)]
pub enum MixConnect {
    Plain(NopConnect),
    Ws(WsConnect<NopConnect>),
    WsFixed(WsConnect<NopConnect, ws::Fixed>),
    WsStandard(WsConnect<NopConnect, ws::Standard>),
    Tls(TlsConnect<NopConnect>),
    Wss(WsConnect<TlsConnect<NopConnect>>),
    WssFixed(WsConnect<TlsConnect<NopConnect>, ws::Fixed>),
    WssStandard(WsConnect<TlsConnect<NopConnect>, ws::Standard>),
}

impl MixConnect {
    pub fn new(conf: MixClientConf) -> Self {
        use MixConnect::*;
        let MixClientConf { ws, tls } = conf;
        match (ws, tls) {
            (None, None) => Plain(NopConnect {}),
            (Some(ws), None) => match ws.mask_mode {
                ws::MaskMode::Skip => Ws(WsConnect::new(NopConnect {}, ws)),
                ws::MaskMode::Fixed => WsFixed(WsConnect::new(NopConnect {}, ws)),
                ws::MaskMode::Standard => WsStandard(WsConnect::new(NopConnect {}, ws)),
            },
            (None, Some(tls)) => Tls(TlsConnect::new(NopConnect {}, tls)),
            (Some(ws), Some(tls)) => match ws.mask_mode {
                ws::MaskMode::Skip => Wss(WsConnect::new(TlsConnect::new(NopConnect {}, tls), ws)),
                ws::MaskMode::Fixed => {
                    WssFixed(WsConnect::new(TlsConnect::new(NopConnect {}, tls), ws))
                }
                ws::MaskMode::Standard => {
                    WssStandard(WsConnect::new(TlsConnect::new(NopConnect {}, tls), ws))
                }
            },
        }
    }

    pub fn new_shared(conf: MixClientConf) -> Self {
        use MixConnect::*;
        let MixClientConf { ws, tls } = conf;
        match (ws, tls) {
            (None, None) => Plain(NopConnect {}),
            (Some(ws), None) => match ws.mask_mode {
                ws::MaskMode::Skip => Ws(WsConnect::new(NopConnect {}, ws)),
                ws::MaskMode::Fixed => WsFixed(WsConnect::new(NopConnect {}, ws)),
                ws::MaskMode::Standard => WsStandard(WsConnect::new(NopConnect {}, ws)),
            },
            (None, Some(tls)) => Tls(TlsConnect::new_shared(NopConnect {}, tls)),
            (Some(ws), Some(tls)) => match ws.mask_mode {
                ws::MaskMode::Skip => Wss(WsConnect::new(
                    TlsConnect::new_shared(NopConnect {}, tls),
                    ws,
                )),
                ws::MaskMode::Fixed => WssFixed(WsConnect::new(
                    TlsConnect::new_shared(NopConnect {}, tls),
                    ws,
                )),
                ws::MaskMode::Standard => WssStandard(WsConnect::new(
                    TlsConnect::new_shared(NopConnect {}, tls),
                    ws,
                )),
            },
        }
    }
}

impl<S: IOStream> AsyncConnect<S> for MixConnect {
    type Stream = stream::MixClientStream<S>;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn connect<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::ConnectFut<'_> {
        use MixConnect::*;
        use stream::MixClientStream as MixS;

        async move {
            match self {
                Plain(cc) => cc.connect(stream, buf).await.map(MixS::Plain),
                Ws(cc) => cc.connect(stream, buf).await.map(MixS::Ws),
                WsFixed(cc) => cc.connect(stream, buf).await.map(MixS::WsFixed),
                WsStandard(cc) => cc.connect(stream, buf).await.map(MixS::WsStandard),
                Tls(cc) => cc.connect(stream, buf).await.map(MixS::Tls),
                Wss(cc) => cc.connect(stream, buf).await.map(MixS::Wss),
                WssFixed(cc) => cc.connect(stream, buf).await.map(MixS::WssFixed),
                WssStandard(cc) => cc.connect(stream, buf).await.map(MixS::WssStandard),
            }
        }
    }
}

// ========== server ==========
#[derive(Debug, Clone)]
pub struct MixServerConf {
    pub ws: Option<WsConf>,
    pub tls: Option<TlsServerConf>,
}

#[derive(Debug, Clone)]
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

    pub fn new_shared(conf: MixServerConf) -> Self {
        use MixAccept::*;
        let MixServerConf { ws, tls } = conf;
        match (ws, tls) {
            (None, None) => Plain(NopAccept {}),
            (Some(ws), None) => Ws(WsAccept::new(NopAccept {}, ws)),
            (None, Some(tls)) => Tls(TlsAccept::new_shared(NopAccept {}, tls)),
            (Some(ws), Some(tls)) => {
                Wss(WsAccept::new(TlsAccept::new_shared(NopAccept {}, tls), ws))
            }
        }
    }
}

impl<S: IOStream> AsyncAccept<S> for MixAccept {
    type Stream = stream::MixServerStream<S>;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn accept<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::AcceptFut<'a> {
        use MixAccept::*;
        use stream::MixServerStream as MixS;

        async move {
            match self {
                Plain(ac) => ac.accept(stream, buf).await.map(MixS::Plain),
                Ws(ac) => ac.accept(stream, buf).await.map(MixS::Ws),
                Tls(ac) => ac.accept(stream, buf).await.map(MixS::Tls),
                Wss(ac) => ac.accept(stream, buf).await.map(MixS::Wss),
            }
        }
    }
}

// ========== stream ==========
pub use stream::{MixClientStream, MixServerStream};

mod stream {
    use std::io::Result;
    use std::pin::Pin;
    use std::task::{Poll, Context};
    use tokio::io::{ReadBuf, AsyncRead, AsyncWrite};
    use crate::ws::{WsClientStream, WsServerStream, WsStandardClientStream, WsFixedClientStream};
    use crate::tls::{TlsClientStream, TlsServerStream};

    #[derive(Debug)]
    pub enum MixClientStream<T> {
        Plain(T),
        Ws(WsClientStream<T>),
        WsFixed(WsFixedClientStream<T>),
        WsStandard(WsStandardClientStream<T>),
        Tls(TlsClientStream<T>),
        Wss(WsClientStream<TlsClientStream<T>>),
        WssFixed(WsFixedClientStream<TlsClientStream<T>>),
        WssStandard(WsStandardClientStream<TlsClientStream<T>>),
    }

    #[derive(Debug)]
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

    macro_rules! impl_async_read_server {
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

    macro_rules! impl_async_write_server {
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

    macro_rules! impl_async_read_client {
        ($stream: ident) => {
            impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for $stream<T> {
                fn poll_read(
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    buf: &mut ReadBuf<'_>,
                ) -> Poll<Result<()>> {
                    use $stream::*;
                    call_each!(
                        self || Plain,
                        Ws,
                        WsFixed,
                        WsStandard,
                        Tls,
                        Wss,
                        WssFixed,
                        WssStandard,
                        || poll_read,
                        cx,
                        buf
                    )
                }
            }
        };
    }

    macro_rules! impl_async_write_client {
        ($stream: ident) => {
            impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for $stream<T> {
                fn poll_write(
                    self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    buf: &[u8],
                ) -> Poll<Result<usize>> {
                    use $stream::*;
                    call_each!(
                        self || Plain,
                        Ws,
                        WsFixed,
                        WsStandard,
                        Tls,
                        Wss,
                        WssFixed,
                        WssStandard,
                        || poll_write,
                        cx,
                        buf
                    )
                }

                fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
                    use $stream::*;
                    call_each!(
                        self || Plain,
                        Ws,
                        WsFixed,
                        WsStandard,
                        Tls,
                        Wss,
                        WssFixed,
                        WssStandard,
                        || poll_flush,
                        cx
                    )
                }

                fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
                    use $stream::*;
                    call_each!(
                        self || Plain,
                        Ws,
                        WsFixed,
                        WsStandard,
                        Tls,
                        Wss,
                        WssFixed,
                        WssStandard,
                        || poll_shutdown,
                        cx
                    )
                }
            }
        };
    }

    impl_async_read_client!(MixClientStream);
    impl_async_write_client!(MixClientStream);
    impl_async_read_server!(MixServerStream);
    impl_async_write_server!(MixServerStream);
}

// ========== type cast ==========

macro_rules! impl_type_cast {
    ($mix: ident || $([$func: ident :: $member: ident => $ret: ty], )+ ) => {
        impl $mix {
            $(
                pub fn $func(&self) -> Option<&$ret> {
                    use $mix::*;
                    if let $member(x) = self {
                        Some(x)
                    } else {
                        None
                    }
                }
            )+
        }
    };
}

impl_type_cast!(
    MixConnect ||
        [as_plain :: Plain => NopConnect],
        [as_ws :: Ws => WsConnect<NopConnect>],
        [as_tls :: Tls => TlsConnect<NopConnect>],
        [as_wss :: Wss => WsConnect<TlsConnect<NopConnect>>],
);

impl_type_cast!(
    MixAccept ||
        [as_plain :: Plain => NopAccept],
        [as_ws :: Ws => WsAccept<NopAccept>],
        [as_tls :: Tls => TlsAccept<NopAccept>],
        [as_wss :: Wss => WsAccept<TlsAccept<NopAccept>>],
);

// ========== display ==========

macro_rules! impl_display {
    ($mix: ident || $($member: ident,)+ ) => {
        impl Display for $mix {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                use $mix::*;
                match self {
                    $(
                        $member(x) => write!(f, "{}", x),
                    )+
                }
            }
        }
    };
}

impl_display!(
    MixConnect || Plain,
    Ws,
    WsFixed,
    WsStandard,
    Tls,
    Wss,
    WssFixed,
    WssStandard,
);
impl_display!(MixAccept || Plain, Ws, Tls, Wss,);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn print_conn() {
        let conf = MixClientConf {
            ws: Some(WsConf {
                host: String::from("abc"),
                path: String::from("chat"),
                mask_mode: ws::MaskMode::Skip,
            }),
            tls: Some(TlsClientConf {
                sni: String::from("abc"),
                alpn: vec![Vec::from("h2"), Vec::from("http/1.1")],
                insecure: true,
                early_data: true,
            }),
        };

        println!("ws: {}", conf.clone().ws.unwrap());
        println!("tls: {}", conf.clone().tls.unwrap());

        let conn = MixConnect::new(conf);

        println!("{conn}");
    }

    #[test]
    fn print_lis() {
        let conf = MixServerConf {
            ws: Some(WsConf {
                host: String::from("abc"),
                path: String::from("chat"),
                mask_mode: ws::MaskMode::Skip,
            }),
            tls: Some(TlsServerConf {
                crt: String::new(),
                key: String::new(),
                ocsp: String::new(),
                server_name: String::from("abc"),
            }),
        };

        println!("ws: {}", conf.clone().ws.unwrap());
        println!("tls: {}", conf.clone().tls.unwrap());

        let lis = MixAccept::new(conf);

        println!("{lis}");
    }
}
