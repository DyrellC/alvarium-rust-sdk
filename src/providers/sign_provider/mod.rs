mod ed25519;

pub use ed25519::*;

pub trait SignProvider {
    fn sign(key: &str, content: &[u8]) -> Result<String, String>;
    fn verify(key: &str, content: &[u8], signed: &str) -> Result<bool, String>;
}