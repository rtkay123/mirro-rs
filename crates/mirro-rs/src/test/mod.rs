use crate::{cli::ArgConfig, config::Configuration, direct::begin};

#[tokio::test]
async fn sample_bin() {
    let config_str = include_str!("../../../../examples/mirro-rs.toml");
    let configuration: ArgConfig = toml::from_str(config_str).unwrap();
    let config = Configuration::from(configuration);
    let result = begin(config).await;
    dbg!(&result);
    assert!(result.is_ok());
}
