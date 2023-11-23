use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::sync::Arc;
use rustls::{Connection, ServerConfig, Stream};
use crate::annotations::{
    Annotation,
    Annotator,
    constants,
};
use crate::config;
use crate::annotations::{derive_hash, sign_annotation};


#[cfg(feature = "native-tls")]
use native_tls::{TlsConnector, TlsAcceptor, TlsStream};
#[cfg(feature = "rustls")]
use rustls::{ClientConnection, ServerConnection, ClientConfig, RootCertStore};
use std::net::TcpStream;
use std::sync::Mutex;

pub struct TlsAnnotator<'a> {
    hash: constants::HashType<'a>,
    kind: constants::AnnotationType<'a>,
    sign: config::SignatureInfo<'a>,

    // TODO: Make type for this
    #[cfg(feature = "native-tls")]
    conn_native: Option<Mutex<TlsStream<TcpStream>>>,

    #[cfg(feature = "rustls")]
    conn_rustls: Option<Connection>,
    #[cfg(feature = "rustls")]
    stream: Option<TcpStream>,
}

impl<'a> TlsAnnotator<'a> {
    pub fn new(cfg: &config::SdkInfo<'a>) -> impl Annotator + Tls<'a> + 'a {
        TlsAnnotator {
            hash: cfg.hash.hash_type,
            kind: constants::ANNOTATION_TLS,
            sign: cfg.signature.clone(),
            #[cfg(feature = "native-tls")]
            conn_native: None,
            #[cfg(feature = "rustls")]
            conn_rustls: None,
            #[cfg(feature = "rustls")]
            stream: None,
        }
    }
}

pub trait Tls<'a> {
    #[cfg(feature = "native-tls")]
    fn set_connection_native(&mut self, tls_stream: TlsStream<TcpStream>);
    #[cfg(feature = "rustls")]
    fn set_connection_rustls(&mut self, conn: Connection, stream: TcpStream);

    #[cfg(feature = "native-tls")]
    fn check_tls_stream_native(&self) -> bool;
    #[cfg(feature = "rustls")]
    fn check_tls_stream_rustls(&self) -> bool;
}

impl<'a> Tls<'a> for TlsAnnotator<'a> {
    #[cfg(feature = "native-tls")]
    fn set_connection_native(&mut self, tls_stream: TlsStream<TcpStream>) {
        self.client_conn_native = Some(Mutex::new(tls_stream));
    }

    #[cfg(feature = "rustls")]
    fn set_connection_rustls(&mut self, conn: Connection, stream: TcpStream) {
        self.stream = Some(stream);
        self.conn_rustls = Some(conn);
    }

    #[cfg(feature = "native-tls")]
    fn check_tls_stream_native(&self) -> bool {
        match &self.conn_native {
            Some(conn) => conn.lock().unwrap().get_ref().peer_certificate().is_ok(),
            None => false
        }
    }

    #[cfg(feature = "rustls")]
    fn check_tls_stream_rustls(&self) -> bool {
        if let Some(stream) = &self.stream {
            println!("Stream exists");
            if let Some(conn) = &self.conn_rustls {
                println!("Connection exists");
                let mut retries = 0;
                loop {
                    if !conn.is_handshaking() {
                        break;
                    } else {
                        retries += 1;
                        std::thread::sleep(std::time::Duration::from_millis(400));
                        if retries == 5 {
                            println!("Max retries attempted for handshake");
                            return false
                        }
                    }
                }

                let mut buffer = [0; 1];
                match stream.peek(&mut buffer) {
                    Ok(_) => true,
                    Err(e) => {
                        match e.kind() {
                            std::io::ErrorKind::WouldBlock => true,
                            std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe => {
                                println!("Connection error in TLS stream: {:?}", e);
                                false
                            },
                            _ => {
                                println!("Unexpected error in TLS stream: {:?}", e);
                                false
                            }
                        }
                    }
                }
            } else {
                false
            }
        } else {
            false
        }
    }

}


// Create a TLS Server Connection instance to determine if it is being used
impl<'a> Annotator for TlsAnnotator<'a> {
    fn annotate(&self, data: &[u8]) -> Result<Annotation, String> {
        let key = derive_hash(self.hash, data);
        match gethostname::gethostname().to_str() {
            Some(host) => {
                #[cfg(all(not(feature = "rustls"), feature = "native-tls"))]
                let is_satisfied = self.check_handshake_native();
                #[cfg(feature = "rustls")]
                let is_satisfied = self.check_tls_stream_rustls();

                let mut annotation = Annotation::new(&key, self.hash, host, self.kind, is_satisfied);
                let signature = sign_annotation(&self.sign, &annotation)?;
                annotation.with_signature(&signature);
                Ok(annotation)
            },
            None => Err(format!("could not retrieve host name"))
        }
    }
}


#[cfg(test)]
mod tls_tests {
    use std::sync::Arc;
    use rustls::{ClientConnection, Connection, Stream, StreamOwned};
    use std::net::TcpStream;
    use crate::{config, providers::sign_provider::get_priv_key};
    use crate::annotations::{Annotator, constants, TlsAnnotator};
    use crate::config::Signable;
    #[cfg(feature = "rustls")]
    use super::Tls;

    #[test]
    fn valid_and_invalid_tls_annotator() {
        let config_file = std::fs::read("resources/test_config.json").unwrap();
        let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

        let mut config2 = config.clone();
        config2.hash.hash_type = constants::HashType("Not a known hash type");

        let data = String::from("Some random data");
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);

        let signable = Signable::new(data, sig);
        let serialised = serde_json::to_vec(&signable).unwrap();

        let tls_annotator_1 = TlsAnnotator::new(&config);
        let tls_annotator_2 = TlsAnnotator::new(&config2);
        let valid_annotation = tls_annotator_1.annotate(&serialised).unwrap();
        let invalid_annotation = tls_annotator_2.annotate(&serialised).unwrap();

        assert!(valid_annotation.validate());
        assert!(!invalid_annotation.validate());
    }

    #[cfg(feature = "rustls")]
    #[test]
    fn make_tls_annotation() {
        let config_file = std::fs::read("resources/test_config.json").unwrap();
        let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

        let data = String::from("Some random data");
        let priv_key_file = std::fs::read(config.signature.private_key_info.path).unwrap();
        let priv_key_string = String::from_utf8(priv_key_file).unwrap();
        let priv_key = get_priv_key(&priv_key_string).unwrap();
        let sig = priv_key.sign(data.as_bytes());

        let signable = Signable::new(data, hex::encode(sig.to_bytes()));
        let serialised = serde_json::to_vec(&signable).unwrap();

        let mut tls_annotator = TlsAnnotator::new(&config);

        let conn = make_client_connection().unwrap();
        let tcp_stream = TcpStream::connect("www.google.com:443").unwrap();
        tls_annotator.set_connection_rustls(conn.into(), tcp_stream);

        let annotation = tls_annotator.annotate(&serialised).unwrap();

        assert!(annotation.validate());
        assert_eq!(annotation.kind, constants::ANNOTATION_TLS);
        assert_eq!(annotation.host, gethostname::gethostname().to_str().unwrap());
        assert_eq!(annotation.hash, config.hash.hash_type);
        assert!(annotation.is_satisfied)
    }

    #[test]
    fn unsatisfied_tls_annotation() {
        let config_file = std::fs::read("resources/test_config.json").unwrap();
        let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

        let data = String::from("Some random data");
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);

        let signable = Signable::new(data, sig);
        let serialised = serde_json::to_vec(&signable).unwrap();

        let tls_annotator = TlsAnnotator::new(&config);
        let annotation = tls_annotator.annotate(&serialised).unwrap();

        assert!(annotation.validate());
        assert_eq!(annotation.kind, constants::ANNOTATION_TLS);
        assert_eq!(annotation.host, gethostname::gethostname().to_str().unwrap());
        assert_eq!(annotation.hash, config.hash.hash_type);
        assert!(!annotation.is_satisfied)
    }

    #[cfg(feature = "rustls")]
    fn make_client_connection() -> Result<ClientConnection, Box<dyn std::error::Error>> {
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_trust_anchors(
            webpki_roots::TLS_SERVER_ROOTS
                .0
                .iter()
                .map(|ta| {
                    rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                })
        );
        let config = rustls::ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13])
            .unwrap()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let server_name = "www.google.com".try_into().unwrap();
        Ok(ClientConnection::new(Arc::new(config), server_name).unwrap())
    }

}