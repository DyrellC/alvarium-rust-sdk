use crate::config::{MqttStreamConfig, StreamConfig, StreamInfo};
use rumqttc::{AsyncClient, ConnectionError, EventLoop, MqttOptions, QoS};
use alvarium_annotator::{MessageWrapper, Publisher};

pub struct MqttPublisher {
    cfg: MqttStreamConfig,
    client: AsyncClient,
    connection: EventLoop
}


#[async_trait::async_trait]
impl Publisher for MqttPublisher {
    type StreamConfig = StreamInfo;
    async fn new(cfg: &StreamInfo) -> Result<Self, String> {
        match &cfg.config {
            StreamConfig::MQTT(cfg)=> {
                let (client, connection) = setup_client(&cfg);
                Ok(MqttPublisher {
                    cfg: cfg.clone(),
                    client,
                    connection
                })
            }
            _ => Err("not an mqtt stream configuration".to_string())
        }
    }

    async fn close(&mut self) -> Result<(), String> {
        for topic in &self.cfg.topics {
            self.client.unsubscribe(topic).await.map_err(|e| e.to_string())?;
        }
        self.client.disconnect().await.map_err(|e| e.to_string())
    }

    async fn connect(&mut self) -> Result<(), String> {
        self.reconnect().await
    }

    async fn reconnect(&mut self) -> Result<(), String> {
        println!("Polling connection");
        if let Ok(_) = self.connection.poll().await {
            return Ok(())
        }

        println!("Making new client");
        let (client, mut connection) = setup_client(&self.cfg);
        println!("Polling for error in reconnection");
        if let Err(ConnectionError::ConnectionRefused(e)) = connection.poll().await {
            println!("Connection Error: {:?}", e);
            return Err("connection is refused".to_string())
        }

        self.client = client;
        self.connection = connection;

        let qos = qos(self.cfg.qos);
        for topic in &self.cfg.topics {
            println!("Subscribing to topic: {}", topic);
            self.client.subscribe(topic, qos).await.map_err(|e| e.to_string())?
        }
        Ok(())
    }

    async fn publish(&mut self, msg: MessageWrapper<'_>) -> Result<(), String> {
        println!("Reconnect");
        self.reconnect().await?;
        let msg_str = serde_json::to_string(&msg).map_err(|e| e.to_string())?;
        let bytes = serde_json::to_vec(&msg).map_err(|e| e.to_string())?;
        for topic in &self.cfg.topics {
            println!("Posting {} to mqtt stream at topic {}", msg_str, topic);
            self.client.publish(topic, qos(self.cfg.qos), true, bytes.clone())
                .await.map_err(|e| e.to_string())?
        }

        Ok(())
    }
}

fn setup_client(cfg: &MqttStreamConfig) -> (AsyncClient, EventLoop) {
    let mut mqtt_options = MqttOptions::new(&cfg.client_id, &cfg.provider.host, cfg.provider.port as u16);
    mqtt_options.set_keep_alive(tokio::time::Duration::from_secs(cfg.keep_alive as u64));
    mqtt_options.set_credentials(&cfg.user, cfg.password());
    mqtt_options.set_clean_session(cfg.cleanness);

    AsyncClient::new(mqtt_options, cfg.cap)
}

fn qos(qos: u8) -> QoS {
    match qos {
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => QoS::AtMostOnce,
    }
}


#[cfg(test)]
mod mqtt_tests {
    use alvarium_annotator::{Annotator, AnnotationList, MessageWrapper, Publisher};
    use crate::annotations::PkiAnnotator;
    use crate::config::{SdkInfo, Signable, StreamInfo};
    use crate::providers::stream_provider::MqttPublisher;

    #[tokio::test]
    async fn new_mqtt_provider() {
        let stream_config_bytes = std::fs::read("resources/mqtt_stream_config.json").unwrap();
        let stream_info: StreamInfo = serde_json::from_slice(stream_config_bytes.as_slice()).unwrap();
        let mut publisher = MqttPublisher::new(&stream_info).await.unwrap();
        publisher.close().await.unwrap();
    }

    #[tokio::test]
    async fn mqtt_provider_publish() {
        let sdk_info: SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        let mqtt_stream_info: StreamInfo = serde_json::from_slice(crate::MQTT_TEST_CONFIG_BYTES.as_slice()).unwrap();

        let mut publisher = MqttPublisher::new(&mqtt_stream_info).await.unwrap();
        publisher.connect().await.unwrap();

        let raw_data_msg = "A packet to send to subscribers".to_string();
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);
        let signable = Signable::new(raw_data_msg, sig);

        let mut list = AnnotationList { items: vec![] };
        let mut pki_annotator = PkiAnnotator::new(&sdk_info).unwrap();
        list.items.push(
            pki_annotator.annotate(
                &serde_json::to_vec(&signable).unwrap()
            ).unwrap()
        );

        let data = MessageWrapper {
            action: crate::annotations::constants::ACTION_CREATE,
            message_type: std::any::type_name::<AnnotationList>(),
            content: &base64::encode(&serde_json::to_vec(&list).unwrap()),
        };

        println!("Publishing...");
        publisher.publish(data).await.unwrap();
        publisher.close().await.unwrap();
    }
}