use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use tracing::error;

use crate::cli::ArgConfig;

fn default_args() -> ArgConfig {
    let config_str = include_str!("../../../../examples/mirro-rs.toml");
    toml::from_str(config_str).unwrap()
}

pub fn read_config_file(file: Option<impl AsRef<Path>>) -> (ArgConfig, Option<PathBuf>) {
    let config_file = match file {
        Some(file) => check_file(file.as_ref(), None),
        None => dirs::config_dir().and_then(|dir| get_config(&dir)),
    };
    config_file.unwrap_or_else(|| (default_args(), None))
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
