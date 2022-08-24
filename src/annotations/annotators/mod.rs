mod utility;
mod pki;
mod source;
mod tls;
mod tpm;

pub use utility::*;
pub use pki::*;
pub use source::*;
pub use tls::*;
pub use tpm::*;

use crate::annotations::Annotation;

pub trait Annotator {
    fn annotate(&self, data: &[u8]) -> Result<Annotation, String>;
}