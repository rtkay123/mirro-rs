use chrono::{DateTime, Utc};
use itertools::Itertools;
use serde::Deserialize;
use tracing::debug;

use super::external::{Protocol, Root};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ArchLinux {
    pub cutoff: u32,
    pub last_check: DateTime<Utc>,
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
    pub duration_avg: Option<f64>,
    pub duration_stddev: Option<f64>,
    pub last_sync: Option<DateTime<Utc>>,
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
                            duration_avg: f.duration_avg,
                            duration_stddev: f.duration_stddev,
                            last_sync: f.last_sync,
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
