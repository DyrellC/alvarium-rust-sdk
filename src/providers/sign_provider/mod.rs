mod ed25519;

use std::error::Error;
pub use ed25519::*;

pub enum SignatureProviderWrap {
    Ed25519(Ed25519Provider)
}

impl alvarium_annotator::SignProvider for SignatureProviderWrap {
    fn sign(&self, content: &[u8]) -> Result<String, Box<dyn Error>> {
        match self {
            SignatureProviderWrap::Ed25519(provider) => provider.sign(content)
        }
    }

    fn verify(&self, content: &[u8], signed: &[u8]) -> Result<bool, Box<dyn Error>> {
        match self {
            SignatureProviderWrap::Ed25519(provider) => provider.verify(content, signed)
        }
    }

}