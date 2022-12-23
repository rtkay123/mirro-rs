use hyper::{body::Buf, Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use tracing::{info, trace};

use crate::response::external::Root;

#[cfg(test)]
mod tests;

mod response;
pub use chrono::*;
pub use response::external::Protocol;
pub use response::internal::*;

const ARCHLINUX_MIRRORS: &str = "https://archlinux.org/mirrors/status/json/";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tracing::instrument]
pub async fn archlinux() -> Result<ArchLinux> {
    let response = get_response().await?;

    let bytes = hyper::body::aggregate(response.into_body()).await?;

    let root: Root = serde_json::from_reader(bytes.reader())?;

    let body = ArchLinux::from(root);
    let count = body.countries.len();
    info!("located mirrors from {count} countries");
    Ok(body)
}

async fn get_response() -> Result<hyper::Response<Body>> {
    trace!("creating http client");
    let client = Client::builder().build::<_, Body>(HttpsConnector::new());
    let uri = ARCHLINUX_MIRRORS.parse::<Uri>()?;

    trace!("building request");
    let req = Request::builder().uri(uri).body(Body::empty())?;
    Ok(client.request(req).await?)
}

pub async fn archlinux_with_raw() -> Result<(ArchLinux, String)> {
    let response = get_response().await?;

    let bytes = hyper::body::aggregate(response.into_body()).await?;

    let root: Root = serde_json::from_reader(bytes.reader())?;
    let value = serde_json::to_string(&root)?;

    Ok((ArchLinux::from(root), value))
}

pub fn archlinux_fallback(contents: &str) -> Result<ArchLinux> {
    let vals = ArchLinux::from(serde_json::from_str::<Root>(contents)?);
    Ok(vals)
}
