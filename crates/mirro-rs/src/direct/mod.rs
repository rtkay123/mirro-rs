use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use archlinux::{
    chrono::{DateTime, Local},
    get_client, ArchLinux, Mirror,
};
use itertools::Itertools;
use tracing::error;

use crate::{
    cli::Protocol,
    config::Configuration,
    tui::io::{self, handler::IoAsyncHandler},
};

pub async fn begin(configuration: Configuration) -> Result<()> {
    let included = configuration.include.clone();
    let connection_timeout = configuration.connection_timeout;
    let rate = configuration.rate;
    let outfile = configuration.outfile.clone();
    let export_count = configuration.export;

    let config = Arc::new(Mutex::new(configuration));
    let (is_fresh, cache_file) = io::handler::is_fresh(Arc::clone(&config)).await;
    let mirrorlist = if is_fresh {
        match tokio::fs::read_to_string(cache_file.as_ref().expect("cache file to be available"))
            .await
        {
            Ok(contents) => {
                let result = archlinux::parse_local(&contents);
                match result {
                    Ok(mirrors) => mirrors,
                    Err(e) => {
                        error!("{e}");
                        get_new_mirrors(Arc::clone(&config), cache_file.as_ref()).await?
                    }
                }
            }
            Err(e) => {
                error!("{e}");
                get_new_mirrors(Arc::clone(&config), cache_file.as_ref()).await?
            }
        }
    } else {
        get_new_mirrors(Arc::clone(&config), cache_file.as_ref()).await?
    };

    let mut results = mirrorlist
        .countries
        .iter()
        .filter_map(|f| {
            let results = f
                .mirrors
                .iter()
                .filter(|f| filter_result(f, Arc::clone(&config)))
                .filter(|_| {
                    let conf = config.lock().unwrap();

                    if conf.country.is_empty() {
                        true
                    } else {
                        conf.country.iter().any(|b| b.eq_ignore_ascii_case(&f.name))
                    }
                })
                .collect_vec();
            if results.is_empty() {
                None
            } else {
                Some(results)
            }
        })
        .flatten()
        .map(|f| f.url.clone())
        .collect_vec();

    if let Some(mut included) = included {
        results.append(&mut included);
    }

    let client = get_client(connection_timeout)?;

    if rate {
        IoAsyncHandler::rate_mirrors(
            results,
            None,
            None,
            outfile,
            export_count.into(),
            None,
            client,
        )
        .await
        .await??;
    } else {
        IoAsyncHandler::write_to_file(outfile, &results, export_count as usize, None, None).await?;
    }

    Ok(())
}

async fn get_new_mirrors(
    config: Arc<Mutex<Configuration>>,
    cache_file: Option<&std::path::PathBuf>,
) -> Result<ArchLinux> {
    let (url, timeout) = {
        let config = config.lock().unwrap();
        (config.url.clone(), config.connection_timeout)
    };

    match archlinux::get_mirrors_with_raw(&url, timeout).await {
        Ok((resp, str_value)) => {
            if let Some(cache) = cache_file {
                if let Err(e) = tokio::fs::write(cache, str_value).await {
                    error!("{e}");
                }
            }
            Ok(resp)
        }
        Err(e) => {
            error!("{e}");
            if let Some(f) = cache_file {
                tokio::fs::read_to_string(f)
                    .await
                    .ok()
                    .and_then(|contents| archlinux::parse_local(&contents).ok())
                    .context("could not read cache file")
            } else {
                bail!("No cache file was configured")
            }
        }
    }
}

pub fn filter_result(f: &Mirror, configuration: Arc<Mutex<Configuration>>) -> bool {
    let mut config = configuration.lock().unwrap();

    let res = |config: &Configuration, f: &Mirror| {
        let mut completion_ok = config.completion_percent as f32 <= f.completion_pct * 100.0;
        let v4_on = config.filters.contains(&Protocol::Ipv4);
        let isos_on = config.filters.contains(&Protocol::Isos);
        let v6_on = config.filters.contains(&Protocol::Ipv6);
        if v4_on {
            completion_ok = completion_ok && f.ipv4;
        }

        if isos_on {
            completion_ok = completion_ok && f.isos;
        }

        if v6_on {
            completion_ok = completion_ok && f.ipv6;
        }
        completion_ok
    };

    if config.age != 0 {
        if let Some(mirror_sync) = f.last_sync {
            let now = Local::now();
            let mirror_sync: DateTime<Local> = DateTime::from(mirror_sync);
            let duration = now - mirror_sync;
            if !config.filters.contains(&Protocol::InSync) {
                config.filters.push(Protocol::InSync);
            }
            duration.num_hours() <= config.age.into()
                && config.filters.contains(&Protocol::from(f.protocol))
                && res(&config, f)
        } else {
            false
        }
    } else {
        config.filters.contains(&Protocol::from(f.protocol)) && res(&config, f)
    }
}
