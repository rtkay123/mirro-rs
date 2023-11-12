use crate::{config::Configuration, direct::begin};

#[tokio::test]
async fn sample_bin() {
    let configuration = Configuration::default();
    let result = begin(configuration).await;
    assert!(result.is_ok());
}
