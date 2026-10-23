#[cfg(feature = "toml")]
mod file;

#[cfg(feature = "toml")]
mod watch;

#[cfg(feature = "toml")]
pub use watch::watch_config;

#[cfg(feature = "toml")]
pub use file::{empty_cli_config, read_config_file};
#[cfg(feature = "toml")]
pub use file::generate_config_file;

use std::path::PathBuf;

use crate::{
    cli::{self, ArgConfig, Protocol, SelectionSort, ViewSort},
    tui::view::sort::ExportSort,
};

#[cfg_attr(test, derive(Default))]
#[derive(Debug)]
pub struct Configuration {
    pub outfile: PathBuf,
    pub export: u16,
    pub filters: Vec<Protocol>,
    pub view: ViewSort,
    #[allow(dead_code)]
    pub sort: ExportSort,
    pub country: Vec<String>,
    pub ttl: u16,
    pub url: String,
    pub completion_percent: u8,
    pub age: u16,
    pub rate: bool,
    pub max_concurrent_ratings: usize,
    pub connection_timeout: Option<u64>,
    pub include: Option<Vec<String>>,
    pub direct: bool,
}

impl Configuration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        outfile: PathBuf,
        export: u16,
        mut filters: Vec<Protocol>,
        view: ViewSort,
        sort: SelectionSort,
        country: Vec<String>,
        ttl: u16,
        url: String,
        ipv4: bool,
        isos: bool,
        ipv6: bool,
        completion_percent: u8,
        age: u16,
        rate: bool,
        max_concurrent_ratings: usize,
        connection_timeout: Option<u64>,
        include: Option<Vec<String>>,
        direct: bool,
    ) -> Self {
        if ipv4 {
            filters.push(Protocol::Ipv4)
        }
        if ipv6 {
            filters.push(Protocol::Ipv6)
        }
        if isos {
            filters.push(Protocol::Isos)
        }
        Self {
            outfile,
            export,
            filters,
            view,
            sort: match sort {
                SelectionSort::Percentage => ExportSort::Completion,
                SelectionSort::Delay => ExportSort::MirroringDelay,
                SelectionSort::Duration => ExportSort::Duration,
                SelectionSort::Score => ExportSort::Score,
            },
            country,
            ttl,
            url,
            completion_percent,
            age,
            rate,
            max_concurrent_ratings,
            connection_timeout,
            include,
            direct,
        }
    }
}

impl From<ArgConfig> for Configuration {
    fn from(args: ArgConfig) -> Self {
        let outfile = args
            .general
            .outfile
            .or_else(|| crate::exit("outfile"))
            .unwrap();
        let export = args.general.export.unwrap_or(cli::DEFAULT_MIRROR_COUNT);
        let filters = args
            .filters
            .protocols
            .unwrap_or_else(|| vec![Protocol::Http, Protocol::Https]);
        let view = args.general.view.unwrap_or_default();
        let sort = args.general.sort.unwrap_or_default();
        let countries = args.filters.country.unwrap_or_default();
        let ttl = args.general.ttl.unwrap_or(cli::DEFAULT_CACHE_TTL);
        let url = args
            .general
            .url
            .unwrap_or_else(|| cli::ARCH_URL.to_string());

        let completion = args.filters.completion_percent.unwrap_or(100);

        let age = args.filters.age.unwrap_or(0);
        let rate = args.general.rate;
        let max_concurrent_ratings = args
            .general
            .max_concurrent_ratings
            .unwrap_or(cli::DEFAULT_MAX_CONCURRENT_RATINGS);
        let timeout = args.general.timeout;
        let include = args.general.include;

        Self::new(
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
            max_concurrent_ratings,
            timeout,
            include,
            args.general.direct,
        )
    }
}
