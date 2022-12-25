use std::fmt::Display;

use clap::ValueEnum;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Https,
    Http,
    Rsync,
    #[value(skip)]
    InSync,
}

impl From<archlinux::Protocol> for Protocol {
    fn from(value: archlinux::Protocol) -> Self {
        match value {
            archlinux::Protocol::Rsync => Self::Rsync,
            archlinux::Protocol::Http => Self::Http,
            archlinux::Protocol::Https => Self::Https,
        }
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Protocol::Https => "https",
                Protocol::Http => "http",
                Protocol::Rsync => "rsync",
                Protocol::InSync => "in-sync",
            }
        )
    }
}
