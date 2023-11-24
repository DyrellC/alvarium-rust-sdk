use alvarium_annotator::{
    Annotator,
    constants
};
use alvarium_annotator::constants::AnnotationType;
use crate::annotations::{PkiAnnotator, SourceAnnotator, TlsAnnotator, TpmAnnotator};
use crate::config::SdkInfo;


pub fn new_annotator<'a>(kind: AnnotationType, cfg: SdkInfo<'a>) -> Result<Box<dyn Annotator + 'a>, String> {
    match kind {
        constants::ANNOTATION_SOURCE => Ok(Box::new(SourceAnnotator::new(&cfg))),
        constants::ANNOTATION_PKI => Ok(Box::new(PkiAnnotator::new(&cfg))),
        constants::ANNOTATION_TLS => Ok(Box::new(TlsAnnotator::new(&cfg))),
        constants::ANNOTATION_TPM => Ok(Box::new(TpmAnnotator::new(&cfg))),
        _ => Err(format!("not a pre known alvarium annotator {}, please build separate", kind.kind()))
    }
}