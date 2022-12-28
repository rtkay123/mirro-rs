#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
mod file;

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
mod watch;

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
pub use watch::watch_config;

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
pub use file::read_config_file;

use std::path::PathBuf;

use crate::{
    cli::SelectionSort,
    tui::dispatch::{
        filter::Protocol,
        sort::{ExportSort, ViewSort},
    },
};

pub struct Configuration {
    pub outfile: PathBuf,
    pub export: u16,
    pub filters: Vec<Protocol>,
    pub view: ViewSort,
    pub sort: ExportSort,
    pub country: Vec<String>,
    pub ttl: u16,
    pub url: String,
    pub ipv4: bool,
    pub ipv6: bool,
    pub isos: bool,
    pub completion_percent: u8,
    pub age: u16,
    pub rate: bool,
    pub connection_timeout: Option<u64>,
}

impl Configuration {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        outfile: PathBuf,
        export: u16,
        filters: Vec<Protocol>,
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
        connection_timeout: Option<u64>,
    ) -> Self {
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
                SelectionSort::Rate => todo!(),
            },
            country,
            ttl,
            url,
            ipv4,
            ipv6,
            isos,
            completion_percent,
            age,
            rate,
            connection_timeout,
        }
    }
}
