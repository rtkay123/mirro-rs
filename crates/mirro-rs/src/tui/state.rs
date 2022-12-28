use archlinux::{ArchLinux, Country, DateTime, Utc};
use std::{
    io::Write,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
};
use tokio::task::JoinSet;

use crate::config::Configuration;

use itertools::Itertools;
use log::{debug, error, info, warn};
use tui::{
    style::{Color, Modifier, Style},
    widgets::{Cell, Row},
};
use unicode_width::UnicodeWidthStr;

use crate::tui::actions::Action;

use super::{
    actions::Actions,
    dispatch::{filter::Protocol, sort::ViewSort},
    inputs::key::Key,
    io::IoEvent,
    ui::filter_result,
};

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub struct App {
    pub actions: Actions,
    pub mirrors: Option<ArchLinux>,
    pub io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    pub input: String,
    pub input_cursor_position: usize,
    pub show_input: bool,
    pub scroll_pos: isize,
    pub filtered_countries: Vec<(Country, usize)>,
    pub selected_mirrors: Vec<SelectedMirror>,
    pub table_viewport_height: u16,
    pub configuration: Arc<Mutex<Configuration>>,
    pub show_insync: bool,
    pub show_popup: AtomicBool,
}

pub struct PopUpState {
    pub popup_text: String,
}

impl PopUpState {
    pub fn new() -> Self {
        Self {
            popup_text: String::from("Getting mirrors... please wait..."),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectedMirror {
    pub country_code: String,
    pub protocol: Protocol,
    pub completion_pct: f32,
    pub delay: Option<i64>,
    pub score: Option<f64>,
    pub duration_stddev: Option<f64>,
    pub last_sync: Option<DateTime<Utc>>,
    pub url: String,
}

impl App {
    pub fn new(
        io_tx: tokio::sync::mpsc::Sender<IoEvent>,
        configuration: Arc<Mutex<Configuration>>,
    ) -> Self {
        let show_sync = configuration.lock().unwrap();
        let sync = show_sync.age != 0;
        drop(show_sync);
        Self {
            actions: vec![Action::Quit].into(),
            show_popup: AtomicBool::new(true),
            show_input: false,
            mirrors: None,
            io_tx,
            input: String::default(),
            input_cursor_position: 0,
            configuration,
            scroll_pos: 0,
            table_viewport_height: 0,
            selected_mirrors: vec![],
            filtered_countries: vec![],
            show_insync: sync,
        }
    }

    pub async fn dispatch_action(
        &mut self,
        key: Key,
        popup_state: Arc<std::sync::Mutex<PopUpState>>,
        exporting: Arc<AtomicBool>,
        progress_transmitter: Sender<f32>,
    ) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
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
                        // self.show_popup = !self.show_popup;
                        self.show_popup
                            .store(false, std::sync::atomic::Ordering::Relaxed);
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
                    Action::FilterHttps => insert_filter(self, Protocol::Https),
                    Action::FilterHttp => insert_filter(self, Protocol::Http),
                    Action::FilterRsync => insert_filter(self, Protocol::Rsync),
                    Action::FilterSyncing => insert_filter(self, Protocol::InSync),
                    Action::ViewSortAlphabetically => insert_sort(self, ViewSort::Alphabetical),
                    Action::ViewSortMirrorCount => insert_sort(self, ViewSort::MirrorCount),
                    Action::ToggleSelect => {
                        self.focused_country();
                        AppReturn::Continue
                    }
                    Action::SelectionSortCompletionPct => {
                        self.selected_mirrors
                            .sort_by(|a, b| b.completion_pct.total_cmp(&a.completion_pct));
                        AppReturn::Continue
                    }
                    Action::SelectionSortDelay => {
                        self.selected_mirrors.sort_by(|a, b| {
                            let a = a.delay.unwrap_or(i64::MAX);
                            let b = b.delay.unwrap_or(i64::MAX);
                            a.partial_cmp(&b).unwrap()
                        });
                        AppReturn::Continue
                    }
                    Action::SelectionSortScore => {
                        self.selected_mirrors.sort_by(|a, b| {
                            let a = a.score.unwrap_or(f64::MAX);
                            let b = b.score.unwrap_or(f64::MAX);
                            a.partial_cmp(&b).unwrap()
                        });
                        AppReturn::Continue
                    }
                    Action::SelectionSortDuration => {
                        self.selected_mirrors.sort_by(|a, b| {
                            let a = a.duration_stddev.unwrap_or(f64::MAX);
                            let b = b.duration_stddev.unwrap_or(f64::MAX);
                            a.partial_cmp(&b).unwrap()
                        });
                        AppReturn::Continue
                    }
                    Action::Export => {
                        if !exporting.load(std::sync::atomic::Ordering::Relaxed) {
                            if self.selected_mirrors.is_empty() {
                                warn!("You haven't selected any mirrors yet");
                            } else {
                                {
                                    exporting.store(true, std::sync::atomic::Ordering::Relaxed);
                                    let mut state = popup_state.lock().unwrap();
                                    state.popup_text =
                                        String::from("exporting your mirrors, please wait");
                                    self.show_popup
                                        .store(true, std::sync::atomic::Ordering::Relaxed);
                                }

                                let mut mirrors = Vec::with_capacity(self.selected_mirrors.len());

                                let rate_enabled = {
                                    let config = self.configuration.lock().unwrap();
                                    config.rate
                                };

                                let results = self
                                    .selected_mirrors
                                    .iter()
                                    .map(|f| f.url.to_owned())
                                    .collect_vec();

                                if !rate_enabled {
                                    write_to_file(
                                        Arc::clone(&self.configuration),
                                        &results,
                                        popup_state,
                                        exporting,
                                    )
                                } else {
                                    let client = archlinux::get_client();
                                    let mut set = JoinSet::new();
                                    for i in self.selected_mirrors.iter() {
                                        let url = i.url.to_owned();
                                        set.spawn(archlinux::rate_mirror(url, client.clone()));
                                    }

                                    let config = Arc::clone(&self.configuration);

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
                                                    archlinux::Error::InvalidURL(e) => {
                                                        error!("{e}");
                                                    }
                                                    archlinux::Error::Rate {
                                                        qualified_url,
                                                        url,
                                                        status_code,
                                                    } => {
                                                        error!(
                                                        "could not locate {} from {url}, reason=> {status_code}",
                                                        qualified_url.to_string()
                                                    );
                                                    }
                                                    archlinux::Error::Request(e) => {
                                                        error!("{e}");
                                                    }
                                                },
                                                Err(res) => {
                                                    error!("{}", res.to_string());
                                                }
                                            }
                                            current += 1;
                                            let value = (current as f32) / (len as f32) * 100.0;
                                            progress_transmitter.send(value).unwrap();
                                        }

                                        let results = {
                                            if !mirrors.is_empty() {
                                                mirrors.sort_by(
                                                    |(duration_a, _), (duration_b, _)| {
                                                        duration_a.cmp(duration_b)
                                                    },
                                                );
                                                mirrors
                                                    .iter()
                                                    .map(|(_, url)| url.to_owned())
                                                    .collect_vec()
                                            } else {
                                                warn!("Exporting mirrors without rating...");
                                                results
                                            }
                                        };
                                        write_to_file(config, &results, popup_state, exporting);
                                        let _ = progress_transmitter.send(0.0); // reset progress
                                    });
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::FilterIpv4 => insert_filter(self, Protocol::Ipv4),
                    Action::FilterIpv6 => insert_filter(self, Protocol::Ipv6),
                    Action::FilterIsos => insert_filter(self, Protocol::Isos),
                }
            }
        } else {
            if self.show_input {
                match key {
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
                        self.scroll_pos = 0;
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
        self.show_popup
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Err(e) = self.io_tx.send(action).await {
            self.show_popup
                .store(false, std::sync::atomic::Ordering::Relaxed);
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
            Action::FilterIpv4,
            Action::FilterIpv6,
            Action::FilterIsos,
            Action::ToggleSelect,
            Action::ViewSortAlphabetically,
            Action::ViewSortMirrorCount,
            Action::SelectionSortCompletionPct,
            Action::SelectionSortDelay,
            Action::SelectionSortDuration,
            Action::SelectionSortScore,
            Action::Export,
        ]
        .into();

        self.show_popup
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn next(&mut self) {
        if self.scroll_pos + 1 == self.filtered_countries.len() as isize {
            self.scroll_pos = 0;
        } else {
            self.scroll_pos += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.scroll_pos - 1 < 0 {
            self.scroll_pos = (self.filtered_countries.len() - 1) as isize;
        } else {
            self.scroll_pos -= 1;
        }
    }

    pub fn view_fragments<'a, T>(&'a self, iter: &'a [T]) -> Vec<&'a [T]> {
        iter.chunks(self.table_viewport_height.into()).collect_vec()
    }

    pub fn rows(&self) -> Vec<Row> {
        self.filtered_countries
            .iter()
            .enumerate()
            .map(|(idx, (f, count))| {
                let c = if idx == self.filtered_countries.len() - 1 {
                    '╰'
                } else {
                    '├'
                };
                let mut selected = false;
                let default = format!("{c}─ [{}] {}", f.code, f.name);
                let item_name = match self.scroll_pos as usize == idx {
                    true => {
                        if idx == self.scroll_pos as usize {
                            selected = true;
                            format!("{c}─»[{}] {}«", f.code, f.name)
                        } else {
                            default
                        }
                    }
                    false => default,
                };

                let index = format!("  {idx}│");

                return Row::new([index, item_name, count.to_string()].iter().map(|c| {
                    Cell::from(c.clone()).style(if selected {
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Gray)
                    })
                }));
            })
            .collect_vec()
    }

    pub fn view<T: Copy>(&self, fragment: &[T]) -> T {
        fragment[self.fragment_number()]
    }

    pub fn focused_country(&mut self) {
        if self.mirrors.is_some() {
            let country = if self.scroll_pos < self.table_viewport_height as isize {
                let (country, _) = &self.filtered_countries[self.scroll_pos as usize];
                // we can directly index
                info!("selected: {}", country.name);
                country
            } else {
                let page = self.fragment_number();
                let index = (self.scroll_pos
                    - (page * self.table_viewport_height as usize) as isize)
                    as usize;
                let fragments = self.view_fragments(&self.filtered_countries);
                let frag = fragments[page];
                let (country, _) = &frag[index];
                info!("selected: {}", country.name);
                country
            };

            let mut mirrors = country
                .mirrors
                .iter()
                .filter(|f| filter_result(self, f))
                .map(|f| SelectedMirror {
                    country_code: country.code.to_string(),
                    protocol: Protocol::from(f.protocol),
                    completion_pct: f.completion_pct,
                    delay: f.delay,
                    score: f.score,
                    duration_stddev: f.duration_stddev,
                    last_sync: f.last_sync,
                    url: f.url.to_string(),
                })
                .collect_vec();

            let pos = self
                .selected_mirrors
                .iter()
                .positions(|f| f.country_code == country.code)
                .collect_vec();

            if pos.is_empty() {
                self.selected_mirrors.append(&mut mirrors)
            } else {
                let new_items = self
                    .selected_mirrors
                    .iter()
                    .filter_map(|f| {
                        if f.country_code != country.code {
                            Some(f.clone())
                        } else {
                            None
                        }
                    })
                    .collect_vec();

                self.selected_mirrors = new_items;
            }
        }
    }

    fn fragment_number(&self) -> usize {
        (self.scroll_pos / self.table_viewport_height as isize) as usize
    }
}

fn insert_character(app: &mut App, key: char) {
    app.input.insert(app.input_cursor_position, key);
    app.input_cursor_position += 1;
    app.scroll_pos = 0;
}

fn insert_filter(app: &mut App, filter: Protocol) -> AppReturn {
    let mut config = app.configuration.lock().unwrap();
    if let Some(idx) = config.filters.iter().position(|f| *f == filter) {
        debug!("protocol filter: removed {filter}");
        config.filters.remove(idx);
        app.show_insync = false;
    } else {
        debug!("protocol filter: added {filter}");
        config.filters.push(filter);
        app.show_insync = false;
    }
    app.scroll_pos = 0;
    AppReturn::Continue
}

fn insert_sort(app: &mut App, view: ViewSort) -> AppReturn {
    let mut config = app.configuration.lock().unwrap();
    config.view = view;
    AppReturn::Continue
}

fn write_to_file(
    config: Arc<Mutex<Configuration>>,
    mirrors: &[String],
    popup_state: Arc<Mutex<PopUpState>>,
    exporting: Arc<AtomicBool>,
) {
    let config = config.lock().unwrap();
    let outfile = &config.outfile;

    if let Some(dir) = outfile.parent() {
        if std::fs::create_dir_all(dir).is_ok() {
            let count = config.export as usize;
            let output = &mirrors[if mirrors.len() >= count {
                ..count
            } else {
                ..mirrors.len()
            }];
            match std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(outfile)
            {
                Ok(mut file) => {
                    for url in output.iter() {
                        if let Err(e) = writeln!(file, "{url}$repo/os/$arch") {
                            error!("{e}");
                        }
                    }
                    let mut state = popup_state.lock().unwrap();

                    state.popup_text = format!(
                        "Your mirrorlist has been successfully exported to: {}",
                        outfile.display()
                    );
                }
                Err(e) => {
                    error!("{e}");
                }
            }
        }
    }
    exporting.store(false, std::sync::atomic::Ordering::Relaxed);

    info!("Your mirrorlist has been exported");
}
