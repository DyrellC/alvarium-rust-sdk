use serde::{Serialize, Deserialize};
use crate::config::{HashInfo, SignatureInfo, StreamInfo};
use crate::annotations::constants::AnnotationType;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SdkInfo {
    pub annotators: Vec<AnnotationType>,
    pub hash: HashInfo,
    pub signature: SignatureInfo,
    pub stream: StreamInfo,
}