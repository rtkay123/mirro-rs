use tracing::{debug, info};

use crate::response::external::Root;

#[cfg(test)]
mod tests;

mod response;
pub use response::internal::*;

const ARCHLINUX_MIRRORS: &str = "https://archlinux.org/mirrors/status/json/";
const LOCAL_SOURCE: &str = include_str!("../sample/archlinux.json");

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn archlinux() -> Result<ArchLinux> {
    debug!("Getting arch mirrors...");
    let response = reqwest::get(ARCHLINUX_MIRRORS).await?;

    let body = ArchLinux::from(response.json::<Root>().await?);
    let count = body.countries.len();
    info!("located mirrors from {count} countries");
    Ok(body)
}

pub fn archlinux_fallback() -> Result<ArchLinux> {
    let vals = ArchLinux::from(serde_json::from_str::<Root>(LOCAL_SOURCE)?);
    Ok(vals)
}
