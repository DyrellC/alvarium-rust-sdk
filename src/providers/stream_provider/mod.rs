mod iota;
mod mqtt;

pub use iota::IotaPublisher;
pub use mqtt::MqttPublisher;


use serde::{Serialize, Deserialize};
use crate::annotations::constants::SdkAction;
use crate::config::StreamInfo;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageWrapper<'a>{
    #[serde(borrow)]
    pub action: SdkAction<'a>,
    #[serde(rename="messageType")]
    pub message_type: &'a str,
    pub content: &'a str,
}


#[async_trait::async_trait(?Send)]
pub trait Publisher<'a>: Sized {
    async fn new(cfg: StreamInfo<'a>) -> Result<Self, String>;
    async fn close(&mut self) -> Result<(), String>;
    async fn connect(&mut self) -> Result<(), String>;

    async fn reconnect(&mut self) -> Result<(), String>;
    async fn publish(&mut self, msg: MessageWrapper<'_>) -> Result<(), String>;
}
