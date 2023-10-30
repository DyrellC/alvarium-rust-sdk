use serde::{Serialize, Deserialize};
use crate::config::{HashInfo, SignatureInfo, StreamInfo};
use crate::annotations::constants::AnnotationType;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SdkInfo<'a> {
    #[serde(borrow)]
    pub annotators: Vec<AnnotationType<'a>>,
    pub hash: HashInfo<'a>,
    pub signature: SignatureInfo<'a>,
    pub stream: StreamInfo<'a>,
}