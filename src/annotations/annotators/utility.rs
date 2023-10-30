use crate::annotations::{mock_annotation};
use crate::config;
use crate::config::SignatureInfo;
use crate::annotations::{Annotation, constants::{self, HashType, MD5_HASH, SHA256_HASH}};
use crate::providers::hash_provider::{MD5Provider, Sha256Provider, NoneProvider, HashProvider};
use crate::providers::sign_provider::{Ed25519Provider, SignProvider};

pub fn derive_hash(hash_type: HashType, data: &[u8]) -> String {
    match hash_type {
        MD5_HASH => MD5Provider::derive(data),
        SHA256_HASH => Sha256Provider::derive(data),
        _ => NoneProvider::derive(data)
    }
}

pub(crate) fn sign_annotation(key: &SignatureInfo, annotation: &Annotation) -> Result<String, String> {
    match key.private_key_info.key_type {
        constants::ED25519_KEY => {
            let signature = serialise_and_sign(Ed25519Provider::sign, key, annotation)?;
            Ok(signature)
        },
        _ => Err(format!("unrecognized key type"))
    }

}


fn serialise_and_sign<F>(sign: F, key: &SignatureInfo, annotation: &Annotation) -> Result<String, String>
where
    F: FnOnce(&str, &[u8]) -> Result<String, String>
{
    let serialised = serde_json::to_vec(annotation);
    match serialised {
        Ok(annotation_bytes) => {
            if let Ok(file) = std::fs::read(key.private_key_info.path) {
                if let Ok(priv_key) = String::from_utf8(file) {
                    return sign(&priv_key, &annotation_bytes)
                }
            }
            Err(format!("could not retrieve private key from path"))
        }
        Err(_) => Err(format!("annotation could not be serialised"))
    }
}


#[test]
fn sign_and_verify_annotation() {
    let config_file = std::fs::read("resources/test_config.json").unwrap();
    let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

    let annotation = mock_annotation();
    let signature = sign_annotation(&config.signature, &annotation);
    assert!(signature.is_ok())
}