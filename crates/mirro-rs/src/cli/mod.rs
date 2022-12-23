use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use serde::Deserialize;

use crate::tui::dispatch::{filter::Filter, sort::ViewSort};

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
    #[arg(short, long)]
    #[serde(rename = "countries")]
    #[serde(default = "opt_vec")]
    pub country: Option<Vec<String>>,

    /// Number of hours to cache mirrorlist for
    #[arg(short, long)]
    #[serde(rename = "cache-ttl")]
    #[serde(default = "default_ttl")]
    pub ttl: Option<u16>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectionSort {
    Percentage,
    Delay,
    Duration,
    Score,
}

fn default_ttl() -> Option<u16> {
    Some(24)
}

fn default_export() -> Option<u16> {
    Some(50)
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
