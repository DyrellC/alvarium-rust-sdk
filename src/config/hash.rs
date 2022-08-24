use crate::constants::HashType;
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HashInfo<'a> {
    #[serde(borrow, rename="type")]
    pub hash_type: HashType<'a>
}

impl HashInfo<'_> {
    fn validate(&self) -> bool {
        self.hash_type.validate()
    }
}