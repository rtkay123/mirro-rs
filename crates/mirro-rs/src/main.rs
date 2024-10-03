mod cli;
mod config;
mod dbg;
mod direct;
#[cfg(test)]
mod test;
mod tui;

use std::sync::{atomic::AtomicBool, Arc, Mutex};

use tracing::error;

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
use self::config::watch_config;

#[tokio::main]
async fn main() {
    let args = <cli::ArgConfig as clap::Parser>::parse();

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let (config, file) = config::read_config_file(args.general.config.as_ref());

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    if !check_outfile(&args.general) && !check_outfile(&config.general) {
        exit("outfile");
    }

    #[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
    if !check_outfile(&args.general) {
        exit("outfile");
    }

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let config = config::Configuration::from((args, config));

    #[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
    let config = config::Configuration::from(args);

    dbg::log(config.direct);

    let mut export_ok = false;
    if config.direct {
        if let Err(ref e) = direct::begin(config).await {
            error!("{e}")
        } else {
            export_ok = true;
        };
    } else {
        let config = Arc::new(Mutex::new(config));

        #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
        watch_config(file, Arc::clone(&config));

        let list_exported = Arc::new(AtomicBool::new(export_ok));
        if let Err(ref e) = tui::start(config, Arc::clone(&list_exported)).await {
            error!("{e}");
            export_ok = false;
        } else {
            export_ok = list_exported.load(std::sync::atomic::Ordering::Relaxed);
        }
    }
    std::process::exit(if export_ok { 0 } else { 1 });
}

pub fn exit(value: &str) -> ! {
    let cmd = clap::Command::new("mirro-rs");
    let mut err = clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(&cmd);
    err.insert(
        clap::error::ContextKind::InvalidArg,
        clap::error::ContextValue::String(format!("--{value}")),
    );

    err.insert(
        clap::error::ContextKind::InvalidValue,
        clap::error::ContextValue::String(String::default()),
    );
    err.exit();
}

fn check_outfile(config: &cli::Args) -> bool {
    if let Some(ref outfile) = config.outfile {
        if outfile.to_string_lossy().ends_with('/') || outfile.to_string_lossy().is_empty() {
            exit("outfile");
        }
        true
    } else {
        false
    }
}
