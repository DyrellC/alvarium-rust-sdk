use crate::config::{IotaStreamsConfig, SdkInfo, Signable, StreamConfig, StreamInfo};
use crate::providers::stream_provider::{MessageWrapper, Publisher};
use streams::{Address, User, transport::utangle::Client, id::{Ed25519, Identity, Identifier}, Message};
use core::str::FromStr;
use std::any::Any;
use std::thread::sleep;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use futures::TryStreamExt;
use crate::annotations::{AnnotationList, Annotator, constants::IOTA_STREAM};
use crate::annotations::PkiAnnotator;

const MAX_RETRIES: u8 = 100;


pub struct IotaPublisher<'a> {
    cfg: IotaStreamsConfig<'a>,
    subscriber: User<Client>,
    identifier: Identifier,
}


impl IotaPublisher<'_> {
    async fn await_keyload(&mut self) -> Result<(), String> {
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
}

#[async_trait::async_trait(?Send)]
impl<'a> Publisher<'a> for IotaPublisher<'a> {
    async fn new(cfg: StreamInfo<'a>) -> Result<IotaPublisher<'a>, String> {
        match cfg.config {
            StreamConfig::IotaStreams(cfg) => {
                let client = Client::new(&cfg.tangle_node.uri());
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
                        cfg,
                        subscriber,
                        identifier,
                    }
                )
            },
            _ => Err("not an Iota Streams configuration".to_string())
        }
    }

    async fn close(&self) -> Result<(), String> {
        // No need to disconnect from stream or drop anything
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
            msgid: String,
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
            msgid: subscription.address().relative().to_string(),
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
        let bytes = serde_json::to_vec(&msg)
            .map_err(|e| e.to_string())?;

        let packet = self.subscriber.send_signed_packet(self.cfg.topic, vec![],bytes).await
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



#[tokio::test]
async fn new_iota_streams_provider() {
    let sdk_config_bytes = std::fs::read("resources/test_config.json").unwrap();
    let sdk_info: SdkInfo = serde_json::from_slice(sdk_config_bytes.as_slice()).unwrap();
    let mut author = IotaPublisher::new(sdk_info.stream.clone()).await.unwrap();
    author.connect().await.unwrap();

    let data = "A packet to send to the author".to_string();
    let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);
    let signable = Signable::new(data, sig);

    let mut list = AnnotationList { items: vec![] };
    let pki_annotator = PkiAnnotator::new(sdk_info);
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
    author.publish(data).await.unwrap()
}


