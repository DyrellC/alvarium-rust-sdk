use crate::annotations::{
    Annotation,
    Annotator,
    constants,
};
use crate::config;
use alvarium_annotator::{derive_hash, serialise_and_sign};
use crate::factories::{new_hash_provider, new_signature_provider};
use crate::providers::sign_provider::SignatureProviderWrap;

pub struct SourceAnnotator<'a> {
    hash: constants::HashType<'a>,
    kind: constants::AnnotationType<'a>,
    sign: SignatureProviderWrap,
}

impl<'a> SourceAnnotator<'a> {
    pub fn new(cfg: &config::SdkInfo) -> Result<impl Annotator + 'a, String> {
        Ok(SourceAnnotator {
            hash: cfg.hash.hash_type,
            kind: constants::ANNOTATION_SOURCE,
            sign: new_signature_provider(&cfg.signature)?,
        })
    }
}

impl<'a> Annotator for SourceAnnotator<'a> {
    fn annotate(&mut self, data: &[u8]) -> Result<Annotation, String> {
        let hasher = new_hash_provider(&self.hash)?;
        let key = derive_hash(hasher, data);
        match gethostname::gethostname().to_str() {
            Some(host) => {
                let mut annotation = Annotation::new(&key, self.hash, host, self.kind, true);
                let signature = serialise_and_sign(&self.sign, &annotation).map_err(|e| e.to_string())?;
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
        let config: config::SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();

        let mut config2 = config.clone();
        config2.hash.hash_type = constants::HashType("Not a known hash type");

        let data = String::from("Some random data");
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);

        let signable = Signable::new(data, sig);
        let serialised = serde_json::to_vec(&signable).unwrap();

        let mut source_annotator_1 = SourceAnnotator::new(&config).unwrap();
        let mut source_annotator_2 = SourceAnnotator::new(&config2).unwrap();
        let valid_annotation = source_annotator_1.annotate(&serialised).unwrap();
        let invalid_annotation = source_annotator_2.annotate(&serialised);

        assert!(valid_annotation.validate());
        assert!(invalid_annotation.is_err());
    }


    #[test]
    fn make_source_annotation() {
        let config: config::SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();

        let data = String::from("Some random data");
        let priv_key_file = std::fs::read(&config.signature.private_key_info.path).unwrap();
        let priv_key_string = String::from_utf8(priv_key_file).unwrap();
        let priv_key = get_priv_key(&priv_key_string).unwrap();
        let sig = priv_key.sign(data.as_bytes());

        let signable = Signable::new(data, hex::encode(sig.to_bytes()));
        let serialised = serde_json::to_vec(&signable).unwrap();

        let mut source_annotator = SourceAnnotator::new(&config).unwrap();
        let annotation = source_annotator.annotate(&serialised).unwrap();

        assert!(annotation.validate());
        assert_eq!(annotation.kind, constants::ANNOTATION_SOURCE);
        assert_eq!(annotation.host, gethostname::gethostname().to_str().unwrap());
        assert_eq!(annotation.hash, config.hash.hash_type);
        assert!(annotation.is_satisfied)
    }
}