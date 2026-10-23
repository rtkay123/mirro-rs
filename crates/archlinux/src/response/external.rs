use std::fmt::Display;

#[cfg(feature = "time")]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The ungrouped mirror status response returned by the Arch Linux API.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RawMirrorList {
    /// The mirror status cutoff.
    pub cutoff: u32,
    #[cfg(feature = "time")]
    /// The time of the last status check.
    pub last_check: DateTime<Utc>,
    #[cfg(not(feature = "time"))]
    /// The time of the last status check.
    pub last_check: String,
    /// The number of status checks.
    pub num_checks: u8,
    /// The interval between status checks.
    pub check_frequency: u16,
    /// The individual mirror entries.
    pub urls: Vec<RawMirror>,
    /// The response format version.
    pub version: u8,
}

/// An individual ungrouped mirror entry from the status response.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RawMirror {
    /// The mirror base URL.
    pub url: String,
    /// The mirror protocol.
    pub protocol: Protocol,
    #[cfg(feature = "time")]
    /// The last successful synchronisation time.
    pub last_sync: Option<DateTime<Utc>>,
    #[cfg(not(feature = "time"))]
    /// The last successful synchronisation time.
    pub last_sync: Option<String>,
    /// Fraction of successful checks.
    pub completion_pct: f32,
    /// Average mirror synchronisation delay in seconds.
    pub delay: Option<i64>,
    /// The average duration of mirror responses.
    pub duration_avg: Option<f64>,
    /// The standard deviation of mirror response duration.
    pub duration_stddev: Option<f64>,
    /// The calculated mirror score.
    pub score: Option<f64>,
    /// Whether the mirror is active.
    pub active: bool,
    /// The country name.
    pub country: String,
    /// The country code.
    pub country_code: String,
    /// Whether the mirror hosts ISO images.
    pub isos: bool,
    /// Whether the mirror supports IPv4.
    pub ipv4: bool,
    /// Whether the mirror supports IPv6.
    pub ipv6: bool,
    /// Additional mirror details.
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
