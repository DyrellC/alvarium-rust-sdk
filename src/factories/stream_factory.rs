use alvarium_annotator::Annotator;
use crate::providers::{
    stream_provider::Publisher,

};

use crate::config::{StreamConfig, StreamInfo};
use crate::providers::stream_provider::{IotaPublisher, MqttPublisher};


pub enum PublisherWrap<'a> {
    Iota(IotaPublisher<'a>),
    Mqtt(MqttPublisher<'a>),
}

pub async fn new_stream_provider<'a>(cfg: StreamInfo<'a>) -> Result<PublisherWrap, String> {
    match cfg.config {
        StreamConfig::IotaStreams(_) => {
            let publisher = IotaPublisher::new(cfg).await?;
            Ok(PublisherWrap::Iota(publisher))
        }
        StreamConfig::MQTT(_) => {
            let publisher = MqttPublisher::new(cfg).await?;
            Ok(PublisherWrap::Mqtt(publisher))
        }
    }
}