use super::Result;
use reqwest::StatusCode;

use crate::{response::external::Root, ARCHLINUX_MIRRORS, LOCAL_SOURCE};

async fn response() -> Result<reqwest::Response> {
    Ok(reqwest::get(ARCHLINUX_MIRRORS).await.unwrap())
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
    let body = response().await?.json::<Root>().await;
    assert!(body.is_ok());
    Ok(())
}

#[tokio::test]
async fn archlinux_parse_body_local() -> Result<()> {
    assert!(response().await.is_ok());
    let body = response().await?.json::<Root>().await;
    assert!(body.is_ok());

    assert!(serde_json::from_str::<Root>(LOCAL_SOURCE).is_ok());
    Ok(())
}
