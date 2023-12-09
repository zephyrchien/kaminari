use std::io::Result;
use std::future::Future;
use std::sync::Arc;
use std::fmt::{Debug, Display, Formatter};

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
    sni: ServerName,
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

        let sni: ServerName = sni.as_str().try_into().expect("invalid DNS name");

        let mut conf = ClientConfig::builder()
            .with_safe_defaults()
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

// ========== server ==========
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsServerConf {
    pub crt: String,
    pub key: String,
    pub ocsp: String,
    pub server_name: String,
}

impl Display for TlsServerConf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cert: {}, key: {}, oscp: {}, server_name: {}",
            self.crt, self.key, self.ocsp, self.server_name
        )
    }
}

#[derive(Clone)]
pub struct TlsAccept<T> {
    lis: T,
    ac: TlsAcceptor,
}

impl<T> Display for TlsAccept<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "[tls]{}", self.lis) }
}

impl<T> Debug for TlsAccept<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TlsAccept").field("lis", &self.lis).finish()
    }
}

impl<T> TlsAccept<T> {
    pub fn new(lis: T, conf: TlsServerConf) -> Self {
        let TlsServerConf {
            crt,
            key,
            ocsp,
            server_name,
        } = conf;

        let (cert, key) = if !crt.is_empty() && !key.is_empty() {
            (
                utils::read_certificates(&crt).expect("failed to read certificate"),
                utils::read_private_key(&key).expect("failed to read private key"),
            )
        } else if !server_name.is_empty() {
            utils::generate_self_signed(&server_name)
        } else {
            panic!("no certificate or private key supplied")
        };

        let ocsp = if !ocsp.is_empty() {
            utils::read_ocsp(&ocsp).expect("failed to read ocsp")
        } else {
            Vec::new()
        };

        let conf = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert_with_ocsp_and_sct(cert, key, ocsp, Vec::new())
            .expect("bad certificate or key");

        Self {
            lis,
            ac: Arc::new(conf).into(),
        }
    }

    // use shared cert, key
    pub fn new_shared(lis: T, conf: TlsServerConf) -> Self {
        let TlsServerConf {
            crt,
            key,
            ocsp,
            server_name,
        } = conf;

        let ocsp = if !ocsp.is_empty() {
            Some(utils::read_ocsp(&ocsp).expect("failed to read ocsp"))
        } else {
            None
        };

        let cert_resolver = if !crt.is_empty() && !key.is_empty() {
            utils::new_crt_key_resolver(crt, key, ocsp, None)
        } else if !server_name.is_empty() {
            utils::new_self_signed_resolver(server_name)
        } else {
            panic!("no certificate or private key supplied")
        };

        let conf = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_cert_resolver(cert_resolver);

        Self {
            lis,
            ac: Arc::new(conf).into(),
        }
    }
}

impl<S, T> AsyncAccept<S> for TlsAccept<T>
where
    S: IOStream,
    T: AsyncAccept<S>,
{
    type Stream = TlsServerStream<T::Stream>;

    type AcceptFut<'a> = impl Future<Output = Result<Self::Stream>> +'a where Self:'a;

    fn accept<'a>(&'a self, stream: S, buf: &'a mut [u8]) -> Self::AcceptFut<'a> {
        async move {
            let stream = self.lis.accept(stream, buf).await?;
            self.ac.accept(stream).await
        }
    }
}

#[allow(unused)]
mod utils {
    pub use client::*;
    pub use server::*;

    mod client {
        use std::sync::Arc;
        use tokio_rustls::rustls;
        use rustls::{Certificate, PrivateKey};
        use rustls::{RootCertStore, OwnedTrustAnchor};
        use rustls::client::{ServerCertVerified, ServerCertVerifier, WebPkiVerifier};
        use webpki_roots::TLS_SERVER_ROOTS;
        use lazy_static::lazy_static;

        pub fn firefox_roots() -> RootCertStore {
            let mut roots = RootCertStore::empty();
            roots.add_trust_anchors(TLS_SERVER_ROOTS.iter().map(|x| {
                OwnedTrustAnchor::from_subject_spki_name_constraints(
                    x.subject(),
                    x.subject_public_key_info.into(),
                    x.name_constraints.into(),
                )
            }));
            roots
        }

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
            ) -> std::result::Result<rustls::client::ServerCertVerified, rustls::Error>
            {
                Ok(ServerCertVerified::assertion())
            }
        }

        fn new_insecure_verifier() -> Arc<SkipVerify> {
            lazy_static! {
                static ref ARC: Arc<SkipVerify> = Arc::new(SkipVerify {});
            }
            ARC.clone()
        }

        fn new_firefox_verifier() -> Arc<WebPkiVerifier> {
            lazy_static! {
                static ref ARC: Arc<WebPkiVerifier> =
                    Arc::new(WebPkiVerifier::new(firefox_roots(), None));
            }
            ARC.clone()
        }

        pub fn new_verifier(insecure: bool) -> Arc<dyn ServerCertVerifier> {
            if insecure {
                new_insecure_verifier()
            } else {
                new_firefox_verifier()
            }
        }
    }

    mod server {
        use std::io::{BufReader, Result};
        use std::fs::{self, File};
        use std::sync::{Arc, Mutex};

        use tokio_rustls::rustls;
        use rustls::{Certificate, PrivateKey};
        use rustls::sign;
        use rustls::server::ResolvesServerCert;
        use rustls::server::ClientHello;

        use rustls_pemfile::Item;
        use webpki_roots::TLS_SERVER_ROOTS;

        use lazy_static::lazy_static;

        // copy & paste from https://github.com/EAimTY/tuic/blob/master/server/src/certificate.rs
        pub fn read_certificates(path: &str) -> Result<Vec<Certificate>> {
            let mut file = BufReader::new(File::open(path)?);
            let mut certs = Vec::new();

            // pem
            while let Ok(Some(item)) = rustls_pemfile::read_one(&mut file) {
                if let Item::X509Certificate(cert) = item {
                    certs.push(Certificate(cert.to_vec()));
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
                priv_key = Some(match item {
                    Item::Pkcs1Key(k) => k.secret_pkcs1_der().to_vec(),
                    Item::Pkcs8Key(k) => k.secret_pkcs8_der().to_vec(),
                    Item::Sec1Key(k) => k.secret_sec1_der().to_vec(),
                    _ => continue,
                })
            }

            // der
            priv_key
                .map(Ok)
                .unwrap_or_else(|| fs::read(path))
                .map(PrivateKey)
        }

        pub fn read_ocsp(path: &str) -> Result<Vec<u8>> { fs::read(path) }

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

        // copy from rustls:
        // https://docs.rs/rustls/latest/src/rustls/server/handy.rs.html
        pub struct AlwaysResolvesChain(Arc<sign::CertifiedKey>);

        impl ResolvesServerCert for AlwaysResolvesChain {
            fn resolve(&self, _: ClientHello) -> Option<Arc<sign::CertifiedKey>> {
                Some(Arc::clone(&self.0))
            }
        }

        pub fn new_resolver(
            chain: Vec<Certificate>,
            priv_key: &PrivateKey,
            ocsp: Option<Vec<u8>>,
            scts: Option<Vec<u8>>,
        ) -> Arc<AlwaysResolvesChain> {
            let key = sign::any_supported_type(priv_key).expect("invalid key");
            Arc::new(AlwaysResolvesChain(Arc::new(sign::CertifiedKey {
                cert: chain,
                key,
                ocsp,
                sct_list: scts,
            })))
        }

        pub fn new_self_signed_resolver(server_name: String) -> Arc<AlwaysResolvesChain> {
            type Store = Mutex<Vec<(String, Arc<AlwaysResolvesChain>)>>;
            lazy_static! {
                static ref STORE: Store = { Mutex::new(Vec::new()) };
            }

            // hold the lock
            let mut store = STORE.lock().unwrap();

            // simply increase ref count
            if let Some(x) = store.iter().find(|(x, _)| *x == server_name) {
                return x.1.clone();
            }

            // generate a new cert
            let (cert, key) = generate_self_signed(&server_name);
            let resolver = new_resolver(cert, &key, None, None);

            store.push((server_name, resolver.clone()));
            store.shrink_to_fit();

            resolver
        }

        pub fn new_crt_key_resolver(
            crt: String,
            key: String,
            ocsp: Option<Vec<u8>>,
            scts: Option<Vec<u8>>,
        ) -> Arc<AlwaysResolvesChain> {
            type Store = Mutex<Vec<(String, Arc<AlwaysResolvesChain>)>>;
            lazy_static! {
                static ref STORE: Store = { Mutex::new(Vec::new()) };
            }

            // hold the lock
            let mut store = STORE.lock().unwrap();

            // find based on key path, no real data
            // simply increase ref count
            if let Some(x) = store.iter().find(|(x, _)| *x == key) {
                return x.1.clone();
            }

            // read cert and key
            let cert = read_certificates(&crt).expect("failed to read certificate");
            let priv_key = read_private_key(&key).expect("failed to read private key");
            let resolver = new_resolver(cert, &priv_key, ocsp, scts);

            store.push((key, resolver.clone()));
            store.shrink_to_fit();

            resolver
        }
    }
}
