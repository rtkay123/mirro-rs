use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::state::App;

pub fn ui(f: &mut Frame<impl Backend>, app: &App) {
    let area = f.size();

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(10), Constraint::Percentage(80)].as_ref())
        .split(area);

    let text = if app.show_popup {
        "Loading mirrors"
    } else {
        "Mirrors loaded"
    };

    let paragraph = Paragraph::new(Span::styled(
        text,
        Style::default().add_modifier(Modifier::SLOW_BLINK),
    ))
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[0]);

    // More content here

    if app.show_popup {
        let block = Block::default().title("Mirrors").borders(Borders::ALL);
        let area = centered_rect(60, 20, area);
        f.render_widget(Clear, area);
        f.render_widget(block, area);
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
