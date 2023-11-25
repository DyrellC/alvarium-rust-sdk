mod iota_streams;
mod mqtt;

use alvarium_annotator::constants::KeyAlgorithm;
pub use iota_streams::*;
pub use mqtt::*;

use serde::{Serialize, Deserialize};
use crate::config;
use crate::providers::sign_provider::{Ed25519Provider, SignProvider};

use crate::annotations::constants::{ED25519_KEY, StreamType};

pub trait StreamConfigWrapper {
    fn stream_type(&self) -> StreamType;
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamInfo {
    #[serde(rename="type")]
    pub(crate) stream_type: String,
    pub(crate) config: StreamConfig
}

impl StreamConfigWrapper for StreamInfo {
    fn stream_type(&self) -> StreamType {
        StreamType(&self.stream_type)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlInfo {
    pub(crate) host: String,
    pub(crate) port: usize,
    pub(crate) protocol: String
}

impl UrlInfo {
    pub fn uri(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StreamConfig {
    IotaStreams(IotaStreamsConfig),
    MQTT(MqttStreamConfig),
}


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Signable {
    seed: String,
    signature: String
}

impl Signable {
    pub(crate) fn new(seed: String, signature: String) -> Self {
        Signable { seed, signature }
    }

    pub(crate) fn verify_signature(&self, key: &config::KeyInfo) -> Result<bool, String> {
        if self.signature.is_empty() {
            return Err(format!("signature field is empty"))
        }

        match KeyAlgorithm(&key.key_type) {
            ED25519_KEY => {
                match std::fs::read_to_string(key.path.clone()) {
                    Ok(pub_key) =>
                        Ed25519Provider::verify(&pub_key, self.seed.as_bytes(), self.signature.as_str()),
                    Err(_) => Err(format!("pub key could not be read from provided path"))
                }
            }
            _ => Err(format!("unrecognized key type"))
        }
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        // Strings should not fail to serde
        // TODO: Verify that this is the case
        serde_json::to_vec(&self).unwrap()
    }
}


#[cfg(test)]
mod config_tests {
    use crypto::signatures::ed25519::SecretKey;
    use super::{config, Signable};

    #[test]
    fn verify_signable() {
        let config: config::SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();

        let priv_key_file = std::fs::read(&config.signature.private_key_info.path).unwrap();
        let priv_key_bytes = hex::decode(String::from_utf8(priv_key_file).unwrap()).unwrap();
        let priv_key = SecretKey::from_bytes(<[u8; 32]>::try_from(priv_key_bytes.as_slice()).unwrap());

        let data = "A data packet to sign".to_string();
        let sig = priv_key.sign(data.as_bytes());

        let signable = Signable {
            seed: data,
            signature: hex::encode(sig.to_bytes())
        };

        assert!(signable.verify_signature(&config.signature.public_key_info).unwrap())
    }

    #[test]
    fn failed_verification_signable() {
        let config: config::SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        let bad_priv_key = SecretKey::generate().unwrap();

        let data = "A data packet to sign".to_string();
        let sig = bad_priv_key.sign(data.as_bytes());

        let signable = Signable {
            seed: data,
            signature: hex::encode(sig.to_bytes())
        };

        assert!(!signable.verify_signature(&config.signature.public_key_info).unwrap())
    }
}