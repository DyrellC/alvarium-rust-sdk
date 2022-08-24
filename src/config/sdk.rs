use serde::{Serialize, Deserialize};
use crate::config::{HashInfo, SignatureInfo};
use crate::constants::AnnotationType;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SdkInfo<'a> {
    #[serde(borrow)]
    pub annotators: Vec<AnnotationType<'a>>,
    pub hash: HashInfo<'a>,
    pub signature: SignatureInfo<'a>,

}