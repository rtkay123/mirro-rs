use log::debug;
use serde::Deserialize;

#[cfg(feature = "time")]
use chrono::{DateTime, Utc};

use super::external::{Protocol, RawMirrorList};

/// A raw mirror list whose entries are ordered by country.
#[derive(Debug, Clone, PartialEq)]
pub struct SortedMirrorList(RawMirrorList);

impl RawMirrorList {
    /// Sort mirror entries by country without converting them to grouped mirrors.
    pub fn sort_by_country(mut self) -> SortedMirrorList {
        self.urls.sort_by(|a, b| a.country.cmp(&b.country));
        SortedMirrorList(self)
    }
}

impl SortedMirrorList {
    /// Group the sorted entries into the compact mirror-list representation.
    pub fn into_archlinux(self) -> ArchLinux {
        let raw = self.0;
        debug!("minifying mirrors");

        let mut countries: Vec<Country> = Vec::new();
        for url in raw.urls {
            if countries
                .last()
                .is_none_or(|country| !country.name.eq_ignore_ascii_case(&url.country))
            {
                countries.push(Country {
                    name: url.country.clone(),
                    code: url.country_code.clone(),
                    mirrors: Vec::new(),
                });
            }

            let country = countries.last_mut().expect("country was just inserted");
            country.mirrors.push(Mirror {
                url: url.url,
                protocol: url.protocol,
                completion_pct: url.completion_pct,
                delay: url.delay,
                score: url.score,
                duration_stddev: url.duration_stddev,
                active: url.active,
                #[cfg(feature = "time")]
                last_sync: url.last_sync,
                #[cfg(not(feature = "time"))]
                last_sync: url.last_sync,
                ipv4: url.ipv4,
                ipv6: url.ipv6,
                isos: url.isos,
            });
        }

        ArchLinux {
            cutoff: raw.cutoff,
            last_check: raw.last_check,
            num_checks: raw.num_checks,
            check_frequency: raw.check_frequency,
            countries,
            version: raw.version,
        }
    }
}

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
    /// The mean synchronisation delay in seconds across checks for this mirror. Values under 3600 seconds are generally ideal.
    pub delay: Option<i64>,
    ///A very rough calculation for ranking mirrors. It is currently calculated as `(hours delay + average duration + standard deviation) / completion percentage`. Lower is better.
    pub score: Option<f64>,
    /// The standard deviation of the connect and retrieval time. A high standard deviation can indicate an unstable or overloaded mirror.
    pub duration_stddev: Option<f64>,
    /// Whether the mirror is currently active.
    pub active: bool,
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

impl From<RawMirrorList> for ArchLinux {
    fn from(raw: RawMirrorList) -> Self {
        raw.sort_by_country().into_archlinux()
    }
}

impl From<SortedMirrorList> for ArchLinux {
    fn from(sorted: SortedMirrorList) -> Self {
        sorted.into_archlinux()
    }
}
