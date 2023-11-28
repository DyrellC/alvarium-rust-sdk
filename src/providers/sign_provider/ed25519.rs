use alvarium_annotator::SignProvider;
use crate::config::SignatureInfo;
use crypto::signatures::ed25519::{PublicKey, SecretKey, Signature};

pub struct Ed25519Provider {
    public: PublicKey,
    private: SecretKey

}

impl Ed25519Provider {
    pub fn new(config: &SignatureInfo) -> Result<Self, String> {
        let pub_key_file = std::fs::read(&config.public_key_info.path).unwrap();
        let pub_key_string = String::from_utf8(pub_key_file).unwrap();
        let pk = get_pub_key(&pub_key_string)?;

        let priv_key_file = std::fs::read(&config.private_key_info.path).unwrap();
        let priv_key_string = String::from_utf8(priv_key_file).unwrap();
        let sk = get_priv_key(&priv_key_string)?;

        Ok(Ed25519Provider {
            public: pk,
            private: sk,
        })
    }
}

impl SignProvider for Ed25519Provider {
    fn sign(&self, content: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(hex::encode(self.private.sign(content).to_bytes()))
    }


    fn verify(&self, content: &[u8], signed: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
        let sig = get_signature(signed)?;
        Ok(self.public.verify(&sig,content))
    }
}


pub(crate) fn get_priv_key(key: &str) -> Result<SecretKey, &str> {
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


pub(crate) fn get_pub_key(key: &str) -> Result<PublicKey, &str> {
    match hex::decode(key) {
        Ok(decoded_key) => {
            match <[u8;crypto::signatures::ed25519::PUBLIC_KEY_LENGTH]>::try_from(decoded_key.as_slice()) {
                Ok(resized) => {
                    match PublicKey::try_from_bytes(resized) {
                        Ok(pub_key) => Ok(pub_key),
                        Err(_) => Err("error making public key")
                    }
                }
                Err(_) => Err("decoded public key is not the correct size")
            }
        }
        Err(_) => Err("could not decode public key")
    }
}


fn get_signature(signature: &[u8]) -> Result<Signature, &str> {
    match <[u8;crypto::signatures::ed25519::SIGNATURE_LENGTH]>::try_from(signature) {
        Ok(resized) => Ok(Signature::from_bytes(resized)),
        Err(_) => Err("decoded signature is not the correct size")
    }
}