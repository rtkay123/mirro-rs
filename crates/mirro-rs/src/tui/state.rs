use std::fmt::Display;

#[cfg(feature = "archlinux")]
use archlinux::ArchLinux;

use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;

use crate::tui::actions::Action;

use super::{actions::Actions, inputs::key::Key, io::IoEvent};

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub enum Filter {
    Alphabetical,
    Mirror,
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Filter::Alphabetical => "a",
            Filter::Mirror => "c",
        };
        write!(f, "{str}")
    }
}

pub struct App {
    pub show_popup: bool,
    pub actions: Actions,
    #[cfg(feature = "archlinux")]
    pub mirrors: Option<ArchLinux>,
    pub io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    pub input: String,
    pub input_cursor_position: usize,
    pub show_input: bool,
    pub active_filter: Vec<Filter>,
}

impl App {
    #[cfg(feature = "archlinux")]
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        Self {
            actions: vec![Action::Quit].into(),
            show_popup: true,
            show_input: false,
            mirrors: None,
            io_tx,
            input: String::default(),
            input_cursor_position: 0,
            active_filter: vec![Filter::Alphabetical, Filter::Mirror],
        }
    }

    pub async fn dispatch_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("action: [{action:?}]");
            if key.is_exit() && !self.show_input {
                AppReturn::Exit
            } else {
                match action {
                    Action::ClosePopUp => {
                        if !self.show_input {
                            self.show_popup = !self.show_popup;
                        } else {
                            insert_character(self, 'p');
                        }
                        AppReturn::Continue
                    }
                    Action::Quit => {
                        insert_character(self, 'q');
                        AppReturn::Continue
                    }
                    Action::ShowInput => {
                        // replace log widget
                        self.show_input = !self.show_input;
                        AppReturn::Continue
                    }
                }
            }
        } else {
            if self.show_input {
                match key {
                    Key::Enter => todo!(),
                    Key::Backspace => {
                        if !self.input.is_empty() {
                            self.input = format!(
                                "{}{}",
                                &self.input[..self.input_cursor_position - 1],
                                &self.input[self.input_cursor_position..]
                            );
                            self.input_cursor_position -= 1;
                        }
                    }
                    Key::Left => {
                        if self.input_cursor_position > 0 {
                            self.input_cursor_position -= 1;
                        }
                    }
                    Key::Right => {
                        if self.input_cursor_position < self.input.width() {
                            self.input_cursor_position += 1;
                        } else {
                            self.input_cursor_position = self.input.width();
                        };
                    }
                    Key::Delete => {
                        if self.input_cursor_position < self.input.width() {
                            self.input.remove(self.input_cursor_position);
                        }
                    }
                    Key::Home => {
                        self.input_cursor_position = 0;
                    }
                    Key::End => {
                        self.input_cursor_position = self.input.width();
                    }
                    Key::Char(c) => {
                        insert_character(self, c);
                    }
                    Key::Esc => {
                        self.show_input = false;
                    }
                    _ => {
                        warn!("No action associated to {key}");
                    }
                }
            } else {
                warn!("No action associated to {key}");
            }
            AppReturn::Continue
        }
    }

    pub async fn dispatch(&mut self, action: IoEvent) {
        self.show_popup = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.show_popup = false;
            error!("Error from dispatch {e}");
        };
    }

    pub async fn update_on_tick(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    pub fn ready(&mut self) {
        self.actions = vec![Action::ShowInput, Action::ClosePopUp, Action::Quit].into();
        self.show_popup = false;
    }
}

fn insert_character(app: &mut App, key: char) {
    app.input.insert(app.input_cursor_position, key);
    app.input_cursor_position += 1;
}
