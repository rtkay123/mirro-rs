use anyhow::{bail, Result};

use archlinux::{ArchLinux, Country};

use std::{path::PathBuf, sync::Arc, time::SystemTime};

use itertools::Itertools;
use log::{error, info, warn};
use tokio::sync::Mutex;

use crate::{config::Configuration, tui::state::App};

use super::IoEvent;

const CACHE_FILE: &str = "cache";

pub struct IoAsyncHandler {
    app: Arc<Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn initialise(&mut self, config: Arc<std::sync::Mutex<Configuration>>) -> Result<()> {
        let (is_fresh, cache_file) = is_fresh(Arc::clone(&config));
        if is_fresh {
            match std::fs::read_to_string(cache_file.as_ref().unwrap()) {
                Ok(contents) => {
                    let result = archlinux::archlinux_fallback(&contents);
                    match result {
                        Ok(mirrors) => {
                            show_stats(&mirrors.countries, is_fresh);

                            update_state(Arc::clone(&self.app), Arc::clone(&config), mirrors).await;
                        }
                        Err(e) => {
                            if let Err(f) = get_new_mirrors(
                                cache_file,
                                Arc::clone(&self.app),
                                Arc::clone(&config),
                            )
                            .await
                            {
                                error!("{e}, {f}");
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("{e}");
                    if let Err(e) =
                        get_new_mirrors(cache_file, Arc::clone(&self.app), Arc::clone(&config))
                            .await
                    {
                        error!("{e}");
                    }
                }
            }
            // read cached
        } else if let Err(e) =
            get_new_mirrors(cache_file, Arc::clone(&self.app), Arc::clone(&config)).await
        {
            error!("{e}");
        }
        Ok(())
    }

    pub async fn handle_io_event(
        &mut self,
        io_event: IoEvent,
        config: Arc<std::sync::Mutex<Configuration>>,
    ) {
        if let Err(e) = match io_event {
            IoEvent::Initialise => self.initialise(config).await,
        } {
            error!("{e}");
        }

        let mut app = self.app.lock().await;
        app.ready();
    }
}

// Do we get a new mirrorlist or nah
fn is_fresh(app: Arc<std::sync::Mutex<Configuration>>) -> (bool, Option<std::path::PathBuf>) {
    if let Some(mut cache) = dirs::cache_dir() {
        let crate_name = env!("CARGO_PKG_NAME");
        cache.push(crate_name);
        if let Err(e) = std::fs::create_dir_all(&cache) {
            error!("could not create cache directory, {e}");
        }
        cache.push(CACHE_FILE);
        if cache.exists() {
            let config = app.lock().unwrap();
            let expires = config.ttl;
            drop(config);

            let duration = cache.metadata().map(|f| {
                f.modified().map(|f| {
                    let now = SystemTime::now();
                    now.duration_since(f)
                })
            });
            match duration {
                Ok(Ok(Ok(duration))) => {
                    let hours = duration.as_secs() / 3600;
                    if hours < expires as u64 {
                        (true, Some(cache))
                    } else {
                        (false, Some(cache))
                    }
                }
                _ => (false, Some(cache)),
            }
        } else {
            (false, Some(cache))
        }
    } else {
        (false, None)
    }
}

async fn get_new_mirrors(
    cache_file: Option<PathBuf>,
    app: Arc<Mutex<App>>,
    config: Arc<std::sync::Mutex<Configuration>>,
) -> Result<()> {
    let url = Arc::new(Mutex::new(String::default()));
    let inner = Arc::clone(&url);
    let timeout = {
        let mut val = inner.lock().await;
        let source = config.lock().unwrap();
        *val = source.url.clone();
        source.connection_timeout
    };
    let strs = url.lock().await;

    match archlinux::archlinux_with_raw(&strs, timeout).await {
        Ok((mirrors, str_value)) => {
            if let Some(cache) = cache_file {
                if let Err(e) = std::fs::write(cache, str_value) {
                    error!("{e}");
                }
            }

            show_stats(&mirrors.countries, false);

            let mut app = app.lock().await;
            app.mirrors = Some(mirrors);
        }
        Err(e) => {
            warn!("{e}, using old cached file fallback");
            let file = cache_file.map(|f| {
                std::fs::read_to_string(f)
                    .ok()
                    .map(|f| archlinux::archlinux_fallback(&f).ok())
            });
            match file {
                Some(Some(Some(mirrors))) => {
                    update_state(app, Arc::clone(&config), mirrors).await;
                }
                _ => {
                    bail!("{e}");
                }
            }
        }
    }
    Ok(())
}

async fn update_state(
    app: Arc<Mutex<App>>,
    config: Arc<std::sync::Mutex<Configuration>>,
    mut mirrors: ArchLinux,
) {
    let mut app = app.lock().await;
    let config = config.lock().unwrap();
    if !config.country.is_empty() {
        let items = mirrors
            .countries
            .into_iter()
            .filter(|f| {
                config
                    .country
                    .iter()
                    .any(|a| a.eq_ignore_ascii_case(&f.name))
            })
            .collect_vec();
        mirrors.countries = items;
    }
    app.mirrors = Some(mirrors);
}

fn show_stats(slice: &[Country], is_cache: bool) {
    let mut count = 0;
    for i in slice.iter() {
        count += i.mirrors.len();
    }
    info!(
        "Found {count} mirrors from {} countries{}.",
        slice.len(),
        if is_cache { " cached" } else { "" }
    );
}
