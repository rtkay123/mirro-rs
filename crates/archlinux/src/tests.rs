use reqwest::{Response, StatusCode};

use super::Result;

use crate::{find_last_sync, get_client, response::external::Root};

const ARCHLINUX_MIRRORS: &str = "https://archlinux.org/mirrors/status/json/";
const LOCAL_SOURCE: &str = include_str!("../sample/archlinux.json");

async fn response() -> Result<Response> {
    let client = get_client(None)?;

    let response = client.get(ARCHLINUX_MIRRORS).send().await;

    Ok(response?)
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

    let root = response().await?.json::<Root>().await;

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
    let client = get_client(None)?;
    let urls = [
        "https://mirror.ufs.ac.za/archlinux/",
        "https://cloudflaremirrors.com/archlinux/",
        "https://mirror.lesviallon.fr/archlinux/",
    ];

    for i in urls.iter() {
        let response = client.get(*i).send().await?.bytes().await?;
        let str_val = String::from_utf8_lossy(&response);
        let last_sync = find_last_sync(&str_val);

        assert!(last_sync.is_ok());
    }
    Ok(())
}
