use std::{path::PathBuf, str::FromStr};

use mirro_rs::{cli::ArgConfig, config::Configuration, direct::begin};

#[tokio::test]
async fn sample_bin() {
    let config_str = include_str!("../../../examples/mirro-rs.toml");
    let configuration: ArgConfig = toml::from_str(config_str).unwrap();
    let mut config = Configuration::from(configuration);
    let mut tmp = PathBuf::from_str(env!("CARGO_TARGET_TMPDIR")).unwrap();
    tmp.push("mirro_rs");
    config.outfile = tmp;
    let result = begin(config).await;
    assert!(result.is_ok());
}
