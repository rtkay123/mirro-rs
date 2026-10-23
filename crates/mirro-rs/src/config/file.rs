use std::{
    fs::OpenOptions,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use figment::{
    Figment,
    providers::{Format, Serialized, Toml},
};
use tracing::error;

use crate::cli::{ArgConfig, Args, Filters};

pub fn generate_config_file(output: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let output = match output {
        Some(output) => output,
        None => dirs::config_dir()
            .map(|dir| dir.join(env!("CARGO_PKG_NAME")).join("mirro-rs.toml"))
            .ok_or_else(|| anyhow::anyhow!("could not determine the XDG config directory"))?,
    };

    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }

    let config = toml::to_string_pretty(&ArgConfig::default())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)?;
    file.write_all(config.as_bytes())?;

    Ok(output)
}

pub fn read_config_file(
    file: Option<impl AsRef<Path>>,
    cli: Option<ArgConfig>,
) -> (ArgConfig, Option<PathBuf>) {
    let config_file = match file {
        Some(file) => check_file(file.as_ref(), None),
        None => dirs::config_dir().and_then(|dir| get_config(&dir)),
    };
    let config_path = config_file.map(|(_, path)| path).flatten();
    let mut figment = Figment::from(Serialized::defaults(ArgConfig::default()));
    if let Some(path) = config_path.as_ref() {
        figment = figment.merge(Toml::file(path));
    }
    if let Some(cli) = cli {
        figment = figment.merge(Serialized::defaults(cli));
    }

    match figment.extract() {
        Ok(config) => (config, config_path),
        Err(error) => {
            error!("config: {error}");
            (ArgConfig::default(), None)
        }
    }
}

pub fn empty_cli_config() -> ArgConfig {
    ArgConfig {
        general: Args::default(),
        filters: Filters::default(),
        #[cfg(feature = "toml")]
        command: None,
    }
}

fn check_file(file: &Path, backup: Option<&Path>) -> Option<(ArgConfig, Option<PathBuf>)> {
    let contents = match std::fs::read_to_string(file) {
        Ok(contents) => contents,
        Err(error) => {
            if error.kind() != ErrorKind::NotFound {
                error!("config: {} -> {error}", file.display());
            }
            return backup.and_then(|backup| check_file(backup, None));
        }
    };

    let Some(extension) = file.extension().and_then(|extension| extension.to_str()) else {
        error!("config: {} -> missing file extension", file.display());
        return backup.and_then(|backup| check_file(backup, None));
    };

    let parsed = match extension {
        "toml" => toml::from_str::<ArgConfig>(&contents).map_err(|error| error.to_string()),
        _ => Err("unsupported file extension: file must be: toml".to_string()),
    };

    match parsed {
        Ok(config) => Some((config, Some(file.to_path_buf()))),
        Err(error) => {
            error!("config: {} -> {error}", file.display());
            backup.and_then(|backup| check_file(backup, None))
        }
    }
}

fn get_config(dir: &Path) -> Option<(ArgConfig, Option<PathBuf>)> {
    let crate_name = env!("CARGO_PKG_NAME");
    let file_name = format!("{crate_name}.toml");
    let primary = dir.join(crate_name).join(&file_name);
    let backup = dir.join(file_name);
    check_file(&primary, Some(&backup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips_through_toml() {
        let serialized = toml::to_string_pretty(&ArgConfig::default()).unwrap();
        let parsed: ArgConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(toml::to_string_pretty(&parsed).unwrap(), serialized,);
    }

    #[test]
    fn cli_values_override_file_and_file_false_values_override_defaults() {
        let path =
            std::env::temp_dir().join(format!("mirro-rs-figment-test-{}.toml", std::process::id()));
        std::fs::write(
            &path,
            "[general]\nrate-speed = false\nexport = 21\n[filters]\nipv4 = false\n",
        )
        .unwrap();

        let mut cli = empty_cli_config();
        cli.general.export = Some(33);
        let (config, _) = read_config_file(Some(&path), Some(cli));
        std::fs::remove_file(path).unwrap();

        assert_eq!(config.general.export, Some(33));
        assert!(!config.general.rate);
        assert!(!config.filters.ipv4);
    }
}
