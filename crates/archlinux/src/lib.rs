use std::time::{Duration, Instant};

use hyper::client::HttpConnector;
use hyper::StatusCode;
use hyper::{body::Buf, Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use tracing::{info, trace};

use crate::response::external::Root;

#[cfg(test)]
mod tests;

mod response;
#[cfg(feature = "time")]
pub use chrono::*;
pub use response::external::Protocol;
pub use response::internal::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not establish connection")]
    Connection(#[from] hyper::Error),
    #[error("could not parse response")]
    Parse(#[from] serde_json::Error),
    #[error("the url you provided `{0}` is invalid")]
    InvalidURL(#[from] hyper::http::uri::InvalidUri),
    #[error("could not find file (expected {qualified_url:?}, from {url:?}), server returned {status_code:?}")]
    Rate {
        qualified_url: Uri,
        url: String,
        status_code: StatusCode,
    },
    #[error("could not build request {0}")]
    Request(String),
}

type Result<T> = std::result::Result<T, Error>;

pub(crate) const FILE_PATH: &str = "core/os/x86_64/core.db.tar.gz";

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
    let client = get_client();
    let uri = source.parse::<Uri>()?;

    trace!("building request");
    let req = Request::builder()
        .uri(uri)
        .body(Body::empty())
        .map_err(|f| Error::Request(f.to_string()))?;
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

pub fn get_client() -> Client<HttpsConnector<HttpConnector>> {
    Client::builder().build::<_, Body>(HttpsConnector::new())
}

pub async fn rate_mirror<T: AsRef<str>>(
    url: T,
    client: Client<HttpsConnector<HttpConnector>>,
) -> Result<(Duration, T)> {
    let uri = format!("{}{FILE_PATH}", url.as_ref()).parse::<Uri>()?;

    let req = Request::builder()
        .uri(&uri)
        .body(Body::empty())
        .map_err(|f| Error::Request(f.to_string()))?;
    let now = Instant::now();
    let response = client.request(req).await?;
    if response.status() == StatusCode::OK {
        Ok((now.elapsed(), url))
    } else {
        Err(Error::Rate {
            qualified_url: uri,
            url: url.as_ref().to_string(),
            status_code: response.status(),
        })
    }
}
