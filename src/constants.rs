use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct HashType<'a>(pub &'a str);

pub const MD5_HASH: HashType = HashType("md5");
pub const SHA256_HASH: HashType = HashType("sha256");
pub const NO_HASH: HashType = HashType("none");

impl HashType<'_> {
    pub(crate) fn validate(&self) -> bool {
        self == &MD5_HASH || self == &SHA256_HASH || self == &NO_HASH
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct KeyAlgorithm<'a>(pub &'a str);

pub const ED25519_KEY: KeyAlgorithm = KeyAlgorithm("ed25519");

impl KeyAlgorithm<'_> {
    pub(crate) fn validate(&self) -> bool {
        self == &ED25519_KEY
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct StreamType<'a>(pub &'a str);

pub const IOTA_STREAM: StreamType = StreamType("iota");
pub const MOCK_STREAM: StreamType = StreamType("mock");
pub const MQTT_STREAM: StreamType = StreamType("mqtt");
pub const PRAVEGA_STREAM: StreamType = StreamType("pravega");

impl StreamType<'_> {
    pub(crate) fn validate(&self) -> bool {
        self == &IOTA_STREAM || self == &MOCK_STREAM || self == &MQTT_STREAM || self == &PRAVEGA_STREAM
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AnnotationType<'a>(pub &'a str);

pub const ANNOTATION_PKI: AnnotationType = AnnotationType("pki");
pub const ANNOTATION_SOURCE: AnnotationType = AnnotationType("source");
pub const ANNOTATION_TLS: AnnotationType = AnnotationType("tls");
pub const ANNOTATION_TPM: AnnotationType = AnnotationType("tpm");

impl AnnotationType<'_> {
    pub(crate) fn validate(&self) -> bool {
        self == &ANNOTATION_PKI || self == &ANNOTATION_SOURCE || self == &ANNOTATION_TLS || self == &ANNOTATION_TPM
    }
}
