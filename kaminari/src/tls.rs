use std::io::Result;
use std::future::Future;
use std::sync::Arc;

use super::{IOStream, AsyncAccept, AsyncConnect};

use tokio_rustls::rustls;
use rustls::client::ClientConfig;
use rustls::server::ServerConfig;
use rustls::ServerName;

use tokio_rustls::{TlsAcceptor, TlsConnector};
pub use tokio_rustls::client::TlsStream as TlsClientStream;
pub use tokio_rustls::server::TlsStream as TlsServerStream;

// ========== client ==========
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsClientConf {
    pub sni: String,
    pub insecure: bool,
    pub early_data: bool,
}

#[derive(Clone)]
pub struct TlsConnect<T> {
    conn: T,
    sni: ServerName,
    cc: TlsConnector,
}

impl<T> TlsConnect<T> {
    pub fn new(conn: T, conf: TlsClientConf) -> Self {
        let TlsClientConf {
            sni,
            insecure,
            early_data,
        } = conf;
        let sni: ServerName = sni.as_str().try_into().expect("invalid DNS name");

        let mut conf = if !insecure {
            ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(utils::firefox_roots())
                .with_no_client_auth()
        } else {
            ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(utils::SkipVerify {}))
                .with_no_client_auth()
        };

        conf.enable_early_data = early_data;

        Self {
            conn,
            sni,
            cc: Arc::new(conf).into(),
        }
    }
}

impl<'a, S, T> AsyncConnect<'a, S> for TlsConnect<T>
where
    S: IOStream,
    T: AsyncConnect<'a, S>,
{
    type Stream = TlsClientStream<T::Stream>;

    type ConnectFut = impl Future<Output = Result<Self::Stream>>;

    fn connect(&'a self, stream: S) -> Self::ConnectFut {
        async move {
            let sni = self.sni.clone();
            let stream = self.conn.connect(stream).await?;
            self.cc.connect(sni, stream).await
        }
    }
}

// ========== server ==========
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsServerConf {
    pub crt: String,
    pub key: String,
    pub server_name: String,
}

#[derive(Clone)]
pub struct TlsAccept<T> {
    lis: T,
    ac: TlsAcceptor,
}

impl<T> TlsAccept<T> {
    pub fn new(lis: T, conf: TlsServerConf) -> Self {
        let cert;
        let key;

        if !conf.crt.is_empty() && !conf.key.is_empty() {
            cert = utils::read_certificates(&conf.crt).expect("failed to read certificate");
            key = utils::read_private_key(&conf.key).expect("failed to read private key");
        } else if !conf.server_name.is_empty() {
            (cert, key) = utils::generate_self_signed(&conf.server_name);
        } else {
            panic!("no certificate or private key supplied")
        }

        let conf = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert, key)
            .expect("bad certificate or key");

        Self {
            lis,
            ac: Arc::new(conf).into(),
        }
    }
}

impl<'a, S, T> AsyncAccept<'a, S> for TlsAccept<T>
where
    S: IOStream,
    T: AsyncAccept<'a, S>,
{
    type Stream = TlsServerStream<T::Stream>;

    type AcceptFut = impl Future<Output = Result<Self::Stream>>;

    fn accept(&'a self, stream: S) -> Self::AcceptFut {
        async move {
            let stream = self.lis.accept(stream).await?;
            self.ac.accept(stream).await
        }
    }
}

#[allow(unused)]
mod utils {
    use std::io::{BufReader, Result};
    use std::fs::{self, File};

    use tokio_rustls::rustls;
    use rustls::{Certificate, PrivateKey};
    use rustls::RootCertStore;
    use rustls::OwnedTrustAnchor;

    use rustls_pemfile::Item;
    use webpki_roots::TLS_SERVER_ROOTS;

    // copy & paste from https://github.com/EAimTY/tuic/blob/master/server/src/certificate.rs

    pub fn read_certificates(path: &str) -> Result<Vec<Certificate>> {
        let mut file = BufReader::new(File::open(path)?);
        let mut certs = Vec::new();

        // pem
        while let Ok(Some(item)) = rustls_pemfile::read_one(&mut file) {
            if let Item::X509Certificate(cert) = item {
                certs.push(Certificate(cert));
            }
        }

        // der
        if certs.is_empty() {
            certs = vec![Certificate(fs::read(path)?)];
        }

        Ok(certs)
    }

    pub fn read_private_key(path: &str) -> Result<PrivateKey> {
        let mut file = BufReader::new(File::open(path)?);
        let mut priv_key = None;

        // pem
        while let Ok(Some(item)) = rustls_pemfile::read_one(&mut file) {
            if let Item::RSAKey(key) | Item::PKCS8Key(key) | Item::ECKey(key) = item {
                priv_key = Some(key);
            }
        }

        // der
        priv_key
            .map(Ok)
            .unwrap_or_else(|| fs::read(path))
            .map(PrivateKey)
    }

    pub fn firefox_roots() -> RootCertStore {
        let mut roots = RootCertStore::empty();
        roots.add_server_trust_anchors(TLS_SERVER_ROOTS.0.iter().map(|x| {
            OwnedTrustAnchor::from_subject_spki_name_constraints(
                x.subject,
                x.spki,
                x.name_constraints,
            )
        }));
        roots
    }

    pub fn generate_self_signed(server_name: &str) -> (Vec<Certificate>, PrivateKey) {
        let self_signed = rcgen::generate_simple_self_signed(vec![server_name.to_string()])
            .expect("failed to generate self signed certificate and private key");

        let key = PrivateKey(self_signed.serialize_private_key_der());

        let cert = self_signed
            .serialize_der()
            .map(Certificate)
            .expect("failed to serialize self signed certificate");

        (vec![cert], key)
    }

    use rustls::client::{ServerCertVerified, ServerCertVerifier};
    pub struct SkipVerify {}
    impl ServerCertVerifier for SkipVerify {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::Certificate,
            _intermediates: &[rustls::Certificate],
            _server_name: &rustls::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> std::result::Result<rustls::client::ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }
    }
}
