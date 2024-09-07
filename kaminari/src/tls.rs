use std::io::Result;
use std::future::Future;
use std::sync::Arc;
use std::fmt::{Debug, Display, Formatter};

use super::{IOStream, AsyncAccept, AsyncConnect};

use tokio_rustls::rustls;
use rustls::client::ClientConfig;
use rustls::pki_types::ServerName;

use tokio_rustls::{TlsAcceptor, TlsConnector};
pub use tokio_rustls::client::TlsStream as TlsClientStream;
pub use tokio_rustls::server::TlsStream as TlsServerStream;

// ========== client ==========
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsClientConf {
    pub sni: String,
    pub alpn: Vec<Vec<u8>>,
    pub insecure: bool,
    pub early_data: bool,
}

impl Display for TlsClientConf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut alpn = [0u8; 64];
        let mut cursor = 1;
        alpn[0] = b'[';
        for (i, b) in self.alpn.iter().enumerate() {
            alpn[cursor..cursor + b.len()].copy_from_slice(b);
            cursor += b.len();
            if i != self.alpn.len() - 1 {
                alpn[cursor..cursor + 2].copy_from_slice(b", ");
                cursor += 2;
            }
        }
        alpn[cursor] = b']';

        let alpn = std::str::from_utf8(&alpn[..cursor + 1]).unwrap();

        write!(
            f,
            "sni: {}, alpn: {}, insecure: {}, early_data: {}",
            self.sni, alpn, self.insecure, self.early_data
        )
    }
}

#[derive(Clone)]
pub struct TlsConnect<T> {
    conn: T,
    sni: ServerName<'static>,
    cc: TlsConnector,
}

impl<T> Display for TlsConnect<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "[tls]{}", self.conn) }
}

impl<T> Debug for TlsConnect<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsConnect")
            .field("conn", &self.conn)
            .field("sni", &self.sni)
            .finish()
    }
}

impl<T> TlsConnect<T> {
    pub fn new(conn: T, conf: TlsClientConf) -> Self {
        let TlsClientConf {
            sni,
            alpn,
            insecure,
            early_data,
        } = conf;
        let sni = ServerName::try_from(sni).expect("invalid DNS name");

        let mut conf = if !insecure {
            ClientConfig::builder()
                .with_root_certificates(utils::firefox_roots())
                .with_no_client_auth()
        } else {
            ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(utils::SkipVerify {}))
                .with_no_client_auth()
        };

        conf.enable_early_data = early_data;
        conf.alpn_protocols = alpn;

        Self {
            conn,
            sni,
            cc: Arc::new(conf).into(),
        }
    }

    // use shared roots
    pub fn new_shared(conn: T, conf: TlsClientConf) -> Self {
        let TlsClientConf {
            sni,
            alpn,
            insecure,
            early_data,
        } = conf;

        let sni = ServerName::try_from(sni).expect("invalid DNS name");

        let mut conf = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(utils::new_verifier(insecure))
            .with_no_client_auth();

        conf.enable_early_data = early_data;
        conf.alpn_protocols = alpn;

        Self {
            conn,
            sni,
            cc: Arc::new(conf).into(),
        }
    }
}

impl<S, T> AsyncConnect<S> for TlsConnect<T>
where
    S: IOStream,
    T: AsyncConnect<S>,
{
    type Stream = TlsClientStream<T::Stream>;

    type ConnectFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn connect<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::ConnectFut<'a> {
        async move {
            let sni = self.sni.clone();
            let stream = self.conn.connect(stream, buf).await?;
            self.cc.connect(sni, stream).await
        }
    }
}
