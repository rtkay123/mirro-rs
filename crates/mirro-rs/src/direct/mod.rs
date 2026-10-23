use anyhow::{Context, Result, bail};
use archlinux::{
    ArchLinux, Mirror,
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

    let (is_fresh, cache_file) = io::handler::is_fresh(configuration.ttl).await;
    let mirrorlist = if is_fresh {
        match tokio::fs::read_to_string(cache_file.as_ref().unwrap()).await {
            Ok(contents) => {
                let result = archlinux::parse_local(&contents);
                match result {
                    Ok(mirrors) => mirrors,
                    Err(e) => {
                        error!("{e}");
                        get_new_mirrors(&configuration, cache_file.as_deref()).await?
                    }
                }
            }
            Err(e) => {
                error!("{e}");
                get_new_mirrors(&configuration, cache_file.as_deref()).await?
            }
        }
    } else {
        get_new_mirrors(&configuration, cache_file.as_deref()).await?
    };

    let mut results = mirrorlist
        .countries
        .iter()
        .filter_map(|f| {
            let results = f
                .mirrors
                .iter()
                .filter(|f| filter_result(f, &configuration))
                .filter(|_| {
                    if configuration.country.is_empty() {
                        true
                    } else {
                        configuration
                            .country
                            .iter()
                            .any(|country| country.eq_ignore_ascii_case(&f.name))
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
        if let Err(e) = IoAsyncHandler::rate_mirrors(
            results,
            None,
            None,
            outfile,
            export_count.into(),
            None,
            client,
        )
        .await
        .await
        {
            error!("{e}");
        }
    } else {
        IoAsyncHandler::write_to_file(outfile, &results, export_count as usize, None, None).await;
    }

    Ok(())
}

async fn get_new_mirrors(
    config: &Configuration,
    cache_file: Option<&std::path::Path>,
) -> Result<ArchLinux> {
    match archlinux::get_mirrors_with_raw(&config.url, config.connection_timeout).await {
        Ok((resp, str_value)) => {
            if let Some(cache) = cache_file
                && let Err(e) = tokio::fs::write(cache, str_value).await
            {
                    error!("{e}");
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

pub fn filter_result(f: &Mirror, config: &Configuration) -> bool {
    if !f.active {
        return false;
    }

    let completion_ok = config.completion_percent as f32 <= f.completion_pct * 100.0
        && (!config.filters.contains(&Protocol::Ipv4) || f.ipv4)
        && (!config.filters.contains(&Protocol::Isos) || f.isos)
        && (!config.filters.contains(&Protocol::Ipv6) || f.ipv6);

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
