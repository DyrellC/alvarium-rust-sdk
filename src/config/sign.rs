use crate::annotations::constants::KeyAlgorithm;
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureInfo<'a> {
    #[serde(borrow)]
    #[serde(rename="public")]
    pub(crate) public_key_info: KeyInfo<'a>,
    #[serde(rename="private")]
    pub(crate) private_key_info: KeyInfo<'a>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyInfo<'a> {
    #[serde(rename="type")]
    pub(crate) key_type: KeyAlgorithm<'a>,
    pub(crate) path: &'a str
}
