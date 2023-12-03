use crate::config::SignatureInfo;
use crate::providers::sign_provider::{Ed25519Provider, SignatureProviderWrap};


pub fn new_signature_provider(config: &SignatureInfo) -> Result<SignatureProviderWrap, String> {
    if !config.private_key_info.key_type.is_base_key_algorithm() {
        return Err(format!("not a pre known signature provider, please build separate"))
    }

    match config.private_key_info.key_type.0.as_str() {
        "ed25519" => Ok(SignatureProviderWrap::Ed25519(Ed25519Provider::new(config)?)),
        _ => Err(format!("not a pre known signature provider, please build separate"))
    }
}