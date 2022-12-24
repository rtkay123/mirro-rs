use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use serde::Deserialize;

use crate::tui::dispatch::{filter::Filter, sort::ViewSort};

pub const DEFAULT_MIRROR_COUNT: u16 = 50;
pub const DEFAULT_CACHE_TTL: u16 = 24;
pub const ARCH_URL: &str = "https://archlinux.org/mirrors/status/json/";

#[derive(Parser, Debug, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// File to write mirrors to
    #[arg(short, long)]
    pub outfile: Option<PathBuf>,

    /// Number of mirrors to export [default: 50]
    #[arg(short, long)]
    #[serde(default = "default_export")]
    pub export: Option<u16>,

    /// Filters to use on mirrorlists
    #[arg(short, long, value_enum)]
    #[serde(default = "filters")]
    pub filters: Option<Vec<Filter>>,

    /// An order to view all countries
    #[arg(short, long, value_enum)]
    #[serde(default = "view")]
    pub view: Option<ViewSort>,

    /// Default sort for exported mirrors
    #[arg(short, long, value_enum)]
    #[serde(default = "sort")]
    pub sort: Option<SelectionSort>,

    /// Countries to search for mirrorlists
    #[arg(short)]
    #[serde(rename = "countries")]
    #[serde(default = "opt_vec")]
    pub country: Option<Vec<String>>,

    /// Number of hours to cache mirrorlist for
    #[arg(short, long)]
    #[serde(rename = "cache-ttl")]
    #[serde(default = "default_ttl")]
    pub ttl: Option<u16>,

    /// URL to check for mirrors
    #[arg(short, long)]
    #[serde(default = "url")]
    pub url: Option<String>,

    /// Specify alternate configuration file [default: $XDG_CONFIG_HOME/mirro-rs/mirro-rs.toml]
    #[arg(long)]
    #[serde(default = "configuration_dir")]
    #[cfg(any(feature = "toml", feature = "toml", feature = "json"))]
    pub config: Option<PathBuf>,
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

#[cfg(any(feature = "json", feature = "toml", feature = "yaml"))]
fn configuration_dir() -> Option<PathBuf> {
    None
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

fn opt_vec<T>() -> Option<Vec<T>> {
    None
}

fn sort() -> Option<SelectionSort> {
    Some(SelectionSort::Score)
}

fn view() -> Option<ViewSort> {
    Some(ViewSort::Alphabetical)
}

fn filters() -> Option<Vec<Filter>> {
    Some(vec![Filter::Http, Filter::Https])
}
