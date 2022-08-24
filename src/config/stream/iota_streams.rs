use serde::{Serialize, Deserialize};
use crate::config::UrlInfo;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct IotaStreamsConfig<'a> {
    #[serde(borrow)]
    provider: UrlInfo<'a>,
    #[serde(rename="tangle")]
    tangle_node: UrlInfo<'a>,
    encoding: &'a str
}

