use crate::annotations::{
    Annotation,
    Annotator,
    constants,
};
use crate::{config::{self, Signable}};
use crate::annotations::{derive_hash, sign_annotation};

pub struct PkiAnnotator<'a> {
    hash: constants::HashType<'a>,
    kind: constants::AnnotationType<'a>,
    sign: config::SignatureInfo<'a>,
}

impl<'a> PkiAnnotator<'a> {
    pub fn new(cfg: &config::SdkInfo<'a>) -> impl Annotator + 'a {
        PkiAnnotator {
            hash: cfg.hash.hash_type,
            kind: constants::ANNOTATION_PKI,
            sign: cfg.signature.clone(),
        }
    }
}

impl<'a> Annotator for PkiAnnotator<'a> {
    fn annotate(&self, data: &[u8]) -> Result<Annotation, String> {
        let key = derive_hash(self.hash, data);
        match gethostname::gethostname().to_str() {
            Some(host) => {
                let signable: Result<Signable, serde_json::Error> = serde_json::from_slice(data);
                match signable {
                    Ok(signable) => {
                        let verified = signable.verify_signature(&self.sign.public_key_info)?;
                        let mut annotation = Annotation::new(&key, self.hash, host, self.kind, verified);
                        let signature = sign_annotation(&self.sign, &annotation)?;
                        annotation.with_signature(&signature);
                        Ok(annotation)
                    }
                    Err(_) => Err(format!("could not deserialize signature"))
                }

            },
            None => Err(format!("could not retrieve host name"))
        }
    }
}


#[cfg(test)]
mod pki_tests {
    use crate::{config, providers::sign_provider::get_priv_key};
    use crate::annotations::{Annotator, PkiAnnotator, constants};
    use crate::config::Signable;

    #[test]
    fn valid_and_invalid_pki_annotator() {
        let config_file = std::fs::read("resources/test_config.json").unwrap();
        let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

        let mut config2 = config.clone();
        config2.hash.hash_type = constants::HashType("Not a known hash type");

        let data = String::from("Some random data");
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);

        let signable = Signable::new(data, sig);
        let serialised = serde_json::to_vec(&signable).unwrap();

        let pki_annotator_1 = PkiAnnotator::new(&config);
        let pki_annotator_2 = PkiAnnotator::new(&config2);
        let valid_annotation = pki_annotator_1.annotate(&serialised).unwrap();
        let invalid_annotation = pki_annotator_2.annotate(&serialised).unwrap();

        assert!(valid_annotation.validate());
        assert!(!invalid_annotation.validate());
    }


    #[test]
    fn make_pki_annotation() {
        let config_file = std::fs::read("resources/test_config.json").unwrap();
        let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

        let data = String::from("Some random data");
        let priv_key_file = std::fs::read(config.signature.private_key_info.path).unwrap();
        let priv_key_string = String::from_utf8(priv_key_file).unwrap();
        let priv_key = get_priv_key(&priv_key_string).unwrap();
        let sig = priv_key.sign(data.as_bytes());

        let signable = Signable::new(data, hex::encode(sig.to_bytes()));
        let serialised = serde_json::to_vec(&signable).unwrap();

        let pki_annotator = PkiAnnotator::new(&config);
        let annotation = pki_annotator.annotate(&serialised).unwrap();

        assert!(annotation.validate());
        assert_eq!(annotation.kind, constants::ANNOTATION_PKI);
        assert_eq!(annotation.host, gethostname::gethostname().to_str().unwrap());
        assert_eq!(annotation.hash, config.hash.hash_type);
        assert!(annotation.is_satisfied)
    }

    #[test]
    fn unsatisfied_pki_annotation() {
        let config_file = std::fs::read("resources/test_config.json").unwrap();
        let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

        let data = String::from("Some random data");
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);

        let signable = Signable::new(data, sig);
        let serialised = serde_json::to_vec(&signable).unwrap();

        let pki_annotator = PkiAnnotator::new(&config);
        let annotation = pki_annotator.annotate(&serialised).unwrap();

        assert!(annotation.validate());
        assert_eq!(annotation.kind, constants::ANNOTATION_PKI);
        assert_eq!(annotation.host, gethostname::gethostname().to_str().unwrap());
        assert_eq!(annotation.hash, config.hash.hash_type);
        assert!(!annotation.is_satisfied)
    }
}