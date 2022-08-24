use serde::{Serialize, Deserialize};
use crate::config::UrlInfo;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MqttStreamConfig<'a> {
    #[serde(borrow)]
    #[serde(rename="clientId")]
    client_id: &'a str,
    qos: u8,
    user: &'a str,
    password: &'a str,
    provider: UrlInfo<'a>,
    cleanness: bool,
    topics: Vec<&'a str>
}