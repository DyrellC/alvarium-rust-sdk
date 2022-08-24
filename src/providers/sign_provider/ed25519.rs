use crate::providers::sign_provider::SignProvider;

pub struct Ed25519Provider {}

impl Ed25519Provider {
    pub fn new() -> Self {
        Ed25519Provider {}
    }
}

impl SignProvider for Ed25519Provider {
    fn sign(key: &str, content: &[u8]) -> Result<String, String> {
        let priv_key = get_priv_key(key)?;
        Ok(hex::encode(priv_key.sign(content).to_bytes()))
    }


    fn verify(key: &str, content: &[u8], signed: &str) -> Result<bool, String> {
        let pub_key = get_pub_key(key)?;
        let sig = get_signature(signed)?;
        Ok(pub_key.verify(&sig,content))
    }
}


pub(crate) fn get_priv_key(key: &str) -> Result<crypto::signatures::ed25519::SecretKey, &str> {
    match hex::decode(key) {
        Ok(decoded_key) => {
            match <[u8;crypto::signatures::ed25519::SECRET_KEY_LENGTH]>::try_from(decoded_key.as_slice()) {
                Ok(resized) => Ok(crypto::signatures::ed25519::SecretKey::from_bytes(resized)),
                Err(_) => Err("decoded private key is not the correct size")
            }
        },
        Err(_) => Err("could not decode private key")
    }
}


pub(crate) fn get_pub_key(key: &str) -> Result<crypto::signatures::ed25519::PublicKey, &str> {
    match hex::decode(key) {
        Ok(decoded_key) => {
            match <[u8;crypto::signatures::ed25519::PUBLIC_KEY_LENGTH]>::try_from(decoded_key.as_slice()) {
                Ok(resized) => {
                    match crypto::signatures::ed25519::PublicKey::try_from_bytes(resized) {
                        Ok(pub_key) => Ok(pub_key),
                        Err(_) => Err("error making public key")
                    }
                }
                Err(_) => Err("decoded public key is not the correct size")
            }
        },
        Err(_) => Err("could not decode public key")
    }
}


fn get_signature(signature: &str) -> Result<crypto::signatures::ed25519::Signature, &str> {
    match hex::decode(signature) {
        Ok(decoded_sig) => {
            match <[u8;crypto::signatures::ed25519::SIGNATURE_LENGTH]>::try_from(decoded_sig.as_slice()) {
                Ok(resized) => Ok(
                    crypto::signatures::ed25519::Signature::from_bytes(resized)
                ),
                Err(_) => Err("decoded signature is not the correct size")
            }
        },
        Err(_) => Err("could not decode signature")
    }
}