use std::{
    fmt::Debug,
    path::Path,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
};

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::error;

use crate::config::read_config_file;

use super::Configuration;

pub fn watch_config(path: Option<PathBuf>, configuration: Arc<Mutex<Configuration>>) {
    if let Some(dir) = dirs::config_dir() {
        if let Some(path) = path {
            tokio::task::spawn_blocking(move || {
                if let Err(e) = async_watch(path, dir, configuration) {
                    error!("error: {:?}", e)
                }
            });
        }
    }
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, mpsc::Receiver<notify::Result<Event>>)> {
    let (tx, rx) = mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

fn async_watch(
    path: impl AsRef<Path> + Debug,
    dir: impl AsRef<Path> + Debug,
    config: Arc<Mutex<Configuration>>,
) -> notify::Result<()> {
    let (mut watcher, rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.

    watcher.watch(dir.as_ref(), RecursiveMode::Recursive)?;

    while let Ok(res) = rx.recv() {
        match res {
            Ok(event) => {
                if event
                    .paths
                    .iter()
                    .any(|f| f.file_name() == path.as_ref().file_name())
                {
                    let (config_file, _) = read_config_file(Some(path.as_ref().to_path_buf()));
                    let parsed_config = Configuration::new(
                        config_file.general.outfile.unwrap(),
                        config_file.general.export.unwrap(),
                        config_file.filters.protocols,
                        config_file.general.view.unwrap(),
                        config_file.general.sort.unwrap(),
                        config_file.filters.country,
                        config_file.general.ttl.unwrap(),
                        config_file.general.url.unwrap(),
                        config_file.filters.ipv4,
                        config_file.filters.isos,
                        config_file.filters.ipv6,
                        config_file.filters.completion_percent.unwrap(),
                        config_file.filters.age.unwrap_or_default(),
                        config_file.general.rate,
                        config_file.general.timeout,
                        config_file.general.include,
                        config_file.general.direct,
                    );

                    let mut new_config = config.lock().unwrap();
                    *new_config = parsed_config;
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }
    Ok(())
}
