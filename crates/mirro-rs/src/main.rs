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
    let args = <cli::ArgConfig as clap::Parser>::parse();

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let (config, file) = config::read_config_file(args.general.config.as_ref());

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    if !check_outfile(&args.general) && !check_outfile(&config.general) {
        exit("outfile");
    }

    #[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
    if !check_outfile(&args) {
        exit("outfile");
    }

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let config = {
        let outfile = args
            .general
            .outfile
            .unwrap_or_else(|| config.general.outfile.unwrap());
        let export = args
            .general
            .export
            .unwrap_or_else(|| config.general.export.unwrap());
        let filters = args
            .filters
            .protocols
            .unwrap_or_else(|| config.filters.protocols.unwrap());
        let view = args
            .general
            .view
            .unwrap_or_else(|| config.general.view.unwrap());
        let sort = args
            .general
            .sort
            .unwrap_or_else(|| config.general.sort.unwrap());
        let countries = args
            .filters
            .country
            .unwrap_or_else(|| config.filters.country.unwrap());
        let ttl = args
            .general
            .ttl
            .unwrap_or_else(|| config.general.ttl.unwrap());
        let url = args
            .general
            .url
            .unwrap_or_else(|| config.general.url.unwrap());

        let ipv4 = if !args.filters.ipv4 && config.filters.ipv4 {
            true
        } else {
            args.filters.ipv4
        };

        let ipv6 = if !args.filters.ipv6 && config.filters.ipv6 {
            true
        } else {
            args.filters.ipv6
        };

        let isos = if !args.filters.isos && config.filters.isos {
            true
        } else {
            args.filters.isos
        };
        let completion = args
            .filters
            .completion_percent
            .unwrap_or_else(|| config.filters.completion_percent.unwrap());

        config::Configuration::new(
            outfile, export, filters, view, sort, countries, ttl, url, ipv4, isos, ipv6, completion,
        )
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
        let countries = args.filters.country.unwrap_or_default();
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
