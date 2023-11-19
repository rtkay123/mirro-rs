use std::{io::ErrorKind, path::PathBuf};

use std::path::Path;

use itertools::Itertools;
use tracing::error;

use crate::cli::ArgConfig;

cfg_if::cfg_if! {
    if #[cfg(feature = "toml")] {
        fn default_args()->ArgConfig {
            let config_str = include_str!("../../../../examples/mirro-rs.toml");
            toml::from_str(config_str).unwrap()
        }
    } else if #[cfg(feature = "yaml")] {
        fn default_args()->ArgConfig {
            let config_str = include_str!("../../../../examples/mirro-rs.yaml");
            serde_yaml::from_str(config_str).unwrap()
        }
    } else {
        fn default_args()->ArgConfig {
            let config_str = include_str!("../../../../examples/mirro-rs.json");
            serde_json::from_str(config_str).unwrap()
        }
    }
}

pub fn read_config_file(file: Option<impl AsRef<Path>>) -> (ArgConfig, Option<PathBuf>) {
    let config_file = if let Some(ref file) = file {
        let buf = file.as_ref().to_path_buf();
        Some(check_file(&buf, None))
    } else {
        dirs::config_dir().map(|dir| get_config(dir, &extensions()))
    };
    match config_file {
        Some(Some(opts)) => opts,
        _ => (default_args(), None),
    }
}

fn check_file(file: &PathBuf, backup: Option<&PathBuf>) -> Option<(ArgConfig, Option<PathBuf>)> {
    let err = |e| {
        error!("{e}");
    };

    let call_backup = |backup: Option<&PathBuf>| {
        if let Some(backup) = backup {
            check_file(backup, None)
        } else {
            None
        }
    };

    let f = std::fs::read_to_string(file);

    let err_type = || -> Result<ArgConfig, String> {
        let ext = String::from_iter(extensions());
        Err(format!("unsupported file extension: file must be: {ext}"))
    };

    match f {
        #[allow(unused_variables)]
        Ok(contents) => {
            let result = if let Some(ext) = file.extension() {
                match ext.to_string_lossy().to_string().as_str() {
                    #[cfg(feature = "toml")]
                    "toml" => toml::from_str::<ArgConfig>(&contents).map_err(|e| e.to_string()),
                    #[cfg(feature = "json")]
                    "json" => {
                        serde_json::from_str::<ArgConfig>(&contents).map_err(|e| e.to_string())
                    }
                    #[cfg(feature = "yaml")]
                    "yaml" | "yml" => {
                        serde_yaml::from_str::<ArgConfig>(&contents).map_err(|e| e.to_string())
                    }
                    _ => err_type(),
                }
            } else {
                err_type()
            };

            match result {
                Ok(e) => Some((e, Some(file.to_owned()))),
                Err(e) => {
                    err(format!("config: {} -> {}", file.display(), e));
                    call_backup(backup)
                }
            }
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                err(format!("config: {} -> {}", file.display(), e));
            }
            call_backup(backup)
        }
    }
}

fn extensions() -> Vec<String> {
    let valid_extensions = vec![
        #[cfg(feature = "toml")]
        "toml",
        #[cfg(feature = "json")]
        "json",
        #[cfg(feature = "yaml")]
        "yaml",
        #[cfg(feature = "yaml")]
        "yml",
    ];

    valid_extensions.into_iter().map(String::from).collect_vec()
}

fn get_config(mut dir: PathBuf, extension: &[String]) -> Option<(ArgConfig, Option<PathBuf>)> {
    let crate_name = env!("CARGO_PKG_NAME");
    let location = PathBuf::from(crate_name);
    let mut file = PathBuf::from(crate_name);
    let mut result: Option<(ArgConfig, Option<PathBuf>)> = None;
    for i in extension.iter() {
        let mut inner_location = location.clone();
        file.set_extension(i);
        inner_location.push(file.clone());

        let mut alt = dir.clone();
        dir.push(inner_location);
        alt.push(file.clone());
        let interim = check_file(&dir, Some(&alt));
        if interim.is_some() {
            result = interim;
            break;
        }
    }
    result
}
