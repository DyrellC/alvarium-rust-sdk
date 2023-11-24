use crate::config::SdkInfo;
use crate::annotations::{Annotator, AnnotationList, constants::ACTION_CREATE};
use crate::providers::stream_provider::{MessageWrapper, Publisher};

pub struct SDK<'a, Pub: Publisher<'a>> {
    annotators: &'a mut [Box<dyn Annotator>],
    pub cfg: SdkInfo<'a>,
    stream: Pub
}

impl<'a, Pub: Publisher<'a>> SDK<'a, Pub> {
    pub async fn new(cfg: SdkInfo<'a>, annotators: &'a mut [Box<dyn Annotator>]) -> Result<SDK<'a, Pub>, String> {
        let mut publisher = Pub::new(cfg.stream.clone()).await?;
        publisher.connect().await?;
        Ok(SDK {
            annotators,
            cfg,
            stream: publisher,
        })
    }

    pub async fn create(&mut self, data: &[u8]) -> Result<(), String> {
        let mut ann_list = AnnotationList::default();
        for annotator in self.annotators.as_mut() {
            ann_list.items.push(annotator.annotate(data)?);
        }

        let ann_bytes = serde_json::to_vec(&ann_list)
            .map_err(|e| e.to_string())?;
        let wrapper = MessageWrapper {
            action: ACTION_CREATE,
            message_type: std::any::type_name::<AnnotationList>(),
            content: &base64::encode(ann_bytes)
        };
        self.stream.publish(wrapper).await
    }

    pub async fn mutate(&mut self, _data: &[u8]) -> Result<(), String> {
        //let src = ANNOTATION_SOURCE;
        // TODO: add new mutation and transit functions
        Ok(())
    }

}


#[cfg(test)]
mod sdk_tests {
    use streams::{
        id::{Ed25519, PermissionDuration, Permissioned},
        transport::utangle::Client,
        User,
    };
    use crate::{annotations::PkiAnnotator, config::{SdkInfo, StreamConfig, Signable}, CONFIG_BYTES, providers::stream_provider::{IotaPublisher, Publisher}};
    use crate::annotations::SourceAnnotator;
    use crate::factories::new_annotator;
    use super::SDK;

    const BASE_TOPIC: &'static str = "Base Topic";

    #[tokio::test]
    async fn sdk_create() {
        // Uses base CONFIG_BYTES pulled from local config file (or the resources/test_config.json
        // if no config file is present)
        let sdk_info: SdkInfo = serde_json::from_slice(CONFIG_BYTES.as_slice()).unwrap();
        let publisher = mock_annotator(sdk_info.clone()).await;

        let mut annotators = Vec::new();
        for ann in &sdk_info.annotators {
            let annotator = new_annotator(ann.clone(), sdk_info.clone());
            annotators.push(new_annotator(ann.clone(), sdk_info.clone()).unwrap())
        }

        // Mocks SDK::new() without Pub::connect()
        let mut sdk = SDK {
            annotators: annotators.as_mut_slice(),
            cfg: sdk_info.clone(),
            stream: publisher,
        };

        let data = "A packet to send to subscribers".to_string();
        let sig = hex::encode([0u8; crypto::signatures::ed25519::SIGNATURE_LENGTH]);
        let signable = Signable::new(data, sig);
        sdk.create(signable.to_bytes().as_slice()).await.unwrap();
    }

    // Mocks Pub::new() with IotaPublisher Annotator
    async fn mock_annotator(sdk_info: SdkInfo<'_>) -> IotaPublisher {
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

            let mut publisher = IotaPublisher::new(sdk_info.stream.clone()).await.unwrap();
            // To test connect, there needs to be a running provider (oracle) so we'll manually test
            // this part
            //annotator.connect().await.unwrap();

            // Annotator will receive the announcement and send a subscription, in connect() it would
            // send a subscription request to the oracle, for now we assume permission for connection
            publisher.client().receive_message(announcement.address()).await.unwrap();
            let sub_message = publisher.client().subscribe().await.unwrap();

            // Streams author accepts the subscription and dedicates a new branch specifically for
            // the annotator
            streams_author.receive_message(sub_message.address()).await.unwrap();
            streams_author.new_branch(BASE_TOPIC, config.topic).await.unwrap();
            streams_author.send_keyload(
                config.topic,
                vec![Permissioned::ReadWrite(publisher.identifier().clone(), PermissionDuration::Perpetual)],
                vec![]
            )
                .await
                .unwrap();

            publisher.await_keyload().await.unwrap();
            return publisher
        } else {
            panic!("Test configuration is not correct, should be IotaStreams config")
        }
    }
}