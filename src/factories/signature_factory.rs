
use alvarium_annotator::constants;
use crate::config::SignatureInfo;
use crate::providers::sign_provider::{Ed25519Provider, SignatureProviderWrap};


pub fn new_signature_provider(config: &SignatureInfo) -> Result<SignatureProviderWrap, String> {
    match config.private_key_info.key_algorithm() {
        constants::ED25519_KEY => Ok(SignatureProviderWrap::Ed25519(Ed25519Provider::new(config)?)),
        _ => Err(format!("not a pre known signature provider, please build separate"))
    }
}