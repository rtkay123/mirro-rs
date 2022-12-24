use std::sync::{Arc, Mutex};

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
use self::config::watch_config;

#[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
use self::{
    cli::{ARCH_URL, DEFAULT_CACHE_TTL, DEFAULT_MIRROR_COUNT},
    tui::dispatch::filter::Filter,
};

mod cli;
mod config;
mod tui;

#[tokio::main]
async fn main() {
    let args = <cli::Args as clap::Parser>::parse();

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let (config, file) = config::read_config_file(args.config.as_ref());

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    if !check_outfile(&args) && !check_outfile(&config) {
        exit("outfile");
    }

    #[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
    if !check_outfile(&args) {
        exit("outfile");
    }

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let config = {
        let outfile = args.outfile.unwrap_or_else(|| config.outfile.unwrap());
        let export = args.export.unwrap_or_else(|| config.export.unwrap());
        let filters = args.filters.unwrap_or_else(|| config.filters.unwrap());
        let view = args.view.unwrap_or_else(|| config.view.unwrap());
        let sort = args.sort.unwrap_or_else(|| config.sort.unwrap());
        let countries = args.country.unwrap_or_else(|| config.country.unwrap());
        let ttl = args.ttl.unwrap_or_else(|| config.ttl.unwrap());
        let url = args.url.unwrap_or_else(|| config.url.unwrap());

        config::Configuration::new(outfile, export, filters, view, sort, countries, ttl, url)
    };

    #[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
    let config = {
        let outfile = args.outfile.or_else(|| exit("outfile")).unwrap();
        let export = args.export.unwrap_or(DEFAULT_MIRROR_COUNT);
        let filters = args
            .filters
            .unwrap_or_else(|| vec![Filter::Http, Filter::Https, Filter::Rsync]);
        let view = args.view.unwrap_or_default();
        let sort = args.sort.unwrap_or_default();
        let countries = args.country.unwrap_or_default();
        let ttl = args.ttl.unwrap_or(DEFAULT_CACHE_TTL);
        let url = args.url.unwrap_or_else(|| ARCH_URL.to_string());

        config::Configuration::new(outfile, export, filters, view, sort, countries, ttl, url)
    };

    let config = Arc::new(Mutex::new(config));

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    watch_config(file, Arc::clone(&config));

    let _ = tui::start(config).await;
    std::process::exit(0);
}

fn exit(value: &str) -> ! {
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
