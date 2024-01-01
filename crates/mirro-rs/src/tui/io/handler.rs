use anyhow::{bail, Result};

use archlinux::{
    chrono::{DateTime, Utc},
    ArchLinux, Client, Country,
};

use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::SystemTime,
};

use itertools::Itertools;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::{
    config::Configuration,
    tui::state::{App, PopUpState},
};

use super::IoEvent;

const CACHE_FILE: &str = "cache";

pub struct IoAsyncHandler {
    app: Arc<Mutex<App>>,
    popup: Arc<Mutex<PopUpState>>,
    client: Client,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<Mutex<App>>, popup: Arc<Mutex<PopUpState>>, client: Client) -> Self {
        Self { app, popup, client }
    }

    pub async fn initialise(&mut self, config: Arc<std::sync::Mutex<Configuration>>) -> Result<()> {
        let (is_fresh, cache_file) = is_fresh(Arc::clone(&config)).await;
        if is_fresh {
            match tokio::fs::read_to_string(cache_file.as_ref().unwrap()).await {
                Ok(contents) => {
                    let result = archlinux::parse_local(&contents);
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
                                self.client.clone(),
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
                    if let Err(e) = get_new_mirrors(
                        cache_file,
                        Arc::clone(&self.app),
                        Arc::clone(&config),
                        self.client.clone(),
                    )
                    .await
                    {
                        error!("{e}");
                    }
                }
            }
            // read cached
        } else if let Err(e) = get_new_mirrors(
            cache_file,
            Arc::clone(&self.app),
            Arc::clone(&config),
            self.client.clone(),
        )
        .await
        {
            error!("{e}");
        }
        Ok(())
    }

    pub async fn close_popup(&self) -> Result<()> {
        let mut state = self.popup.lock().await;
        state.visible = false;
        Ok(())
    }

    pub async fn export(
        &self,
        in_progress: Arc<AtomicBool>,
        progress_transmitter: std::sync::mpsc::Sender<f32>,
        list_exported: Arc<AtomicBool>,
    ) -> Result<()> {
        in_progress.store(true, std::sync::atomic::Ordering::Relaxed);

        let mut popup_state = self.popup.lock().await;
        popup_state.popup_text = String::from("Exporting your mirrors, please wait...");
        popup_state.visible = true;
        std::mem::drop(popup_state);

        let (check_dl_speed, outfile, export_count, mut selected_mirrors, extra_urls, age) = {
            let app_state = self.app.lock().await;
            let configuration = app_state.configuration.lock().unwrap();
            let check_dl_speed = configuration.rate;
            let outfile = configuration.outfile.clone();
            let export_count = configuration.export as usize;
            let include = configuration.include.clone();
            let age = configuration.age;

            let selected_mirrors = app_state
                .selected_mirrors
                .iter()
                .map(|f| f.url.to_owned())
                .collect_vec();
            (
                check_dl_speed,
                outfile,
                export_count,
                selected_mirrors,
                include,
                age,
            )
        };

        let client = self.client.clone();
        let included_urls = tokio::spawn(async move {
            if let Some(extra_urls) = extra_urls {
                let results = check_extra_urls(extra_urls, age, client).await;
                Some(results)
            } else {
                None
            }
        });

        if let Ok(Some(Ok(mut item))) = included_urls.await {
            selected_mirrors.append(&mut item)
        }

        if !check_dl_speed {
            Self::write_to_file(
                outfile,
                &selected_mirrors,
                export_count,
                Some(in_progress),
                Some(Arc::clone(&self.popup)),
            )
            .await?;
        } else {
            Self::rate_mirrors(
                selected_mirrors,
                Some(Arc::clone(&self.popup)),
                Some(progress_transmitter),
                outfile,
                export_count,
                Some(in_progress),
                self.client.clone(),
            )
            .await;
        }
        // mirrorlist has been exported
        list_exported.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    pub async fn rate_mirrors(
        selected_mirrors: Vec<String>,
        popup: Option<Arc<Mutex<PopUpState>>>,
        progress_transmitter: Option<std::sync::mpsc::Sender<f32>>,
        outfile: PathBuf,
        export_count: usize,
        in_progress: Option<Arc<AtomicBool>>,
        client: Client,
    ) -> tokio::task::JoinHandle<Result<()>> {
        let mut mirrors = Vec::with_capacity(selected_mirrors.len());

        let mut set = tokio::task::JoinSet::new();

        for i in selected_mirrors.iter() {
            set.spawn(archlinux::rate_mirror(i.clone(), client.clone()));
        }

        let popup_state = popup.map(|popup| Arc::clone(&popup));

        tokio::spawn(async move {
            let mut current = 0;
            let len = set.len();

            while let Some(res) = set.join_next().await {
                match res {
                    Ok(Ok((duration, url))) => {
                        mirrors.push((duration, url));
                    }
                    Ok(Err(cause)) => match cause {
                        archlinux::Error::Connection(e) => {
                            error!("{e}");
                        }
                        archlinux::Error::Parse(e) => {
                            error!("{e}");
                        }
                        archlinux::Error::Rate {
                            qualified_url,
                            url,
                            status_code,
                        } => {
                            error!(
                                    "could not locate {qualified_url} from {url}, reason=> {status_code}",
                                );
                        }
                        archlinux::Error::Request(e) => {
                            error!("{e}");
                        }
                        archlinux::Error::TimeError(e) => {
                            error!("{e}")
                        }
                    },
                    Err(e) => error!("{e}"),
                }
                if let Some(progress_transmitter) = progress_transmitter.as_ref() {
                    current += 1;
                    let value = (current as f32) / (len as f32) * 100.0;
                    let _ = progress_transmitter.send(value);
                }
            }

            let results = {
                if !mirrors.is_empty() {
                    mirrors.sort_by(|(duration_a, _), (duration_b, _)| duration_a.cmp(duration_b));

                    mirrors.iter().map(|(_, url)| url.to_owned()).collect_vec()
                } else {
                    warn!("Exporting mirrors without rating...");
                    selected_mirrors.to_vec()
                }
            };

            Self::write_to_file(outfile, &results, export_count, in_progress, popup_state).await?;

            if let Some(progress) = progress_transmitter {
                let _ = progress.send(0.0); // reset progress
            }
            Ok(())
        })
    }

    pub async fn write_to_file(
        outfile: PathBuf,
        selected_mirrors: &[String],
        export_count: usize,
        in_progress: Option<Arc<AtomicBool>>,
        popup: Option<Arc<Mutex<PopUpState>>>,
    ) -> Result<()> {
        if let Some(dir) = outfile.parent() {
            info!(count = %export_count, "making export of mirrors");
            tokio::fs::create_dir_all(dir).await?;
            let output = &selected_mirrors[if selected_mirrors.len() >= export_count {
                ..export_count
            } else {
                ..selected_mirrors.len()
            }];
            let output: Vec<_> = output
                .iter()
                .map(|f| format!("Server = {f}$repo/os/$arch"))
                .collect();

            if let Err(e) = tokio::fs::write(&outfile, output.join("\n")).await {
                error!("{e}");
                return Err(e.into());
            } else {
                info!("Your mirrorlist has been exported");
            }
            if let Some(popup) = popup {
                let mut state = popup.lock().await;
                state.popup_text = format!(
                    "Your mirrorlist has been successfully exported to: {}",
                    outfile.display()
                );
            }
        }
        if let Some(in_progress) = in_progress {
            in_progress.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(())
    }

    pub async fn handle_io_event(
        &mut self,
        io_event: IoEvent,
        config: Arc<std::sync::Mutex<Configuration>>,
        list_exported: Arc<AtomicBool>,
    ) {
        if let Err(e) = match io_event {
            IoEvent::Initialise => {
                if let Err(e) = self.initialise(config).await {
                    error!("{e}")
                };
                let mut popup = self.popup.lock().await;
                popup.visible = false;
                Ok(())
            }
            IoEvent::ClosePopUp => self.close_popup().await,
            IoEvent::Export {
                in_progress,
                progress_transmitter,
            } => {
                self.export(in_progress, progress_transmitter, list_exported)
                    .await
            }
        } {
            error!("{e}");
        }
        let mut app = self.app.lock().await;
        app.ready();
    }
}

async fn check_extra_urls(
    extra_urls: Vec<String>,
    age: u16,
    client: Client,
) -> Result<Vec<String>> {
    info!("parsing included URLs");
    let mut results = Vec::with_capacity(extra_urls.len());

    let mut set = tokio::task::JoinSet::new();

    for i in extra_urls.into_iter() {
        set.spawn(archlinux::get_last_sync(i, client.clone()));
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Ok((dt, url))) => {
                let utc: DateTime<Utc> = Utc::now();
                let diff = utc - dt;
                if i64::from(age) >= diff.num_hours() {
                    results.push(url);
                }
            }
            Ok(Err(e)) => {
                error!("{e}")
            }
            Err(e) => {
                error!("{e}")
            }
        }
    }

    Ok(results)
}

// Do we get a new mirrorlist or nah
pub async fn is_fresh(
    app: Arc<std::sync::Mutex<Configuration>>,
) -> (bool, Option<std::path::PathBuf>) {
    if let Some(mut cache) = dirs::cache_dir() {
        let crate_name = env!("CARGO_PKG_NAME");
        cache.push(crate_name);
        if let Err(e) = tokio::fs::create_dir_all(&cache).await {
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
    client: Client,
) -> Result<()> {
    let url = Arc::new(Mutex::new(String::default()));
    let inner = Arc::clone(&url);
    {
        let mut val = inner.lock().await;
        let source = config.lock().unwrap();
        *val = source.url.clone();
    };
    let strs = url.lock().await;

    match archlinux::get_mirrors_with_client(&strs, client).await {
        Ok((mirrors, str_value)) => {
            if let Some(cache) = cache_file {
                if let Err(e) = tokio::fs::write(cache, str_value).await {
                    error!("{e}");
                }
            }

            show_stats(&mirrors.countries, false);

            let mut app = app.lock().await;
            app.mirrors = Some(mirrors);
        }
        Err(e) => {
            warn!("{e}, using old cached file fallback");
            if let Some(file) = cache_file {
                let slice = tokio::fs::read_to_string(file).await;

                match slice.ok().and_then(|f| archlinux::parse_local(&f).ok()) {
                    Some(mirrors) => {
                        update_state(app, Arc::clone(&config), mirrors).await;
                    }
                    _ => {
                        bail!("{e}");
                    }
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
