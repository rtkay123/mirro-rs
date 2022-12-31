use std::sync::{Arc, Mutex};

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
use self::config::watch_config;

#[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
use self::cli::{Protocol, ARCH_URL, DEFAULT_CACHE_TTL, DEFAULT_MIRROR_COUNT};

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
    if !check_outfile(&args.general) {
        exit("outfile");
    }

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let get_bools = |args: &cli::Filters, config: &cli::Filters| {
        let ipv4 = if !args.ipv4 && config.ipv4 {
            true
        } else {
            args.ipv4
        };

        let ipv6 = if !args.ipv6 && config.ipv6 {
            true
        } else {
            args.ipv6
        };

        let isos = if !args.isos && config.isos {
            true
        } else {
            args.isos
        };

        (ipv4, ipv6, isos)
    };

    #[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
    let config = {
        let (ipv4, isos, ipv6) = get_bools(&args.filters, &config.filters);
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

        let completion = args
            .filters
            .completion_percent
            .unwrap_or_else(|| config.filters.completion_percent.unwrap());

        let age = args
            .filters
            .age
            .unwrap_or_else(|| config.filters.age.unwrap_or_default());

        let rate = if !args.general.rate && config.general.rate {
            true
        } else {
            args.general.rate
        };

        let timoeut = if args.general.timeout.is_none() && config.general.timeout.is_some() {
            config.general.timeout
        } else {
            args.general.timeout
        };

        config::Configuration::new(
            outfile, export, filters, view, sort, countries, ttl, url, ipv4, isos, ipv6,
            completion, age, rate, timoeut,
        )
    };

    #[cfg(not(any(feature = "json", feature = "toml", feature = "yaml")))]
    let config = {
        let outfile = args.general.outfile.or_else(|| exit("outfile")).unwrap();
        let export = args.general.export.unwrap_or(DEFAULT_MIRROR_COUNT);
        let filters = args
            .filters
            .protocols
            .unwrap_or_else(|| vec![Protocol::Http, Protocol::Https]);
        let view = args.general.view.unwrap_or_default();
        let sort = args.general.sort.unwrap_or_default();
        let countries = args.filters.country.unwrap_or_default();
        let ttl = args.general.ttl.unwrap_or(DEFAULT_CACHE_TTL);
        let url = args.general.url.unwrap_or_else(|| ARCH_URL.to_string());

        let completion = args.filters.completion_percent.unwrap_or(100);

        let age = args.filters.age.unwrap_or(0);
        let rate = args.general.rate;
        let timeout = args.general.timeout;

        config::Configuration::new(
            outfile,
            export,
            filters,
            view,
            sort,
            countries,
            ttl,
            url,
            args.filters.ipv4,
            args.filters.isos,
            args.filters.ipv6,
            completion,
            age,
            rate,
            timeout,
        )
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
