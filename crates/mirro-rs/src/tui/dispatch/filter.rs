use std::fmt::Display;

use clap::ValueEnum;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Filter {
    Https,
    Http,
    Rsync,
    InSync,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Filter::Https => "https",
                Filter::Http => "http",
                Filter::Rsync => "rsync",
                Filter::InSync => "in-sync",
            }
        )
    }
}
