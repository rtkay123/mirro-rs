use anyhow::Result;
use std::sync::Arc;

use log::error;
use tokio::sync::Mutex;

use crate::{config::Configuration, tui::state::App};

use super::IoEvent;

pub struct IoAsyncHandler {
    app: Arc<Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<Mutex<App>>) -> Self {
        Self { app }
    }

    #[cfg(feature = "archlinux")]
    pub async fn initialise(&mut self, config: Arc<std::sync::Mutex<Configuration>>) -> Result<()> {
        use anyhow::bail;
        use itertools::Itertools;
        // use log::info;
        // match archlinux::archlinux().await {
        //     Ok(mirrors) => {
        //         let mut count = 0;
        //         for i in mirrors.countries.iter() {
        //             count += i.mirrors.len();
        //         }
        //         info!(
        //             "Found {count} mirrors from {} countries.",
        //             mirrors.countries.len()
        //         );
        //         let mut app = self.app.lock().await;
        //         app.mirrors = Some(mirrors);
        //     }
        //     Err(e) => {
        //         error!("{e}, trying fallback");
        match archlinux::archlinux_fallback() {
            Ok(mut mirrors) => {
                let mut app = self.app.lock().await;
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
            Err(e) => {
                bail!("{e}")
            } //        }
              //     }
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
