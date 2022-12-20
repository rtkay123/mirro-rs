use anyhow::Result;
use std::sync::Arc;

use log::error;
use tokio::sync::Mutex;

use crate::tui::state::App;

use super::IoEvent;

pub struct IoAsyncHandler {
    app: Arc<Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<Mutex<App>>) -> Self {
        Self { app }
    }

    #[cfg(feature = "archlinux")]
    pub async fn initialise(&mut self) -> Result<()> {
        use anyhow::bail;
        match archlinux::archlinux().await {
            Ok(mirrors) => {
                let mut app = self.app.lock().await;
                app.mirrors = Some(mirrors);
            }
            Err(e) => bail!("{e}"),
        }
        Ok(())
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        if let Err(e) = match io_event {
            IoEvent::Initialize => self.initialise().await,
        } {
            error!("{e}");
        }

        let mut app = self.app.lock().await;
        app.ready();
    }
}
