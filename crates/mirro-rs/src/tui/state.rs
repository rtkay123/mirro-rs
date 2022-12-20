#[cfg(feature = "archlinux")]
use archlinux::ArchLinux;

use log::{debug, error, warn};

use crate::tui::actions::Action;

use super::{actions::Actions, inputs::key::Key, io::IoEvent};

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub struct App {
    pub show_popup: bool,
    pub actions: Actions,
    #[cfg(feature = "archlinux")]
    pub mirrors: Option<ArchLinux>,
    pub io_tx: tokio::sync::mpsc::Sender<IoEvent>,
}

impl App {
    #[cfg(feature = "archlinux")]
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        Self {
            actions: vec![Action::Quit].into(),
            show_popup: true,
            mirrors: None,
            io_tx,
        }
    }

    pub async fn dispatch_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("action: [{:?}]", action);
            match action {
                Action::ClosePopUp => {
                    self.show_popup = !self.show_popup;
                    AppReturn::Continue
                }
                Action::Quit => AppReturn::Exit,
            }
        } else {
            warn!("No action associated to {key}");
            AppReturn::Continue
        }
    }

    pub async fn dispatch(&mut self, action: IoEvent) {
        self.show_popup = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.show_popup = false;
            error!("Error from dispatch {}", e);
        };
    }

    pub async fn update_on_tick(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    pub fn ready(&mut self) {
        self.show_popup = false;
    }
}
