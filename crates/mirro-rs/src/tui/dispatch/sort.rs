use std::fmt::Display;

#[allow(dead_code)]
pub enum Sort {
    Alphabetical,
    MirrorCount,
    Completion,
    MirrorindDelay,
    StandardDeviation,
    Score,
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Sort::Alphabetical => "a",
            Sort::MirrorCount => "c",
            Sort::Completion => "%",
            Sort::MirrorindDelay => "μ",
            Sort::StandardDeviation => "σ",
            Sort::Score => "~",
        };
        write!(f, "{str}")
    }
}
