use std::fmt::Display;

#[cfg(feature = "time")]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct Root {
    pub cutoff: u32,
    #[cfg(feature = "time")]
    pub last_check: DateTime<Utc>,
    #[cfg(not(feature = "time"))]
    pub last_check: String,
    pub num_checks: u8,
    pub check_frequency: u16,
    pub urls: Vec<Url>,
    pub version: u8,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct Url {
    pub url: String,
    pub protocol: Protocol,
    #[cfg(feature = "time")]
    pub last_sync: Option<DateTime<Utc>>,
    #[cfg(not(feature = "time"))]
    pub last_sync: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
/// Protocols serving the mirrors
pub enum Protocol {
    /// rsync
    Rsync,
    /// http
    Http,
    /// https
    Https,
    /// ftp
    Ftp,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Protocol::Rsync => "rsync",
                Protocol::Http => "http",
                Protocol::Https => "https",
                Protocol::Ftp => "ftp",
            }
        )
    }
}
