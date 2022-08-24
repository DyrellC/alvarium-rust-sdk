mod md5_provider;
mod none_provider;
mod sha256_provider;

pub use md5_provider::MD5Provider;
pub use none_provider::NoneProvider;
pub use sha256_provider::Sha256Provider;

pub trait HashProvider {
    fn derive(data: &[u8]) -> String;
}