use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Root {
    pub cutoff: u32,
    pub last_check: DateTime<Utc>,
    pub num_checks: u8,
    pub check_frequency: u16,
    pub urls: Vec<Url>,
    pub version: u8,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Url {
    pub url: String,
    pub protocol: Protocol,
    pub last_sync: Option<DateTime<Utc>>,
    pub completion_pct: f32,
    pub delay: Option<i64>,
    pub duration_avg: Option<f64>,
    pub duration_stddev: Option<f64>,
    pub score: Option<f64>,
    pub active: bool,
    pub country: String,
    pub country_code: String,
    pub isos: bool,
    pub ipv4: bool,
    pub ipv6: bool,
    pub details: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Rsync,
    Http,
    Https,
}
