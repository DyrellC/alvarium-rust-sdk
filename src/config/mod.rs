mod hash;
mod sdk;
mod sign;
mod stream;

pub use hash::*;
pub use sdk::*;
pub use sign::*;
pub use stream::*;



#[test]
fn new_config() {
    let config_file = std::fs::read("resources/test_config.json").unwrap();
    let config: SdkInfo = serde_json::from_slice(config_file.as_slice()).unwrap();
    assert!(config.hash.hash_type.validate());
    assert!(config.signature.private_key_info.key_type.validate());
    assert!(config.annotators[0].validate());
}

#[test]
fn iota_streams_config() {
    let config_file = std::fs::read("resources/iota_streams_config.json").unwrap();
    let config: StreamInfo = serde_json::from_slice(config_file.as_slice()).unwrap();
    let _is_config: IotaStreamsConfig;
    assert!(config.stream_type.validate());
    assert!(matches!(config.config, _is_config));
}

#[test]
fn mqtt_stream_config() {
    let config_file = std::fs::read("resources/mqtt_stream_config.json").unwrap();
    let config: StreamInfo = serde_json::from_slice(config_file.as_slice()).unwrap();
    let _mqtt_config: MqttStreamConfig;
    assert!(config.stream_type.validate());
    assert!(matches!(config.config, _mqtt_config));
}
