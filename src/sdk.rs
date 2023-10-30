use crate::config::SdkInfo;
use crate::annotations::{Annotator, AnnotationList, constants::{ACTION_CREATE, ANNOTATION_SOURCE}};
use crate::providers::stream_provider::{MessageWrapper, Publisher};

pub struct SDK<'a, Ann: Annotator, Pub: Publisher<'a>> {
    annotators: &'a [Ann],
    pub cfg: SdkInfo<'a>,
    stream: Pub
}

impl<'a, Ann: Annotator, Pub: Publisher<'a>> SDK<'a, Ann, Pub> {
    pub async fn new(cfg: SdkInfo<'a>, annotators: &'a [Ann]) -> Result<SDK<'a, Ann, Pub>, String> {
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
        for annotator in self.annotators {
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