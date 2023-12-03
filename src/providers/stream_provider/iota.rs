use crate::config::{IotaStreamsConfig, StreamConfig, StreamInfo};
use alvarium_annotator::{MessageWrapper, Publisher};
use streams::{Address, User, transport::utangle::Client, id::{Ed25519, Identifier}, Message};
use core::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use futures::TryStreamExt;

const MAX_RETRIES: u8 = 100;


pub struct IotaPublisher {
    cfg: IotaStreamsConfig,
    subscriber: User<Client>,
    identifier: Identifier,
}


impl IotaPublisher {
    pub(crate) async fn await_keyload(&mut self) -> Result<(), String> {
        let mut i = 0;
        while i < MAX_RETRIES {
            let m = self.subscriber.messages();
            if let Ok(next_messages) = m.try_collect::<Vec<Message>>().await {
                println!("Found messages? {}", !next_messages.is_empty());
                //if let Some(message) = next {
                for message in next_messages {
                    if let Some(keyload) = message.as_keyload() {
                        if keyload.includes_subscriber(&self.identifier) {
                            return Ok(())
                        }
                    }
                }
            }
            sleep(Duration::from_secs(5));
            i += 1;
        }
        Err("Did not find keyload, subscription may not have been processed correctly".to_string())
    }

    pub fn client(&mut self) -> &mut User<Client> {
        &mut self.subscriber
    }

    pub fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

#[async_trait::async_trait]
impl Publisher for IotaPublisher {
    type StreamConfig = StreamInfo;
    async fn new(cfg: &StreamInfo) -> Result<IotaPublisher, String> {
        match &cfg.config {
            StreamConfig::IotaStreams(cfg) => {
                let client = Client::new(cfg.tangle_node.uri());
                let mut seed = [0u8; 64];
                crypto::utils::rand::fill(&mut seed)
                    .map_err(|e| e.to_string())?;

                let subscriber = User::builder()
                    .with_transport(client)
                    .with_identity(Ed25519::from_seed(seed))
                    .build();

                let identifier = subscriber.identifier().unwrap().clone();
                Ok(
                    IotaPublisher {
                        cfg: cfg.clone(),
                        subscriber,
                        identifier,
                    }
                )
            },
            _ => Err("not an Iota Streams configuration".to_string())
        }
    }

    async fn close(&mut self) -> Result<(), String> {
        // No need to disconnect from stream or drop anything
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<(), String> {
        // No need to reconnect as disconnection does not occur
        Ok(())
    }
    async fn connect(&mut self) -> Result<(), String> {
        let announcement = get_announcement_id(&self.cfg.provider.uri()).await?;
        println!("Got announcement id");
        let announcement_address = Address::from_str(&announcement)
            .map_err(|e| e.to_string())?;
        println!("announcement address: {}", announcement_address.to_string());

        println!("Fetching message");
        self.subscriber.receive_message(announcement_address)
            .await
            .map_err(|e| e.to_string())?;

        println!("Starting subscription");
        let subscription = self.subscriber.subscribe()
            .await
            .map_err(|e| e.to_string())?;

        #[derive(Serialize, Deserialize)]
        struct SubscriptionRequest {
            address: String,
            identifier: String,
            #[serde(rename="idType")]
            id_type: u8,
            topic: String,
        }

        #[cfg(feature = "did-streams")]
        let id_type = 1;
        #[cfg(not(feature = "did-streams"))]
        let id_type = 0;

        let body = SubscriptionRequest {
            address: subscription.address().to_string(),
            identifier: self.identifier.to_string(),
            id_type,
            topic: self.cfg.topic.to_string(),
        };

        let body_bytes = serde_json::to_vec(&body)
            .map_err(|e| e.to_string())?;

        println!("Sending subscription request to console");
        send_subscription_request(&self.cfg.provider.uri(), body_bytes).await?;
        self.await_keyload().await?;
        Ok(())
    }

    async fn publish(&mut self, msg: MessageWrapper<'_>) -> Result<(), String> {
        println!("Message being published: {:?}", msg);
        let bytes = serde_json::to_vec(&msg)
            .map_err(|e| e.to_string())?;

        let packet = self.subscriber.message()
            .with_payload(bytes)
            .with_topic(self.cfg.topic.as_str())
            .signed()
            .send()
            .await
            .map_err(|e| e.to_string())?;

        println!("Published new message: {}", packet.address());
        Ok(())
    }
}

async fn get_announcement_id(uri: &str) -> Result<String, String> {
    #[derive(Serialize, Deserialize)]
    struct AnnouncementResponse {
        announcement_id: String
    }

    let client = reqwest::Client::new();
    let response = client.get(uri.to_owned() + "/get_announcement_id")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    let announcement: AnnouncementResponse = serde_json::from_slice(&response)
        .map_err(|e| e.to_string())?;
    Ok(announcement.announcement_id)
}


async fn send_subscription_request(uri: &str, body: Vec<u8>) -> Result<(), String> {
    reqwest::Client::new()
        .post(uri.to_owned() + "/subscribe")
        .body(body)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}



#[cfg(test)]
mod iota_test {
    use crate::{
        annotations::{AnnotationList, Annotator, PkiAnnotator},
        config::{SdkInfo, StreamConfig, Signable}
    };
    use streams::id::{PermissionDuration, Permissioned};
    use super::{Client, IotaPublisher, Ed25519, Publisher, MessageWrapper, User};
    const BASE_TOPIC: &'static str = "Base Topic";

    #[tokio::test]
    async fn new_iota_streams_provider() {
        let sdk_info: SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        let _annotator = mock_provider(sdk_info).await;
    }

    #[tokio::test]
    async fn streams_provider_publish() {
        let sdk_info: SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        let mut publisher = mock_provider(sdk_info.clone()).await;

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
            action: crate::annotations::constants::ACTION_CREATE.clone(),
            message_type: std::any::type_name::<AnnotationList>(),
            content: &base64::encode(&serde_json::to_vec(&list).unwrap()),
        };

        println!("Publishing...");
        publisher.publish(data).await.unwrap()
    }

    async fn mock_provider(sdk_info: SdkInfo) -> IotaPublisher {
        if let StreamConfig::IotaStreams(config) = &sdk_info.stream.config {
            let client: Client = Client::new(&config.tangle_node.uri());
            let mut seed = [0u8; 64];
            crypto::utils::rand::fill(&mut seed).unwrap();

            // Create an author to attach to
            let mut streams_author = User::builder()
                .with_transport(client)
                .with_identity(Ed25519::from_seed(seed))
                .build();
            let announcement = streams_author.create_stream(BASE_TOPIC).await.unwrap();

            let mut annotator = IotaPublisher::new(&sdk_info.stream).await.unwrap();
            // To test connect, there needs to be a running provider (oracle) so we'll manually test
            // this part
            //annotator.connect().await.unwrap();

            // Annotator will receive the announcement and send a subscription, in connect() it would
            // send a subscription request to the oracle, for now we assume permission for connection
            annotator.client().receive_message(announcement.address()).await.unwrap();
            let sub_message = annotator.client().subscribe().await.unwrap();

            // Streams author accepts the subscription and dedicates a new branch specifically for
            // the annotator
            streams_author.receive_message(sub_message.address()).await.unwrap();
            streams_author.new_branch(BASE_TOPIC, config.topic.as_str()).await.unwrap();
            streams_author.send_keyload(
                config.topic.as_str(),
                vec![Permissioned::ReadWrite(annotator.identifier().clone(), PermissionDuration::Perpetual)],
                vec![]
            )
                .await
                .unwrap();

            annotator.await_keyload().await.unwrap();
            return annotator
        } else {
            panic!("Test configuration is not correct, should be IotaStreams config")
        }
    }
}



