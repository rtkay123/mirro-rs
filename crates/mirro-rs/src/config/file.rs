use std::{
    fmt::Display,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use crate::cli::Args;

pub fn read_config_file(file: Option<impl AsRef<Path>>) -> (Args, Option<PathBuf>) {
    let config_file = if let Some(ref file) = file {
        let buf = file.as_ref().to_path_buf();
        Some(check_file(&buf, None))
    } else {
        dirs::config_dir().map(|dir| {
            let is_yaml = extension() == Config::Yaml;
            match get_config(dir.clone(), &extension().to_string()) {
                Some(res) => Some(res),
                None => {
                    if is_yaml {
                        get_config(dir, "yml")
                    } else {
                        None
                    }
                }
            }
        })
    };
    match config_file {
        Some(Some(opts)) => opts,
        #[allow(unused_variables)]
        _ => {
            #[cfg(feature = "toml")]
            let config_str = include_str!("../../../../examples/mirro-rs.toml");

            #[cfg(feature = "yaml")]
            let config_str = include_str!("../../../../examples/mirro-rs.yaml");

            #[cfg(feature = "json")]
            let config_str = include_str!("../../../../examples/mirro-rs.json");

            #[cfg(feature = "toml")]
            let config: Args = toml::from_str(config_str).unwrap();

            #[cfg(feature = "json")]
            let config: Args = serde_json::from_str(config_str).unwrap();

            #[cfg(feature = "yaml")]
            let config: Args = serde_yaml::from_str(config_str).unwrap();

            (config, None)
        }
    }
}

fn check_file(file: &PathBuf, backup: Option<&PathBuf>) -> Option<(Args, Option<PathBuf>)> {
    let err = |e| {
        eprintln!("{e}");
    };

    let call_backup = |backup: Option<&PathBuf>| {
        if let Some(backup) = backup {
            check_file(backup, None)
        } else {
            None
        }
    };

    let f = std::fs::read_to_string(file);

    match f {
        #[allow(unused_variables)]
        Ok(contents) => {
            #[cfg(feature = "toml")]
            let result: Result<Args, _> = toml::from_str::<Args>(&contents);

            #[cfg(feature = "json")]
            let result: Result<Args, _> = serde_json::from_str::<Args>(&contents);

            #[cfg(feature = "yaml")]
            let result: Result<Args, _> = serde_yaml::from_str::<Args>(&contents);

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

fn extension() -> Config {
    if cfg!(feature = "config-json") {
        Config::Json
    } else if cfg!(feature = "config-yaml") {
        Config::Yaml
    } else {
        Config::Toml
    }
}

#[derive(PartialEq, Eq)]
enum Config {
    Json,
    Toml,
    Yaml,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Config::Json => "json",
                Config::Toml => "toml",
                Config::Yaml => "yaml",
            }
        )
    }
}

fn get_config(mut dir: PathBuf, extension: &str) -> Option<(Args, Option<PathBuf>)> {
    let crate_name = env!("CARGO_PKG_NAME");
    let mut location = PathBuf::from(crate_name);
    let mut file = PathBuf::from(crate_name);
    file.set_extension(extension);
    location.push(file.clone());
    let mut alt = dir.clone();
    dir.push(location);
    alt.push(file);
    check_file(&dir, Some(&alt))
}
