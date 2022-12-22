use std::fmt::Display;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ViewSort {
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
    StandardDeviation,
    Score,
}

impl Display for ExportSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ExportSort::Completion => "%",
            ExportSort::MirroringDelay => "μ",
            ExportSort::StandardDeviation => "σ",
            ExportSort::Score => "~",
        };
        write!(f, "{str}")
    }
}
