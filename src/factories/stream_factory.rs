use alvarium_annotator::Publisher;

use crate::config::{StreamConfig, StreamInfo};
use crate::providers::stream_provider::{IotaPublisher, MqttPublisher, PublisherWrap};


pub async fn new_stream_provider(cfg: StreamInfo) -> Result<PublisherWrap, String> {
    match cfg.config {
        StreamConfig::IotaStreams(_) => {
            let publisher = IotaPublisher::new(&cfg).await?;
            Ok(PublisherWrap::Iota(publisher))
        }
        StreamConfig::MQTT(_) => {
            let publisher = MqttPublisher::new(&cfg).await?;
            Ok(PublisherWrap::Mqtt(publisher))
        }
    }
}