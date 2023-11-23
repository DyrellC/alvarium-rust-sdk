use crate::annotations::{
    Annotation,
    Annotator,
    constants,
};
use crate::config;
use crate::annotations::{derive_hash, sign_annotation};

pub struct SourceAnnotator<'a> {
    hash: constants::HashType<'a>,
    kind: constants::AnnotationType<'a>,
    sign: config::SignatureInfo<'a>,
}

impl<'a> SourceAnnotator<'a> {
    pub fn new(cfg: &config::SdkInfo<'a>) -> impl Annotator + 'a {
        SourceAnnotator {
            hash: cfg.hash.hash_type,
            kind: constants::ANNOTATION_SOURCE,
            sign: cfg.signature.clone(),
        }
    }
}

impl<'a> Annotator for SourceAnnotator<'a> {
    fn annotate(&mut self, data: &[u8]) -> Result<Annotation, String> {
        let key = derive_hash(self.hash, data);
        match gethostname::gethostname().to_str() {
            Some(host) => {
                let mut annotation = Annotation::new(&key, self.hash, host, self.kind, true);
                let signature = sign_annotation(&self.sign, &annotation)?;
                annotation.with_signature(&signature);
                Ok(annotation)
            },
            None => Err(format!("could not retrieve host name"))
        }
    }
}


#[cfg(test)]
mod source_tests {
    use crate::{config, providers::sign_provider::get_priv_key};
    use crate::annotations::{Annotator, constants, SourceAnnotator};
    use crate::config::Signable;

    #[test]
    fn valid_and_invalid_source_annotator() {
        let config_file = std::fs::read("resources/test_config.json").unwrap();
        let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

        let mut config2 = config.clone();
        config2.hash.hash_type = constants::HashType("Not a known hash type");

        let data = String::from("Some random data");
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);

        let signable = Signable::new(data, sig);
        let serialised = serde_json::to_vec(&signable).unwrap();

        let mut source_annotator_1 = SourceAnnotator::new(&config);
        let mut source_annotator_2 = SourceAnnotator::new(&config2);
        let valid_annotation = source_annotator_1.annotate(&serialised).unwrap();
        let invalid_annotation = source_annotator_2.annotate(&serialised).unwrap();

        assert!(valid_annotation.validate());
        assert!(!invalid_annotation.validate());
    }


    #[test]
    fn make_source_annotation() {
        let config_file = std::fs::read("resources/test_config.json").unwrap();
        let config: config::SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();

        let data = String::from("Some random data");
        let priv_key_file = std::fs::read(config.signature.private_key_info.path).unwrap();
        let priv_key_string = String::from_utf8(priv_key_file).unwrap();
        let priv_key = get_priv_key(&priv_key_string).unwrap();
        let sig = priv_key.sign(data.as_bytes());

        let signable = Signable::new(data, hex::encode(sig.to_bytes()));
        let serialised = serde_json::to_vec(&signable).unwrap();

        let mut source_annotator = SourceAnnotator::new(&config);
        let annotation = source_annotator.annotate(&serialised).unwrap();

        assert!(annotation.validate());
        assert_eq!(annotation.kind, constants::ANNOTATION_SOURCE);
        assert_eq!(annotation.host, gethostname::gethostname().to_str().unwrap());
        assert_eq!(annotation.hash, config.hash.hash_type);
        assert!(annotation.is_satisfied)
    }
}