use alvarium_annotator::{
    Annotator,
    constants
};
use crate::annotations::{PkiAnnotator, SourceAnnotator, TlsAnnotator, TpmAnnotator};
use crate::config::SdkInfo;


pub fn new_annotator(kind: constants::AnnotationType, cfg: SdkInfo) -> Result<Box<dyn Annotator>, String> {
    if !kind.is_base_annotation_type() {
        return Err(format!("not a pre known alvarium annotator {}, please build separate", kind.kind()))
    }

    match kind.kind() {
        "source" => Ok(Box::new(SourceAnnotator::new(&cfg).map_err(|e| e.to_string())?)),
        "pki" => Ok(Box::new(PkiAnnotator::new(&cfg).map_err(|e| e.to_string())?)),
        "tls" => Ok(Box::new(TlsAnnotator::new(&cfg).map_err(|e| e.to_string())?)),
        "tpm" => Ok(Box::new(TpmAnnotator::new(&cfg).map_err(|e| e.to_string())?)),
        _ => Err(format!("not a pre known alvarium annotator {}, please build separate", kind.kind()))
    }
}