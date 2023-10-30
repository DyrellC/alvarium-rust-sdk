use serde::{Serialize, Deserialize};
use crate::config::UrlInfo;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct IotaStreamsConfig<'a> {
    #[serde(borrow)]
    pub provider: UrlInfo<'a>,
    #[serde(rename="tangle")]
    pub tangle_node: UrlInfo<'a>,
    pub encoding: &'a str,
    pub topic: &'a str,
}

