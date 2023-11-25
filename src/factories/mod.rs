mod stream_factory;
mod annotator_factory;

pub use stream_factory::*;
pub use annotator_factory::*;



#[cfg(test)]
mod factory_tests {
    use crate::config::SdkInfo;
    use crate::factories::{new_annotator, new_stream_provider};

    #[tokio::test]
    async fn provider_factory() {
        let sdk_info: SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        let _provider = new_stream_provider(sdk_info.stream).await.unwrap();
    }

    #[tokio::test]
    async fn annotator_factory() {
        let sdk_info: SdkInfo = serde_json::from_slice(crate::CONFIG_BYTES.as_slice()).unwrap();
        for ann in &sdk_info.annotators {
            let _annotator = new_annotator(*ann, sdk_info.clone()).unwrap();
        }
    }
}