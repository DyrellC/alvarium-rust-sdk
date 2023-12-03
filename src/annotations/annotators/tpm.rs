use crate::annotations::{
    Annotation,
    Annotator,
    constants,
};
use crate::config;
use alvarium_annotator::{derive_hash, serialise_and_sign};


#[cfg(unix)]
use std::os::linux::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use crate::factories::{new_hash_provider, new_signature_provider};
use crate::providers::sign_provider::SignatureProviderWrap;

const UNIX_TPM_PATH: &str = "/dev/tpm0"; // Adjust the path as needed

pub struct TpmAnnotator {
    hash: constants::HashType,
    kind: constants::AnnotationType,
    sign: SignatureProviderWrap,
}

impl TpmAnnotator {
    pub fn new(cfg: &config::SdkInfo) -> Result<impl Annotator, String> {
        Ok(TpmAnnotator {
            hash: cfg.hash.hash_type.clone(),
            kind: constants::ANNOTATION_TPM.clone(),
            sign: new_signature_provider(&cfg.signature)?,
        })
    }

    #[cfg(windows)]
    fn check_tpm_presence_windows() -> bool {
        let output = std::process::Command::new("tpmtool")
            .arg("getdeviceinformation")
            .output();

        match output {
            Ok(output) => {
                // Check if the tpmtool command executed successfully and contains "TPM Present"
                output.status.success() && String::from_utf8_lossy(&output.stdout).contains("TPM Present: Yes")
            }
            Err(_) => false,
        }
    }

    #[cfg(unix)]
    fn check_tpm_presence_unix(&self) -> bool {
        match std::fs::metadata(UNIX_TPM_PATH) {
            Ok(metadata) => {
                let file_type = metadata.st_mode() & libc::S_IFMT;
                file_type == libc::S_IFCHR || file_type == libc::S_IFSOCK
            },
            Err(_) => false,
        }
    }


}

impl Annotator for TpmAnnotator {
    fn annotate(&mut self, data: &[u8]) -> Result<Annotation, String> {
        let hasher = new_hash_provider(&self.hash)?;
        let key = derive_hash(hasher, data);
        match gethostname::gethostname().to_str() {
            Some(host) => {
                #[cfg(unix)]
                let is_satisfied = self.check_tpm_presence_unix();
                #[cfg(windows)]
                let is_satisfied = self.check_tpm_presence_windows();

                let mut annotation = Annotation::new(&key, self.hash.clone(), host, self.kind.clone(), is_satisfied);
                let signature = serialise_and_sign(&self.sign, &annotation).map_err(|e| e.to_string())?;
                annotation.with_signature(&signature);
                Ok(annotation)
            },
            None => Err(format!("could not retrieve host name"))
        }
    }
}


#[cfg(test)]
mod tpm_tests {
    use crate::{config, providers::sign_provider::get_priv_key};
    use crate::annotations::{Annotator, constants, TpmAnnotator};
    use crate::config::Signable;
    #[cfg(unix)]
    use super::UNIX_TPM_PATH;

    #[test]
    fn valid_and_invalid_tpm_annotator() {
        let config: config::SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();

        let mut config2 = config.clone();
        config2.hash.hash_type = constants::HashType("Not a known hash type".to_string());

        let data = String::from("Some random data");
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);

        let signable = Signable::new(data, sig);
        let serialised = serde_json::to_vec(&signable).unwrap();

        let mut tpm_annotator_1 = TpmAnnotator::new(&config).unwrap();
        let mut tpm_annotator_2 = TpmAnnotator::new(&config2).unwrap();

        let valid_annotation = tpm_annotator_1.annotate(&serialised).unwrap();
        let invalid_annotation = tpm_annotator_2.annotate(&serialised);

        assert!(valid_annotation.validate_base());
        assert!(invalid_annotation.is_err());
    }


    #[test]
    fn make_tpm_annotation() {
        let config: config::SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();

        let data = String::from("Some random data");
        let priv_key_file = std::fs::read(&config.signature.private_key_info.path).unwrap();
        let priv_key_string = String::from_utf8(priv_key_file).unwrap();
        let priv_key = get_priv_key(&priv_key_string).unwrap();
        let sig = priv_key.sign(data.as_bytes());

        let signable = Signable::new(data, hex::encode(sig.to_bytes()));
        let serialised = serde_json::to_vec(&signable).unwrap();

        let mut tpm_annotator = TpmAnnotator::new(&config).unwrap();
        let annotation = tpm_annotator.annotate(&serialised).unwrap();

        assert!(annotation.validate_base());
        assert_eq!(annotation.kind, *constants::ANNOTATION_TPM);
        assert_eq!(annotation.host, gethostname::gethostname().to_str().unwrap());
        assert_eq!(annotation.hash, config.hash.hash_type);

        #[cfg(unix)]
        let should_be_satisfied = std::fs::metadata(UNIX_TPM_PATH).is_ok();
        #[cfg(windows)]
        let should_be_satisfied = {
            let output = std::process::Command::new("tpmtool")
                .arg("getdeviceinformation")
                .output();
            match output {
                Ok(output) => output.status.success() && String::from_utf8_lossy(&output.stdout).contains("TPM Present: Yes"),
                Err(_) => false
            }
        };

        assert_eq!(annotation.is_satisfied, should_be_satisfied);
    }
}