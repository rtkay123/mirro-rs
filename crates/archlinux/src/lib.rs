use hyper::{body::Buf, Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use tracing::{info, trace};

use crate::response::external::Root;

#[cfg(test)]
mod tests;

mod response;
pub use response::external::Protocol;
pub use response::internal::*;
#[cfg(feature = "time")]
pub use time::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tracing::instrument]
pub async fn archlinux(source: &str) -> Result<ArchLinux> {
    let response = get_response(source).await?;

    let bytes = hyper::body::aggregate(response.into_body()).await?;

    let root: Root = serde_json::from_reader(bytes.reader())?;

    let body = ArchLinux::from(root);
    let count = body.countries.len();
    info!("located mirrors from {count} countries");
    Ok(body)
}

async fn get_response(source: &str) -> Result<hyper::Response<Body>> {
    trace!("creating http client");
    let client = Client::builder().build::<_, Body>(HttpsConnector::new());
    let uri = source.parse::<Uri>()?;

    trace!("building request");
    let req = Request::builder().uri(uri).body(Body::empty())?;
    Ok(client.request(req).await?)
}

pub async fn archlinux_with_raw(source: &str) -> Result<(ArchLinux, String)> {
    let response = get_response(source).await?;

    let bytes = hyper::body::aggregate(response.into_body()).await?;

    let root: Root = serde_json::from_reader(bytes.reader())?;
    let value = serde_json::to_string(&root)?;

    Ok((ArchLinux::from(root), value))
}

pub fn archlinux_fallback(contents: &str) -> Result<ArchLinux> {
    let vals = ArchLinux::from(serde_json::from_str::<Root>(contents)?);
    Ok(vals)
}
