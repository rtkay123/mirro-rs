use std::{
    sync::{atomic::AtomicBool, mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

use archlinux::{DateTime, Local, Mirror};

use itertools::Itertools;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Clear, Gauge, Paragraph, Row, Table},
    Frame,
};
use tui_logger::TuiLoggerWidget;

use super::{
    actions::{Action, Actions},
    dispatch::{filter::Protocol, sort::ViewSort},
    state::{App, PopUpState},
};

pub fn ui(
    f: &mut Frame<impl Backend>,
    app: &mut App,
    popup: Arc<Mutex<PopUpState>>,
    exporting: Arc<AtomicBool>,
    percentage: &Receiver<f32>,
) {
    let area = f.size();
    check_size(&area);

    let chunks = Layout::default()
        .constraints([Constraint::Min(20), Constraint::Length(3)].as_ref())
        .split(area);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(60)].as_ref())
        .split(chunks[0]);

    {
        // Body & Help
        let sidebar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(body_chunks[1]);

        let help = draw_help(&app.actions);
        f.render_widget(help, sidebar[1]);

        f.render_widget(draw_selection(app), sidebar[0]);

        match app.show_input {
            true => {
                f.render_widget(draw_filter(app), chunks[1]);
                f.set_cursor(
                    // Put cursor past the end of the input text
                    chunks[1].x + app.input_cursor_position as u16 + 1,
                    // Move one line down, from the border to the input line
                    chunks[1].y + 1,
                )
            }
            false => f.render_widget(draw_logs(), chunks[1]),
        };
    }

    {
        let content_bar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(20)].as_ref())
            .split(body_chunks[0]);

        f.render_widget(draw_sort(app), content_bar[0]);

        draw_table(app, f, content_bar[1]);
    }

    let p = {
        let popup_state = popup.lock().unwrap();
        Paragraph::new(popup_state.popup_text.clone())
    };

    if app.show_popup.load(std::sync::atomic::Ordering::Relaxed) {
        let rate_enabled = {
            let state = app.configuration.lock().unwrap();
            state.rate
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        let p = p.block(block).alignment(Alignment::Center);
        let area = centered_rect(60, 20, area);
        f.render_widget(Clear, area);
        if exporting.load(std::sync::atomic::Ordering::Relaxed) && rate_enabled {
            while let Ok(pos) = percentage.try_recv() {
                log::info!("exporting mirrors: progress {pos:.2}%");
                let gauge = Gauge::default()
                    .gauge_style(Style::default().fg(Color::Blue).bg(Color::Black))
                    .block(create_block("Exporting mirrors"))
                    .percent(pos as u16);
                f.render_widget(gauge, area);
            }
        } else {
            f.render_widget(p, area);
        }
    }
}

fn draw_table(app: &mut App, f: &mut Frame<impl Backend>, region: Rect) {
    let header_cells = ["  index", "╭─── country", "mirrors"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default()));

    if let Some(items) = app.mirrors.as_ref() {
        app.filtered_countries = items
            .countries
            .iter()
            .filter_map(|f| {
                let count = f.mirrors.iter().filter(|m| filter_result(app, m)).count();
                if count == 0 {
                    None
                } else if f
                    .name
                    .to_ascii_lowercase()
                    .contains(&app.input.to_ascii_lowercase())
                {
                    Some((f.clone(), count))
                } else {
                    None
                }
            })
            .sorted_by(|(f, count), (b, second_count)| {
                let config = app.configuration.lock().unwrap();
                match config.view {
                    ViewSort::Alphabetical => Ord::cmp(&f.name, &b.name),
                    ViewSort::MirrorCount => Ord::cmp(&second_count, &count),
                }
            })
            .collect_vec();
    };

    // 3 is the height offset
    app.table_viewport_height = region.height - 3;

    let rows = app.rows();

    let pagination_fragments = app.view_fragments(&rows);

    let header = Row::new(header_cells).height(1);

    let t = Table::new(if pagination_fragments.is_empty() {
        rows
    } else {
        app.view(&pagination_fragments).to_vec()
    })
    .header(header)
    .block(create_block(format!(
        "Results from ({}) countries",
        app.filtered_countries.len()
    )))
    .widths(&[
        Constraint::Percentage(6),
        Constraint::Length(33),
        Constraint::Min(10),
    ]);

    f.render_widget(t, region);
}

fn draw_help(actions: &Actions) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let rows = actions.actions().iter().filter_map(|action| match action {
        Action::NavigateUp | Action::NavigateDown => None,
        _ => {
            let mut actions: Vec<_> = action
                .keys()
                .iter()
                .map(|k| Span::styled(k.to_string(), key_style))
                .collect();

            if actions.len() == 1 {
                actions.push(Span::raw(""));
            }

            let text = Span::styled(action.to_string(), help_style);
            actions.push(text);
            Some(Row::new(actions))
        }
    });

    Table::new(rows)
        .block(create_block("Help"))
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(60),
        ])
        .column_spacing(1)
}

fn check_size(area: &Rect) {
    if area.width < 52 {
        panic!("Require width >= 52, (got {})", area.width);
    }
    if area.height < 28 {
        panic!("Require height >= 28, (got {})", area.height);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn draw_logs<'a>() -> TuiLoggerWidget<'a> {
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Blue))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Magenta))
        .style_info(Style::default().fg(Color::Green))
        .output_file(false)
        .output_timestamp(None)
        .output_line(false)
        .output_target(false)
        .block(create_block("Logs"))
}

fn draw_filter(app: &App) -> Paragraph {
    Paragraph::new(app.input.as_ref()).block(create_block("Filter"))
}

fn draw_selection<'a>(app: &App) -> Table<'a> {
    let header_cells = ["code", "proto", "comp %", "delay", "dur", "score"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default()));
    let headers = Row::new(header_cells);

    let items = app.selected_mirrors.iter().map(|f| {
        let delay = f.delay.map(|f| {
            let duration = Duration::from_secs(f as u64);
            let minutes = (duration.as_secs() / 60) % 60;
            let hours = (duration.as_secs() / 60) / 60;
            (hours, minutes)
        });

        let score = f.score.map(format_float);

        let dur = f.duration_stddev.map(format_float);

        let completion = f.completion_pct;

        Row::new(vec![
            Cell::from(f.country_code.to_string()),
            Cell::from(f.protocol.to_string()),
            Cell::from(format!("{:.2}", (completion * 100.0))).style(if completion == 1.0 {
                Style::default().fg(Color::Green)
            } else if completion > 0.90 {
                Style::default().fg(Color::LightCyan)
            } else if completion > 0.80 {
                Style::default().fg(Color::Cyan)
            } else if completion > 0.70 {
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::SLOW_BLINK)
            } else if completion > 0.60 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::SLOW_BLINK)
            } else if completion > 0.50 {
                Style::default()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::SLOW_BLINK)
            } else {
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::SLOW_BLINK)
            }),
            Cell::from(match delay {
                Some((hours, minutes)) => {
                    format!("{hours}:{minutes}")
                }
                None => "-".to_string(),
            })
            .style(match delay {
                Some((hours, _)) => {
                    if hours < 1 {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    }
                }
                None => Style::default(),
            }),
            Cell::from(
                dur.map(|f| f.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::from(
                score
                    .map(|f| f.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            ),
        ])
    });

    let mirror_count = app.selected_mirrors.len();
    let config = app.configuration.lock().unwrap();

    let t = Table::new(items)
        // You can set the style of the entire Table.
        .style(Style::default().fg(Color::White))
        // It has an optional header, which is simply a Row always visible at the top.
        .header(headers)
        // As any other widget, a Table can be wrapped in a Block.
        .block(create_block(if mirror_count < 1 {
            format!("Selection({mirror_count})")
        } else {
            format!(
                "Selection({})▶ ({}) to {}",
                mirror_count,
                if config.export as usize <= mirror_count {
                    config.export.to_string()
                } else {
                    "ALL".to_string()
                },
                config.outfile.display()
            )
        }))
        // Columns widths are constrained in the same way as Layout...
        .widths(&[
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(16),
            Constraint::Percentage(20),
        ]);
    // ...and they can be separated by a fixed spacing.

    t
}

fn draw_sort<'a>(app: &App) -> Paragraph<'a> {
    let config = app.configuration.lock().unwrap();
    let count: isize = config.filters.len() as isize;
    let active_sort = vec![config.view];
    let mut sorts: Vec<_> = active_sort
        .iter()
        .enumerate()
        .flat_map(|(idx, f)| {
            let mut ret = vec![
                Span::raw(format!(" [{f}]")),
                Span::styled(" ⇣", Style::default()),
            ];
            if (idx as isize) < count - 1 {
                ret.push(Span::styled(" 🢒", Style::default().fg(Color::Black)))
            }
            ret
        })
        .collect();

    let mut filters: Vec<_> = config
        .filters
        .iter()
        .enumerate()
        .flat_map(|(idx, f)| {
            let mut ret = vec![Span::styled(
                format!(" {f}"),
                Style::default()
                    .fg(match f {
                        Protocol::InSync => Color::Cyan,
                        _ => Color::Blue,
                    })
                    .add_modifier(Modifier::BOLD),
            )];
            if (idx as isize) < count - 1 {
                ret.push(Span::styled(" 🢒", Style::default().fg(Color::Black)))
            }
            ret
        })
        .collect();

    sorts.append(&mut filters);

    let widget = Spans::from(sorts);

    let bt = format!("Sort & Filter ({count})");

    Paragraph::new(widget).block(create_block(bt))
}

fn create_block<'a>(title: impl Into<String>) -> Block<'a> {
    let title = title.into();
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Black))
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::White),
        ))
}

fn format_float(str: impl ToString) -> f32 {
    match str.to_string().parse::<f32>() {
        Ok(res) => (res * 100.0).round() / 100.0,
        Err(_) => -999.0,
    }
}

pub fn filter_result(app: &App, f: &Mirror) -> bool {
    use crate::config::Configuration;
    let mut config = app.configuration.lock().unwrap();

    let res = |config: &Configuration, f: &Mirror| {
        let mut completion_ok = config.completion_percent as f32 <= f.completion_pct * 100.0;
        let v4_on = config.filters.contains(&Protocol::Ipv4);
        let isos_on = config.filters.contains(&Protocol::Isos);
        let v6_on = config.filters.contains(&Protocol::Ipv6);
        if v4_on {
            completion_ok = completion_ok && f.ipv4;
        }

        if isos_on {
            completion_ok = completion_ok && f.isos;
        }

        if v6_on {
            completion_ok = completion_ok && f.ipv6;
        }
        completion_ok
    };

    if config.age != 0 {
        if let Some(mirror_sync) = f.last_sync {
            let now = Local::now();
            let mirror_sync: DateTime<Local> = DateTime::from(mirror_sync);
            let duration = now - mirror_sync;
            if !config.filters.contains(&Protocol::InSync) && app.show_insync {
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
