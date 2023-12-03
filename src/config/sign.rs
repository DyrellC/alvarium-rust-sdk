use alvarium_annotator::constants::KeyAlgorithm;
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureInfo {
    #[serde(rename="public")]
    pub(crate) public_key_info: KeyInfo,
    #[serde(rename="private")]
    pub(crate) private_key_info: KeyInfo,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyInfo {
    #[serde(rename="type")]
    pub(crate) key_type: KeyAlgorithm,
    pub(crate) path: String
}