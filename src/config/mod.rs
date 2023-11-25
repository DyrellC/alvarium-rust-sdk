mod hash;
mod sdk;
mod sign;
mod stream;
mod tls;

pub use hash::*;
pub use sdk::*;
pub use sign::*;
pub use stream::*;
pub use tls::*;



#[cfg(test)]
mod make_config_tests {
    use alvarium_annotator::constants::{KeyAlgorithm, StreamType};
    use super::{SdkInfo, StreamInfo, IotaStreamsConfig, MqttStreamConfig};
    #[test]
    fn new_config() {
        let config: SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        assert!(config.hash.hash_type.validate());
        assert!(KeyAlgorithm(&config.signature.private_key_info.key_type).validate());
        assert!(config.annotators[0].validate());
    }

    #[test]
    fn iota_streams_config() {
        let config: StreamInfo = serde_json::from_slice(crate::IOTA_TEST_CONFIG_BYTES.as_slice()).unwrap();
        let _is_config: IotaStreamsConfig;
        assert!(StreamType(&config.stream_type).validate());
        assert!(matches!(config.config, _is_config));
    }

    #[test]
    fn mqtt_stream_config() {
        let config: StreamInfo = serde_json::from_slice(crate::MQTT_TEST_CONFIG_BYTES.as_slice()).unwrap();
        let _mqtt_config: MqttStreamConfig;
        assert!(StreamType(&config.stream_type).validate());
        assert!(matches!(config.config, _mqtt_config));
    }
}
