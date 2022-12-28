use itertools::Itertools;
use log::debug;
use serde::Deserialize;

#[cfg(feature = "time")]
use chrono::{DateTime, Utc};

use super::external::{Protocol, Root};

#[derive(Debug, Clone, PartialEq, Deserialize)]
/// The type returned as the mirrorlist
pub struct ArchLinux {
    /// Cutoff as returned by the server
    pub cutoff: u32,
    #[cfg(feature = "time")]
    /// Last successful check for mirrorlists
    pub last_check: DateTime<Utc>,
    #[cfg(not(feature = "time"))]
    /// Last successful check for mirrorlists
    pub last_check: String,
    /// Number of checks as returned by the server
    pub num_checks: u8,
    /// Check frequency as returned by the server
    pub check_frequency: u16,
    /// A list of [countries](Country) that group [mirrors](Mirror)
    pub countries: Vec<Country>,
    /// Version number as returned by the server
    pub version: u8,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
/// Holds a collection of mirrors
pub struct Country {
    /// The string representation of the country name
    pub name: String,
    /// A short representation the the current country, i.e `ZA` for `South Africa`
    pub code: String,
    /// A list of [mirrors](Mirror)
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
/// An ArchLinux mirror
pub struct Mirror {
    /// The mirror's URL
    pub url: String,
    /// Represents a mirror's [protocol](Protocol)
    pub protocol: Protocol,
    /// The number of mirror checks that have successfully connected and disconnected from the given URL. If it's less than 100, it may be a sign of an unreliable mirror
    pub completion_pct: f32,
    ///  The calculated average mirroring delay; e.g. the mean value of `last_check − last_sync` for each check of this mirror URL. Any value under one hour should be viewed as ideal.
    pub delay: Option<i64>,
    ///A very rough calculation for ranking mirrors. It is currently calculated as `(hours delay + average duration + standard deviation) / completion percentage`. Lower is better.
    pub score: Option<f64>,
    /// The standard deviation of the connect and retrieval time. A high standard deviation can indicate an unstable or overloaded mirror.
    pub duration_stddev: Option<f64>,
    /// Time when the last successful synchronisation occurred
    #[cfg(feature = "time")]
    pub last_sync: Option<DateTime<Utc>>,
    #[cfg(not(feature = "time"))]
    /// Time when the last successful synchronisation occurred
    pub last_sync: Option<String>,
    /// ipv4 enabled
    pub ipv4: bool,
    /// ipv6 enabled
    pub ipv6: bool,
    /// isos enabled
    pub isos: bool,
}

impl From<Root> for ArchLinux {
    fn from(mut raw: Root) -> Self {
        debug!("minifying mirrors");
        raw.urls.sort_by(|a, b| a.country.cmp(&b.country));
        let countries = raw
            .urls
            .iter()
            .dedup_by(|a, b| a.country == b.country)
            .map(|f| f.country.to_string())
            .collect_vec();

        let mut output = Vec::with_capacity(countries.len());
        let urls = &raw.urls;

        for i in countries.iter() {
            let mut code = String::default();
            let mirrors = urls
                .iter()
                .filter_map(|f| {
                    if f.country.eq_ignore_ascii_case(i) {
                        code = f.country_code.clone();
                        Some(Mirror {
                            url: f.url.clone(),
                            protocol: f.protocol,
                            completion_pct: f.completion_pct,
                            delay: f.delay,
                            score: f.score,
                            duration_stddev: f.duration_stddev,
                            #[cfg(feature = "time")]
                            last_sync: f.last_sync,
                            #[cfg(not(feature = "time"))]
                            last_sync: f.last_sync.clone(),
                            ipv4: f.ipv4,
                            ipv6: f.ipv6,
                            isos: f.isos,
                        })
                    } else {
                        None
                    }
                })
                .collect_vec();
            let country = Country {
                name: i.to_string(),
                code,
                mirrors,
            };
            output.push(country);
        }
        Self {
            cutoff: raw.cutoff,
            last_check: raw.last_check,
            num_checks: raw.num_checks,
            check_frequency: raw.check_frequency,
            countries: output,
            version: raw.version,
        }
    }
}
