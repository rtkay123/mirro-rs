use crate::{cli::ArgConfig, config::Configuration, direct::begin};

#[tokio::test]
async fn sample_bin() {
    let config_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/mirro-rs.toml");
    let config_str = std::fs::read_to_string(config_path).unwrap();
    let configuration: ArgConfig = toml::from_str(&config_str).unwrap();
    let config = Configuration::from(configuration);
    let result = begin(config).await;
    dbg!(&result);
    assert!(result.is_ok());
}
