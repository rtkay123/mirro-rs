use std::path::PathBuf;

use clap::{Parser, ValueEnum};
#[cfg(feature = "toml")]
use clap::Subcommand;
use serde::{Deserialize, Serialize};

pub const DEFAULT_MIRROR_COUNT: u16 = 50;
pub const DEFAULT_CACHE_TTL: u16 = 24;
#[allow(dead_code)]
pub const DEFAULT_MAX_CONCURRENT_RATINGS: usize = 4;
pub const ARCH_URL: &str = "https://archlinux.org/mirrors/status/json/";

#[derive(Parser, Debug, Deserialize, Serialize)]
#[command(author, version, about, long_about = None)]
#[serde(default)]
pub struct ArgConfig {
    #[command(flatten)]
    pub general: Args,
    #[command(flatten)]
    pub filters: Filters,
    #[cfg(feature = "toml")]
    #[command(subcommand)]
    #[serde(skip)]
    pub command: Option<Commands>,
}

#[cfg(feature = "toml")]
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Write a default configuration file
    GenerateConfig {
        /// Destination file (defaults to the XDG config directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(clap::Args, Debug, Deserialize, Serialize, Default)]
#[command(author, version, about, long_about = None)]
#[serde(default)]
pub struct Args {
    /// File to write mirrors to
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outfile: Option<PathBuf>,

    /// Number of mirrors to export [default: 50]
    #[arg(short, long)]
    #[serde(default = "default_export")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<u16>,

    /// An order to view all countries
    #[arg(short, long, value_enum)]
    #[serde(default = "view")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<ViewSort>,

    /// Default sort for exported mirrors
    #[arg(short, long, value_enum)]
    #[serde(default = "sort")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<SelectionSort>,

    /// Number of hours to cache mirrorlist for
    #[arg(short, long)]
    #[serde(rename = "cache-ttl")]
    #[serde(default = "default_ttl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u16>,

    /// URL to check for mirrors
    #[arg(short, long)]
    #[serde(default = "url")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Specify alternate configuration file
    #[arg(long)]
    #[serde(skip)]
    #[cfg(feature = "toml")]
    pub config: Option<PathBuf>,

    /// Sort mirrorlists by download speed when exporting
    #[arg(short, long)]
    #[serde(default, rename = "rate-speed", skip_serializing_if = "is_false")]
    pub rate: bool,

    /// Connection timeout in seconds
    #[arg(long = "timeout")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    /// Maximum number of mirrors to rate concurrently [default: 4]
    #[arg(long = "max-concurrent-ratings")]
    #[serde(default, rename = "max-concurrent-ratings")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_ratings: Option<usize>,

    /// Extra CDNs to check for mirrors
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,

    /// Skip TUI session and directly export the mirrorlist
    #[arg(short, long)]
    #[serde(default, skip_serializing_if = "is_false")]
    pub direct: bool,
}

#[derive(clap::Args, Debug, Clone, Eq, PartialEq, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Filters {
    /// How old (in hours) should the mirrors be since last synchronisation
    #[arg(long, short)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u16>,

    /// Countries to search for mirrorlists
    #[arg(short)]
    #[serde(rename = "countries")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<Vec<String>>,

    /// Filters to use on mirrorlists
    #[arg(short, long, value_enum)]
    #[serde(default = "filters")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<Protocol>>,

    ///Only return mirrors that support IPv4.
    #[arg(long)]
    #[serde(default = "enable", skip_serializing_if = "is_false")]
    pub ipv4: bool,
    ///Only return mirrors that support IPv6.
    #[arg(long)]
    #[serde(default = "enable", skip_serializing_if = "is_false")]
    pub ipv6: bool,
    /// Only return mirrors that host ISOs.
    #[arg(long)]
    #[serde(default = "enable", skip_serializing_if = "is_false")]
    pub isos: bool,

    /// Set the minimum completion percent for the returned mirrors.
    #[arg(long)]
    #[serde(default = "completion", rename = "completion-percent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_percent: Option<u8>,
}

#[derive(
    Default, Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize, Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum SelectionSort {
    Percentage,
    Delay,
    Duration,
    #[default]
    Score,
}

fn enable() -> bool {
    true
}

fn is_false(value: &bool) -> bool {
    !value
}

fn completion() -> Option<u8> {
    Some(100)
}

fn url() -> Option<String> {
    Some(ARCH_URL.to_string())
}

fn default_ttl() -> Option<u16> {
    Some(DEFAULT_CACHE_TTL)
}

fn default_export() -> Option<u16> {
    Some(DEFAULT_MIRROR_COUNT)
}

fn sort() -> Option<SelectionSort> {
    Some(SelectionSort::Score)
}

fn view() -> Option<ViewSort> {
    Some(ViewSort::Alphabetical)
}

fn filters() -> Option<Vec<Protocol>> {
    Some(vec![Protocol::Http, Protocol::Https])
}

#[cfg_attr(test, derive(Default))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    #[cfg_attr(test, default)]
    Https,
    Http,
    Rsync,
    Ftp,
    #[value(skip)]
    InSync,
    #[value(skip)]
    Ipv4,
    #[value(skip)]
    Ipv6,
    #[value(skip)]
    Isos,
}

impl Default for ArgConfig {
    fn default() -> Self {
        Self {
            general: Args {
                outfile: Some(PathBuf::from("/home/user/example/generated-mirrors")),
                export: Some(DEFAULT_MIRROR_COUNT),
                view: Some(ViewSort::Alphabetical),
                sort: Some(SelectionSort::Score),
                ttl: Some(DEFAULT_CACHE_TTL),
                url: Some(ARCH_URL.to_string()),
                #[cfg(feature = "toml")]
                config: None,
                rate: true,
                timeout: Some(5),
                max_concurrent_ratings: Some(DEFAULT_MAX_CONCURRENT_RATINGS),
                include: None,
                direct: false,
            },
            filters: Filters {
                age: Some(0),
                country: Some(Vec::new()),
                protocols: Some(vec![Protocol::Https, Protocol::Http, Protocol::Rsync]),
                ipv4: true,
                ipv6: true,
                isos: true,
                completion_percent: Some(100),
            },
            #[cfg(feature = "toml")]
            command: None,
        }
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize, Serialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum ViewSort {
    #[default]
    Alphabetical,
    MirrorCount,
}
