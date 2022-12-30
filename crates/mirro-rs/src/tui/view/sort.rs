use std::fmt::Display;

use clap::ValueEnum;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, ValueEnum, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewSort {
    #[default]
    Alphabetical,
    MirrorCount,
}

impl Display for ViewSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ViewSort::Alphabetical => "A",
            ViewSort::MirrorCount => "1",
        };
        write!(f, "{str}")
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ExportSort {
    Completion,
    MirroringDelay,
    Duration,
    Score,
}

impl Display for ExportSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ExportSort::Completion => "%",
            ExportSort::MirroringDelay => "μ",
            ExportSort::Duration => "σ",
            ExportSort::Score => "~",
        };
        write!(f, "{str}")
    }
}
