use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
/// Error type definitions returned by the crate
pub enum Error {
    /// The connection could not be made (perhaps a network error is
    /// the cause)
    #[error("could not establish connection")]
    Connection(#[from] reqwest::Error),
    /// The response could not be parsed to an internal type
    #[error("could not parse response")]
    Parse(#[from] serde_json::Error),
    /// The mirror could not be rated
    #[error("could not find file (expected {qualified_url:?}, from {url:?}), server returned {status_code:?}")]
    Rate {
        /// The URL including the filepath that was sent in the request
        qualified_url: String,
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
