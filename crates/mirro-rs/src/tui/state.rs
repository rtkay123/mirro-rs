#[cfg(feature = "archlinux")]
use archlinux::ArchLinux;

use log::{debug, error, warn};
use unicode_width::UnicodeWidthStr;

use crate::tui::actions::Action;

use super::{
    actions::Actions,
    dispatch::{filter::Filter, sort::ViewSort},
    inputs::key::Key,
    io::IoEvent,
};

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
    pub input: String,
    pub input_cursor_position: usize,
    pub show_input: bool,
    pub active_sort: Vec<ViewSort>,
    pub active_filter: Vec<Filter>,
    pub scroll_pos: isize,
    pub filtered_count: usize,
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
            active_sort: vec![ViewSort::Alphabetical],
            active_filter: vec![Filter::Https, Filter::Http],
            scroll_pos: 0,
            filtered_count: 0,
        }
    }

    pub async fn dispatch_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            //debug!("action: [{action:?}]");
            if key.is_exit() && !self.show_input {
                AppReturn::Exit
            } else if self.show_input {
                match action {
                    Action::Quit => {
                        if key == Key::Char('q') {
                            insert_character(self, 'q');
                        }
                    }
                    Action::NavigateUp => {
                        if key == Key::Char('k') {
                            insert_character(self, 'k');
                        }
                    }
                    Action::NavigateDown => {
                        if key == Key::Char('j') {
                            insert_character(self, 'j');
                        }
                    }
                    Action::ViewSortAlphabetically => insert_character(self, '1'),
                    Action::ViewSortMirrorCount => insert_character(self, '2'),
                    _ => {}
                }
                AppReturn::Continue
            } else {
                match action {
                    Action::ClosePopUp => {
                        self.show_popup = !self.show_popup;
                        AppReturn::Continue
                    }
                    Action::Quit => AppReturn::Continue,
                    Action::ShowInput => {
                        self.show_input = !self.show_input;
                        AppReturn::Continue
                    }
                    Action::NavigateUp => {
                        self.previous();
                        AppReturn::Continue
                    }
                    Action::NavigateDown => {
                        self.next();
                        AppReturn::Continue
                    }
                    Action::FilterHttps => insert_filter(self, Filter::Https),
                    Action::FilterHttp => insert_filter(self, Filter::Http),
                    Action::FilterRsync => insert_filter(self, Filter::Rsync),
                    Action::FilterSyncing => insert_filter(self, Filter::InSync),
                    Action::ViewSortAlphabetically => insert_sort(self, ViewSort::Alphabetical),
                    Action::ViewSortMirrorCount => insert_sort(self, ViewSort::MirrorCount),
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
        self.actions = vec![
            Action::ShowInput,
            Action::ClosePopUp,
            Action::Quit,
            Action::NavigateDown,
            Action::NavigateUp,
            Action::FilterHttp,
            Action::FilterHttps,
            Action::FilterRsync,
            Action::FilterSyncing,
            Action::ViewSortAlphabetically,
            Action::ViewSortMirrorCount,
        ]
        .into();
        if let Some(mirrors) = self.mirrors.as_ref() {
            self.filtered_count = mirrors.countries.len();
        }
        self.show_popup = false;
    }

    pub fn next(&mut self) {
        if self.scroll_pos + 1 == self.filtered_count as isize {
            self.scroll_pos = 0;
        } else {
            self.scroll_pos += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.scroll_pos - 1 < 0 {
            self.scroll_pos = (self.filtered_count - 1) as isize;
        } else {
            self.scroll_pos -= 1;
        }
    }
}

fn insert_character(app: &mut App, key: char) {
    app.input.insert(app.input_cursor_position, key);
    app.input_cursor_position += 1;
}

fn insert_filter(app: &mut App, filter: Filter) -> AppReturn {
    if let Some(idx) = app.active_filter.iter().position(|f| *f == filter) {
        debug!("protocol filter: removed {filter}");
        app.active_filter.remove(idx);
    } else {
        debug!("protocol filter: added {filter}");
        app.active_filter.push(filter);
    }
    app.scroll_pos = 0;
    AppReturn::Continue
}

fn insert_sort(app: &mut App, view: ViewSort) -> AppReturn {
    app.active_sort.clear();
    app.active_sort.push(view);
    AppReturn::Continue
}
