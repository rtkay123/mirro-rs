use itertools::Itertools;
use serde::Deserialize;
use tracing::debug;

#[cfg(feature = "time")]
use time::OffsetDateTime;

use super::external::{Protocol, Root};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ArchLinux {
    pub cutoff: u32,
    #[cfg(feature = "time")]
    #[serde(with = "time::serde::rfc3339")]
    pub last_check: OffsetDateTime,
    #[cfg(not(feature = "time"))]
    pub last_check: String,
    pub num_checks: u8,
    pub check_frequency: u16,
    pub countries: Vec<Country>,
    pub version: u8,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Country {
    pub name: String,
    pub code: String,
    pub mirrors: Vec<Mirror>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Mirror {
    pub url: String,
    pub protocol: Protocol,
    pub completion_pct: f32,
    pub delay: Option<i64>,
    pub score: Option<f64>,
    pub duration_stddev: Option<f64>,
    #[cfg(feature = "time")]
    #[serde(with = "time::serde::rfc3339::option", default)]
    pub last_sync: Option<OffsetDateTime>,
    #[cfg(not(feature = "time"))]
    pub last_sync: Option<String>,
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
