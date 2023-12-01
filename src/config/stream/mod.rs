mod iota_streams;
mod mqtt;

use alvarium_annotator::{SignProvider, StreamConfigWrapper};
pub use iota_streams::*;
pub use mqtt::*;

use serde::{Serialize, Deserialize};

use crate::annotations::constants::StreamType;
use crate::providers::sign_provider::SignatureProviderWrap;


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamInfo {
    #[serde(rename="type")]
    pub stream_type: String,
    pub config: StreamConfig
}

impl StreamConfigWrapper for StreamInfo {
    fn stream_type(&self) -> StreamType {
        StreamType(&self.stream_type)
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlInfo {
    pub host: String,
    pub port: usize,
    pub protocol: String
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

    pub(crate) fn verify_signature(&self, provider: &SignatureProviderWrap) -> Result<bool, String> {
        if self.signature.is_empty() {
            return Err(format!("signature field is empty"))
        }

        match provider {
            SignatureProviderWrap::Ed25519(provider)=> {
                let sig_bytes = hex::decode(&self.signature).map_err(|e| e.to_string())?;
                provider.verify(self.seed.as_bytes(), &sig_bytes)
                    .map_err(|e| e.to_string())
            }
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
    use crate::providers::sign_provider::{Ed25519Provider, SignatureProviderWrap};
    use crate::config;
    use super::Signable;
    use alvarium_annotator::SignProvider;

    #[tokio::test]
    async fn verify_signable() {
        let config: config::SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        let sig_provider = SignatureProviderWrap::Ed25519(Ed25519Provider::new(&config.signature).unwrap());

        let data = "A data packet to sign".to_string();
        let sig = sig_provider.sign(data.as_bytes()).unwrap();

        let signable = Signable {
            seed: data,
            signature: sig
        };

        assert!(signable.verify_signature(&sig_provider).unwrap())
    }

    #[test]
    fn failed_verification_signable() {
        let config: config::SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        let bad_priv_key = SecretKey::generate().unwrap();

        let data = "A data packet to sign".to_string();
        let raw_sig = bad_priv_key.sign(data.as_bytes());

        let signable = Signable {
            seed: data,
            signature: hex::encode(raw_sig.to_bytes())
        };

        let sig_provider = SignatureProviderWrap::Ed25519(Ed25519Provider::new(&config.signature).unwrap());

        assert!(!signable.verify_signature(&sig_provider).unwrap())
    }
}