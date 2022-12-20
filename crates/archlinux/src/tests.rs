use super::Result;
use hyper::{body::Buf, Body, Client, Request, StatusCode, Uri};
use hyper_tls::HttpsConnector;

use crate::{response::external::Root, ARCHLINUX_MIRRORS, LOCAL_SOURCE};

async fn response() -> Result<hyper::Response<Body>> {
    let client = Client::builder().build::<_, Body>(HttpsConnector::new());
    let uri = ARCHLINUX_MIRRORS.parse::<Uri>()?;

    let req = Request::builder().uri(uri).body(Body::empty())?;

    Ok(client.request(req).await.unwrap())
}

#[tokio::test]
async fn arch_mirrors_ok() -> Result<()> {
    assert!(response().await.is_ok());
    assert_eq!(response().await?.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn archlinux_parse_body_remote() -> Result<()> {
    assert!(response().await.is_ok());
    let bytes = hyper::body::aggregate(response().await?.into_body()).await?;

    let root = serde_json::from_reader::<_, Root>(bytes.reader());

    assert!(root.is_ok());

    Ok(())
}

#[tokio::test]
async fn archlinux_parse_body_local() -> Result<()> {
    assert!(serde_json::from_str::<Root>(LOCAL_SOURCE).is_ok());
    Ok(())
}