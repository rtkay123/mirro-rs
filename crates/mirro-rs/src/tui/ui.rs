use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tui_logger::TuiLoggerWidget;

use super::state::App;

pub fn ui(f: &mut Frame<impl Backend>, app: &App) {
    let area = f.size();
    check_size(&area);

    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Min(10),
                Constraint::Length(3),
                Constraint::Min(12),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(area);

    let input = Paragraph::new(app.input.as_ref())
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(draw_logs(), chunks[3]);

    // More content here

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
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue))
        .block(Block::default().title("Logs").borders(Borders::ALL))
}
