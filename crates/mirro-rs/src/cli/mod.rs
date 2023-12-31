use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use serde::Deserialize;

pub const DEFAULT_MIRROR_COUNT: u16 = 50;
pub const DEFAULT_CACHE_TTL: u16 = 24;
pub const ARCH_URL: &str = "https://archlinux.org/mirrors/status/json/";

#[cfg_attr(test, derive(Default))]
#[derive(Parser, Debug, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct ArgConfig {
    #[command(flatten)]
    pub general: Args,
    #[command(flatten)]
    pub filters: Filters,
}

#[cfg_attr(test, derive(Default))]
#[derive(clap::Args, Debug, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// File to write mirrors to
    #[arg(short, long)]
    pub outfile: Option<PathBuf>,

    /// Number of mirrors to export [default: 50]
    #[arg(short, long)]
    #[serde(default = "default_export")]
    pub export: Option<u16>,

    /// An order to view all countries
    #[arg(short, long, value_enum)]
    #[serde(default = "view")]
    pub view: Option<ViewSort>,

    /// Default sort for exported mirrors
    #[arg(short, long, value_enum)]
    #[serde(default = "sort")]
    pub sort: Option<SelectionSort>,

    /// Number of hours to cache mirrorlist for
    #[arg(short, long)]
    #[serde(rename = "cache-ttl")]
    #[serde(default = "default_ttl")]
    pub ttl: Option<u16>,

    /// URL to check for mirrors
    #[arg(short, long)]
    #[serde(default = "url")]
    pub url: Option<String>,

    /// Specify alternate configuration file
    #[arg(long)]
    #[serde(skip)]
    #[cfg(any(feature = "toml", feature = "yaml", feature = "json"))]
    pub config: Option<PathBuf>,

    /// Sort mirrorlists by download speed when exporting
    #[arg(short, long)]
    #[serde(default, rename = "rate-speed")]
    pub rate: bool,

    /// Connection timeout in seconds
    #[arg(long = "timeout")]
    pub timeout: Option<u64>,

    /// Extra CDNs to check for mirrors
    #[arg(short, long)]
    #[serde(default)]
    pub include: Vec<String>,

    /// Skip TUI session and directly export the mirrorlist
    #[arg(short, long)]
    #[serde(default)]
    pub direct: bool,
}

#[cfg_attr(test, derive(Default))]
#[derive(clap::Args, Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Filters {
    /// How old (in hours) should the mirrors be since last synchronisation
    #[arg(long, short)]
    pub age: Option<u16>,

    /// Countries to search for mirrorlists
    #[arg(short)]
    #[serde(rename = "countries")]
    #[serde(default)]
    pub country: Option<Vec<String>>,

    /// Filters to use on mirrorlists
    #[arg(short, long, value_enum)]
    #[serde(default = "filters")]
    pub protocols: Option<Vec<Protocol>>,

    ///Only return mirrors that support IPv4.
    #[arg(long)]
    #[serde(default = "enable")]
    pub ipv4: bool,
    ///Only return mirrors that support IPv6.
    #[arg(long)]
    #[serde(default = "enable")]
    pub ipv6: bool,
    /// Only return mirrors that host ISOs.
    #[arg(long)]
    #[serde(default = "enable")]
    pub isos: bool,

    /// Set the minimum completion percent for the returned mirrors.
    #[arg(long)]
    #[serde(default = "completion", rename = "completion-percent")]
    pub completion_percent: Option<u8>,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize)]
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewSort {
    #[default]
    Alphabetical,
    MirrorCount,
}
