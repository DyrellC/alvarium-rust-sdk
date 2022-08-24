use std::time::SystemTime;
use crate::constants::{self, AnnotationType, HashType};
use serde::{Serialize, Deserialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Annotation<'a> {
    pub id: String,
    pub key: String,
    #[serde(borrow)]
    pub hash: HashType<'a>,
    pub host: String,
    #[serde(borrow)]
    pub kind: AnnotationType<'a>,
    pub signature: String,
    #[serde(rename = "isSatisfied")]
    pub is_satisfied: bool,
    pub timestamp: SystemTime,
}

pub struct AnnotationList<'a>(Vec<Annotation<'a>>);

impl<'a> Annotation<'a> {
    pub fn new(key: &str, hash: HashType<'a>, host: &str, kind: AnnotationType<'a>, is_satisfied: bool) -> Self {
        Annotation {
            id: ulid::Ulid::new().to_string(),
            key: key.to_string(),
            hash,
            host: host.to_string(),
            kind,
            signature: String::new(),
            is_satisfied,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn with_signature(&mut self, signature: &str) {
        self.signature = signature.to_string()
    }

    pub fn validate(&self) -> bool {
        self.hash.validate() && self.kind.validate()
    }
}

#[test]
fn new_annotation() {
    let key = "The hash of the contents";
    let hash = constants::SHA256_HASH;
    let host = "Host Device";
    let kind = constants::ANNOTATION_SOURCE;
    let satisfied = true;
    let signature = "Validation Signature";

    let mut annotation = Annotation::new(key, hash, host, kind, satisfied);
    annotation.with_signature(signature);

    assert_eq!(&annotation.key, key);
    assert_eq!(annotation.hash, hash);
    assert_eq!(&annotation.host, host);
    assert_eq!(annotation.kind, kind);
    assert_eq!(annotation.is_satisfied, satisfied);
    assert_eq!(&annotation.signature, signature);
}

#[test]
fn validate_good_annotation() {
    let annotation = mock_annotation();
    assert_eq!(annotation.validate(), true);
}

#[test]
fn validate_bad_annotation() {
    let mut annotation = mock_annotation();
    annotation.hash = HashType("An unrecognized Hash Type");
    assert_eq!(annotation.validate(), false);
}

#[test]
fn serde_annotation() {
    let annotation = mock_annotation();
    let serialized = serde_json::to_string(&annotation).unwrap();
    let new_annotation: Annotation = serde_json::from_str(&serialized).unwrap();
    assert_eq!(new_annotation, annotation);
}

pub(crate) fn mock_annotation<'a>() -> Annotation<'a> {
    let key = "The hash of the contents";
    let hash = constants::SHA256_HASH;
    let host = "Host Device";
    let kind = constants::ANNOTATION_SOURCE;
    let satisfied = true;

    Annotation::new(key, hash, host, kind, satisfied)
}
