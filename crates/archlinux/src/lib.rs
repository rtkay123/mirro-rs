#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations
)]

//! # mirrors-arch
use std::time::{Duration, Instant};

use futures::{future::BoxFuture, FutureExt};
use hyper::{
    client::HttpConnector,
    header::LOCATION,
    StatusCode,
    {body::Buf, Body, Client, Request, Uri},
};
use hyper_tls::HttpsConnector;
use log::{info, trace};

use crate::response::external::Root;

#[cfg(test)]
mod tests;

mod response;
#[cfg(feature = "time")]
#[doc(no_inline)]
pub use chrono::*;

pub use response::{external::Protocol, internal::*};

use thiserror::Error;

#[derive(Error, Debug)]
/// Error type definitions returned by the crate
pub enum Error {
    /// The connection could not be made (perhaps a network error is
    /// the cause)
    #[error("could not establish connection")]
    Connection(#[from] hyper::Error),
    /// The response could not be parsed to an internal type
    #[error("could not parse response")]
    Parse(#[from] serde_json::Error),
    /// The constructed URL is invalid
    #[error("the url you provided `{0}` is invalid")]
    InvalidURL(#[from] hyper::http::uri::InvalidUri),
    /// The mirror could not be rated
    #[error("could not find file (expected {qualified_url:?}, from {url:?}), server returned {status_code:?}")]
    Rate {
        /// The URL including the filepath that was sent in the request
        qualified_url: Uri,
        /// The URL of the particular mirror
        url: String,
        /// The status code returned by the server
        status_code: StatusCode,
    },
    #[error("could not build request {0}")]
    /// There was an error performing the request
    Request(String),
    /// There was an error performing the request
    #[cfg(feature = "time")]
    #[error("could not parse time")]
    TimeError(#[from] chrono::ParseError),
}

type Result<T> = std::result::Result<T, Error>;

pub(crate) const FILE_PATH: &str = "core/os/x86_64/core.db.tar.gz";

/// Get ArchLinux mirrors from an `json` endpoint and return them in a [minified](ArchLinux) format
///
/// # Parameters
///
/// - `source` - The URL to query for a mirrorlist
/// - `with_timeout` - Connection timeout (in seconds) to be used in network requests
///
/// # Example
///
/// ```rust
/// # use mirrors_arch::get_mirrors;
/// # async fn foo()->Result<(), Box<dyn std::error::Error>>{
/// let arch_mirrors = get_mirrors("https://archlinux.org/mirrors/status/json/", None).await?;
/// println!("{arch_mirrors:?}");
/// #    Ok(())
/// # }
/// ```
pub async fn get_mirrors(source: &str, with_timeout: Option<u64>) -> Result<ArchLinux> {
    let response = get_response(source, with_timeout).await?;

    let bytes = hyper::body::aggregate(response.into_body()).await?;

    let root: Root = serde_json::from_reader(bytes.reader())?;

    let body = ArchLinux::from(root);
    let count = body.countries.len();
    info!("located mirrors from {count} countries");
    Ok(body)
}

async fn get_response(source: &str, with_timeout: Option<u64>) -> Result<hyper::Response<Body>> {
    trace!("creating http client");
    let client = get_client(with_timeout);
    let uri = source.parse::<Uri>()?;

    trace!("building request");
    let req = Request::builder()
        .uri(uri)
        .body(Body::empty())
        .map_err(|f| Error::Request(f.to_string()))?;
    Ok(client.request(req).await?)
}

/// The same as [get_mirrors](get_mirrors) but returns a tuple including the json as a
/// `String`
///
/// # Example
///
/// ```rust
/// # use mirrors_arch::get_mirrors_with_raw;
/// # async fn foo()->Result<(), Box<dyn std::error::Error>>{
/// let timeout = Some(10);
/// let arch_mirrors = get_mirrors_with_raw("https://my-url.com/json/", timeout).await?;
/// println!("{arch_mirrors:?}");
/// #    Ok(())
/// # }
/// ```
pub async fn get_mirrors_with_raw(
    source: &str,
    with_timeout: Option<u64>,
) -> Result<(ArchLinux, String)> {
    let response = get_response(source, with_timeout).await?;

    let bytes = hyper::body::aggregate(response.into_body()).await?;

    let root: Root = serde_json::from_reader(bytes.reader())?;
    let value = serde_json::to_string(&root)?;

    Ok((ArchLinux::from(root), value))
}

/// Parses a `string slice` to the [ArchLinux](ArchLinux) type
///
/// # Parameters
/// - `contents` - A `json` string slice to be parsed and returned as a [mirrorlist](ArchLinux)
///
/// # Example
///
/// ```rust
/// # use mirrors_arch::parse_local;
/// # async fn foo()->Result<(), Box<dyn std::error::Error>>{
/// let json = std::fs::read_to_string("archmirrors.json")?;
/// let arch_mirrors = parse_local(&json)?;
/// println!("{arch_mirrors:?}");
/// #  Ok(())
/// # }
/// ```
pub fn parse_local(contents: &str) -> Result<ArchLinux> {
    let vals = ArchLinux::from(serde_json::from_str::<Root>(contents)?);
    Ok(vals)
}

/// Gets a client that can be used to rate mirrors
///
/// # Parameters
/// - `with_timeout` - an optional connection timeout to be used when rating the mirrors
///
/// # Example
///
/// ```rust
/// # use mirrors_arch::get_client;
/// # async fn foo()->Result<(), Box<dyn std::error::Error>>{
/// let timeout = Some(5);
/// let client = get_client(timeout);
/// #  Ok(())
/// # }
/// ```
pub fn get_client(
    with_timeout: Option<u64>,
) -> Client<hyper_timeout::TimeoutConnector<HttpsConnector<HttpConnector>>> {
    let timeout = with_timeout.map(Duration::from_secs);
    let h = HttpsConnector::new();
    let mut connector = hyper_timeout::TimeoutConnector::new(h);
    connector.set_connect_timeout(timeout);
    connector.set_read_timeout(timeout);
    connector.set_write_timeout(timeout);
    Client::builder().build::<_, hyper::Body>(connector)
}

/// Queries a mirrorlist and calculates how long it took to get a response
///
/// # Parameters
/// - `url` - The mirrorlist
/// - `client` - The client returned from [get_client](get_client)
///
/// # Example
///
/// ```rust
/// # use mirrors_arch::{get_client, rate_mirror};
/// # async fn foo()->Result<(), Box<dyn std::error::Error>>{
/// # let url = String::default();
/// # let client = get_client(Some(5));
/// let (duration, url) = rate_mirror(url, client).await?;
/// #  Ok(())
/// # }
/// ```
pub fn rate_mirror(
    url: String,
    client: Client<hyper_timeout::TimeoutConnector<HttpsConnector<HttpConnector>>>,
) -> BoxFuture<'static, Result<(Duration, String)>> {
    async move {
        let uri = format!("{url}{FILE_PATH}").parse::<Uri>()?;

        let req = Request::builder()
            .uri(&uri)
            .body(Body::empty())
            .map_err(|f| Error::Request(f.to_string()))?;
        let now = Instant::now();
        let response = client.request(req).await?;
        if response.status() == StatusCode::OK {
            Ok((now.elapsed(), url))
        } else if response.status() == StatusCode::MOVED_PERMANENTLY {
            if let Some(new_uri) = response.headers().get(LOCATION) {
                let new_url = String::from_utf8_lossy(new_uri.as_bytes()).replace(FILE_PATH, "");
                rate_mirror(new_url.to_string(), client.clone()).await
            } else {
                Err(Error::Rate {
                    qualified_url: uri,
                    url,
                    status_code: response.status(),
                })
            }
        } else {
            Err(Error::Rate {
                qualified_url: uri,
                url,
                status_code: response.status(),
            })
        }
    }
    .boxed()
}

///
#[cfg(feature = "time")]
pub async fn get_last_sync(
    mirror: impl Into<String>,
    client: Client<hyper_timeout::TimeoutConnector<HttpsConnector<HttpConnector>>>,
) -> Result<(DateTime<Utc>, String)> {
    let mirror = mirror.into();
    let url = mirror.parse::<Uri>()?;

    let req = Request::builder()
        .uri(&url)
        .body(Body::empty())
        .map_err(|f| Error::Request(f.to_string()))?;

    let response = client.request(req).await?;
    let body = hyper::body::to_bytes(response.into_body()).await?;
    let str_val = String::from_utf8_lossy(&body);
    let x = find_last_sync(&str_val).map_err(Error::TimeError)?;
    Ok((x, mirror))
}

#[cfg(feature = "time")]
fn find_last_sync(body: &str) -> std::result::Result<DateTime<Utc>, ParseError> {
    let item: Vec<_> = body
        .lines()
        .filter(|f| f.contains("lastsync"))
        .take(1)
        .collect();
    let item: Vec<_> = item[0].split_whitespace().collect();
    let date = &item[2];
    let time = &item[3];

    let dt = format!("{date} {time}");
    NaiveDateTime::parse_from_str(&dt, "%d-%b-%Y %H:%M")
        .map(|res| DateTime::<Utc>::from_utc(res, Utc))
}
