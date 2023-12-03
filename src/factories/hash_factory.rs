use alvarium_annotator::constants;
use crate::providers::hash_provider::{HashProviderWrapper, MD5Provider, NoneProvider, Sha256Provider};


pub fn new_hash_provider(kind: &constants::HashType) -> Result<HashProviderWrapper, String> {
    if !kind.is_base_hash_type() {
        return Err(format!("not a pre known hash provider {}, please build separate", kind.0))
    }


    match kind.0.as_str() {
        "md5" => Ok(HashProviderWrapper::MD5(MD5Provider::new())),
        "sha256" => Ok(HashProviderWrapper::Sha256(Sha256Provider::new())),
        "none" => Ok(HashProviderWrapper::None(NoneProvider::new())),
        _ => Err(format!("not a pre known hash provider {}, please build separate", kind.0))
    }
}

