use alvarium_annotator::constants;
use crate::providers::hash_provider::{HashProviderWrapper, MD5Provider, NoneProvider, Sha256Provider};


pub fn new_hash_provider(kind: &constants::HashType) -> Result<HashProviderWrapper, String> {
    match kind {
        &constants::MD5_HASH => Ok(HashProviderWrapper::MD5(MD5Provider::new())),
        &constants::SHA256_HASH => Ok(HashProviderWrapper::Sha256(Sha256Provider::new())),
        &constants::NO_HASH => Ok(HashProviderWrapper::None(NoneProvider::new())),
        _ => Err(format!("not a pre known hash provider {}, please build separate", kind.0))
    }
}

