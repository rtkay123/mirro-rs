#![warn(
    missing_docs,
    rustdoc::broken_intra_doc_links,
    missing_debug_implementations
)]

//! # mirrors-arch
use std::time::{Duration, Instant};

use futures::{future::BoxFuture, FutureExt};
use log::{info, trace};
use reqwest::{header::LOCATION, ClientBuilder, Response, StatusCode};

use crate::response::external::Root;

#[cfg(test)]
mod test;

mod errors;
pub use errors::Error;

pub use reqwest::Client;

mod response;
#[cfg(feature = "time")]
#[doc(no_inline)]
pub use chrono;

pub use response::{external::Protocol, internal::*};

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

    let root: Root = response.json().await?;

    let body = ArchLinux::from(root);
    let count = body.countries.len();
    info!("located mirrors from {count} countries");
    Ok(body)
}

async fn get_response(source: &str, with_timeout: Option<u64>) -> Result<Response> {
    trace!("creating http client");
    let client = get_client(with_timeout)?;

    trace!("sending request");
    let response = client.get(source).send().await?;

    Ok(response)
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
    deserialise_mirrors(response).await
}

async fn deserialise_mirrors(response: Response) -> Result<(ArchLinux, String)> {
    let root: Root = response.json().await?;

    let value = serde_json::to_string(&root)?;
    Ok((ArchLinux::from(root), value))
}

/// The same as [get_mirrors_with_raw](get_mirrors_with_raw) but uses a specified
/// [Client](reqwest::Client) for requests
pub async fn get_mirrors_with_client(source: &str, client: Client) -> Result<(ArchLinux, String)> {
    let response = client.get(source).send().await?;
    deserialise_mirrors(response).await
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
pub fn get_client(with_timeout: Option<u64>) -> Result<Client> {
    let timeout = with_timeout.map(Duration::from_secs);

    let mut client_builder = ClientBuilder::new();
    if let Some(timeout) = timeout {
        client_builder = client_builder.timeout(timeout).connect_timeout(timeout);
    }

    Ok(client_builder.build()?)
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
/// # let client = get_client(Some(5))?;
/// let (duration, url) = rate_mirror(url, client).await?;
/// #  Ok(())
/// # }
/// ```
pub fn rate_mirror(url: String, client: Client) -> BoxFuture<'static, Result<(Duration, String)>> {
    async move {
        let uri = format!("{url}{FILE_PATH}");

        let now = Instant::now();

        let response = client.get(&uri).send().await?;

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
    client: Client,
) -> Result<(chrono::DateTime<chrono::Utc>, String)> {
    let mirror = mirror.into();
    let lastsync_url = format!("{mirror}lastsync");

    let timestamp = client
        .get(&lastsync_url)
        .send()
        .await
        .map_err(|e| Error::Request(e.to_string()))?
        .text()
        .await?;

    let result = chrono::NaiveDateTime::parse_from_str(&timestamp, "%s")
        .map(|res| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(res, chrono::Utc))
        .map_err(Error::TimeError)?;

    Ok((result, mirror))
}
