use serde::{Serialize, Deserialize};
use crate::config::UrlInfo;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct IotaStreamsConfig {
    pub provider: UrlInfo,
    #[serde(rename="tangle")]
    pub tangle_node: UrlInfo,
    pub encoding: String,
    pub topic: String,
}

