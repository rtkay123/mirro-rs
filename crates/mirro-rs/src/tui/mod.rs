mod state;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, sync::Arc};
use tokio::sync::Mutex;

use tui::{backend::CrosstermBackend, Terminal};

use self::{state::App, ui::ui};

pub async fn start() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new();
    let app = Arc::new(Mutex::new(app));
    let inner = Arc::clone(&app);
    tokio::spawn(async move {
        println!("going");
        let mut app = inner.lock().await;
        let x = app.initialise().await;
    });

    let res = run_app(&mut terminal, Arc::clone(&app)).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err:?}")
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: Arc<Mutex<App>>,
) -> io::Result<()> {
    loop {
        let guard = app.lock().await;
        terminal.draw(|f| ui(f, &guard))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('p') => {
                    let mut app = app.lock().await;
                    app.show_popup = !app.show_popup
                }
                _ => {}
            }
        }
    }
}
