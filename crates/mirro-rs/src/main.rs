mod cli;
mod config;
mod dbg;
mod direct;
#[cfg(test)]
mod test;
mod tui;

use std::sync::{Arc, Mutex};

use tracing::error;

#[cfg(feature = "toml")]
use self::config::watch_config;

#[tokio::main]
async fn main() {
    let args = <cli::ArgConfig as clap::Parser>::parse();

    #[cfg(feature = "toml")]
    if let Some(cli::Commands::GenerateConfig { output }) = args.command.as_ref() {
        match config::generate_config_file(output.clone()) {
            Ok(path) => println!("Configuration written to {}", path.display()),
            Err(error) => {
                error!("could not generate config file: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    #[cfg(feature = "toml")]
    let (config, config_file) = {
        let config_path = args.general.config.clone();
        let (config, file) = config::read_config_file(config_path.as_ref(), Some(args));

        if !check_outfile(&config.general) {
            exit("outfile");
        }

        (config::Configuration::from(config), file)
    };

    #[cfg(not(feature = "toml"))]
    let config = config::Configuration::from(args);

    dbg::log(config.direct);

    if config.direct {
        if let Err(e) = direct::begin(config).await {
            error!("{e}")
        }
    } else {
        let config = Arc::new(Mutex::new(config));

        #[cfg(feature = "toml")]
        watch_config(config_file, Arc::clone(&config));

        let _ = tui::start(config).await;
    }
    std::process::exit(0);
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

#[cfg(feature = "toml")]
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
