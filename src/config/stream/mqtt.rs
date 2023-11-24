use serde::{Serialize, Deserialize};
use crate::config::UrlInfo;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MqttStreamConfig<'a> {
    #[serde(borrow)]
    #[serde(rename="clientId")]
    pub client_id: &'a str,
    #[serde(rename="boundedCap")]
    pub cap: usize,
    #[serde(rename="keepAlive")]
    pub keep_alive: u8,
    pub qos: u8,
    pub user: &'a str,
    password: &'a str,
    pub provider: UrlInfo<'a>,
    pub cleanness: bool,
    pub topics: Vec<&'a str>
}

impl MqttStreamConfig<'_> {
    pub(crate) fn password(&self) -> &str {
        self.password
    }
}