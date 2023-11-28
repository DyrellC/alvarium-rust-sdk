mod iota;
mod mqtt;

pub use iota::IotaPublisher;
pub use mqtt::MqttPublisher;


// TODO: Implement publisher for enum
pub enum PublisherWrap {
    Iota(IotaPublisher),
    Mqtt(MqttPublisher),
}
