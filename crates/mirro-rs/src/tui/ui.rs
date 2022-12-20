use archlinux::ArchLinux;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};
use tui_logger::TuiLoggerWidget;

use super::{actions::Actions, state::App};

pub fn ui(f: &mut Frame<impl Backend>, app: &App) {
    let area = f.size();
    check_size(&area);

    let chunks = Layout::default()
        .constraints([Constraint::Min(20), Constraint::Length(3)].as_ref())
        .split(area);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(40)].as_ref())
        .split(chunks[0]);

    {
        // Body & Help
        let sidebar = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(body_chunks[1]);

        let help = draw_help(&app.actions);
        f.render_widget(help, sidebar[1]);

        let selection = Block::default()
            .title(title("Selection"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Black))
            .border_type(BorderType::Rounded);
        f.render_widget(selection, sidebar[0]);

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
        let table = draw_table(app.mirrors.as_ref());
        f.render_widget(table, body_chunks[0]);
    }

    if app.show_popup {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        let p = Paragraph::new("Preparing mirrors. Please wait...")
            .block(block)
            .alignment(Alignment::Center);
        let area = centered_rect(60, 20, area);
        f.render_widget(Clear, area);
        f.render_widget(p, area);
    }
}

fn draw_table(mirrors: Option<&ArchLinux>) -> Table {
    let header_cells = ["index", "country:", "mirrors:"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default()));
    let items: Vec<_> = if let Some(items) = mirrors {
        items
            .countries
            .iter()
            .enumerate()
            .map(|(idx, f)| {
                let mut item_name = format!("{}| {}", f.code, f.name);
                if item_name.is_empty() {
                    item_name = "other".to_string()
                }
                let index = format!("{idx}.");
                return Row::new(
                    [index, item_name, f.mirrors.len().to_string()]
                        .iter()
                        .map(|c| Cell::from(c.clone()).style(Style::default().fg(Color::Blue))),
                );
            })
            .collect()
    } else {
        vec![]
    };

    let count = items.len();
    let val = format!("Results from ({count}) countries");
    let header = Row::new(header_cells).height(1);

    let t = Table::new(items)
        .header(header)
        .block(
            Block::default()
                .title(title(val))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Black)),
        )
        .highlight_symbol(">>")
        .widths(&[
            Constraint::Percentage(4),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);

    t
}

fn draw_help(actions: &Actions) -> Table {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    for action in actions.actions().iter() {
        let mut first = true;
        for key in action.keys() {
            let help = if first {
                first = false;
                action.to_string()
            } else {
                String::from("")
            };
            let row = Row::new(vec![
                Cell::from(Span::styled(key.to_string(), key_style)),
                Cell::from(Span::styled(help, help_style)),
            ]);
            rows.push(row);
        }
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Black))
                .title(title("Help")),
        )
        .widths(&[Constraint::Length(11), Constraint::Min(20)])
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
        .output_target(false)
        .block(
            Block::default()
                .title(title("Logs"))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Black))
                .border_type(BorderType::Rounded),
        )
}

fn draw_filter(app: &App) -> Paragraph {
    Paragraph::new(app.input.as_ref()).block(Block::default().borders(Borders::ALL).title("Input"))
}

fn title(text: impl AsRef<str>) -> Spans<'static> {
    Spans::from(vec![Span::styled(
        format!(" {} ", text.as_ref()),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )])
}
