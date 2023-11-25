use crate::annotations::constants::HashType;
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HashInfo {
    #[serde(borrow, rename="type")]
    pub hash_type: HashType<'static>
}

impl HashInfo {
    pub fn validate(&self) -> bool {
        self.hash_type.validate()
    }
}