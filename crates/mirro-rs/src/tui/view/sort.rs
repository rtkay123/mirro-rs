use std::fmt::Display;

use crate::cli::ViewSort;

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
#[cfg_attr(test, derive(Default))]
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ExportSort {
    Completion,
    MirroringDelay,
    #[cfg_attr(test, default)]
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
