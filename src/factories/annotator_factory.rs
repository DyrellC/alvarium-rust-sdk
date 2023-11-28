use alvarium_annotator::{
    Annotator,
    constants
};
use crate::annotations::{PkiAnnotator, SourceAnnotator, TlsAnnotator, TpmAnnotator};
use crate::config::SdkInfo;


pub fn new_annotator<'a>(kind: constants::AnnotationType, cfg: SdkInfo) -> Result<Box<dyn Annotator + 'a>, String> {
    match kind {
        constants::ANNOTATION_SOURCE => Ok(Box::new(SourceAnnotator::new(&cfg).map_err(|e| e.to_string())?)),
        constants::ANNOTATION_PKI => Ok(Box::new(PkiAnnotator::new(&cfg).map_err(|e| e.to_string())?)),
        constants::ANNOTATION_TLS => Ok(Box::new(TlsAnnotator::new(&cfg).map_err(|e| e.to_string())?)),
        constants::ANNOTATION_TPM => Ok(Box::new(TpmAnnotator::new(&cfg).map_err(|e| e.to_string())?)),
        _ => Err(format!("not a pre known alvarium annotator {}, please build separate", kind.kind()))
    }
}