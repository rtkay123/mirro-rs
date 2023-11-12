use crate::{config::Configuration, direct::begin};

#[tokio::test]
async fn sample_bin() {
    let configuration = Configuration::default();
    let result = begin(configuration).await;
    dbg!(&result);
    assert!(result.is_ok());
}
