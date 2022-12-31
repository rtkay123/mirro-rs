use super::Result;
use hyper::{body::Buf, Body, Client, Request, StatusCode, Uri};
use hyper_tls::HttpsConnector;

use crate::{find_last_sync, response::external::Root, Error};

const ARCHLINUX_MIRRORS: &str = "https://archlinux.org/mirrors/status/json/";
const LOCAL_SOURCE: &str = include_str!("../sample/archlinux.json");

async fn response() -> Result<hyper::Response<Body>> {
    let client = Client::builder().build::<_, Body>(HttpsConnector::new());
    let uri = ARCHLINUX_MIRRORS.parse::<Uri>()?;

    let req = Request::builder()
        .uri(uri)
        .body(Body::empty())
        .map_err(|f| Error::Request(f.to_string()))?;

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

#[ignore]
#[tokio::test]
#[cfg(feature = "time")]
async fn check_last_sync() -> Result<()> {
    let client = Client::builder().build::<_, Body>(HttpsConnector::new());
    let urls = vec![
        "https://mirror.ufs.ac.za/archlinux/",
        "https://cloudflaremirrors.com/archlinux/",
        "https://mirror.lesviallon.fr/archlinux/",
    ];

    for i in urls.iter() {
        let url = i.parse::<Uri>()?;

        let req = Request::builder()
            .uri(&url)
            .body(Body::empty())
            .map_err(|f| Error::Request(f.to_string()))?;

        let response = client.request(req).await?;
        let body = hyper::body::to_bytes(response.into_body()).await?;
        let str_val = String::from_utf8_lossy(&body);
        let last_sync = find_last_sync(&str_val);

        assert!(last_sync.is_ok());
    }
    Ok(())
}
