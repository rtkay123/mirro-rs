use std::{io::ErrorKind, path::PathBuf};

use crate::cli::Args;

pub fn read_config_file() -> (Args, Option<PathBuf>) {
    let config_file = dirs::config_dir().map(|mut dir| {
        let crate_name = env!("CARGO_PKG_NAME");
        let mut location = PathBuf::from(crate_name);
        let mut file = PathBuf::from(crate_name);
        file.set_extension("toml");
        location.push(file.clone());

        let mut alt = dir.clone();
        dir.push(location);
        alt.push(file);
        check_file(&dir, Some(&alt))
    });
    match config_file {
        Some(Some(opts)) => opts,
        _ => {
            let config = include_str!("../../../../examples/mirro-rs.toml");
            let config: Args = toml::from_str(config).unwrap();
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
        Ok(contents) => match toml::from_str::<Args>(&contents) {
            Ok(e) => Some((e, Some(file.to_owned()))),
            Err(e) => {
                err(format!("config: {} -> {}", file.display(), e));
                call_backup(backup)
            }
        },
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                err(format!("config: {} -> {}", file.display(), e));
            }
            call_backup(backup)
        }
    }
}
