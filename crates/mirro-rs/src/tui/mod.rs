mod actions;
mod inputs;
mod io;
mod state;
mod ui;

use anyhow::Result;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use tui::{backend::CrosstermBackend, Terminal};

use self::{
    inputs::{event::Events, InputEvent},
    io::{handler::IoAsyncHandler, IoEvent},
    state::{App, AppReturn},
    ui::ui,
};

pub async fn start() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);
    let app = App::new(sync_io_tx);
    let app = Arc::new(Mutex::new(app));
    let inner = Arc::clone(&app);

    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(inner);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
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
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: Arc<Mutex<App>>,
) -> std::io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut events = Events::new(tick_rate);

    // Trigger state change from Init to Initialized
    {
        let mut app = app.lock().await;
        // Here we assume the the first load is a long task
        app.dispatch(IoEvent::Initialize).await;
    }

    loop {
        let mut app = app.lock().await;

        terminal.draw(|f| ui(f, &app))?;

        let result = match events.next().await {
            InputEvent::Input(key) => app.dispatch_action(key).await,
            InputEvent::Tick => app.update_on_tick().await,
        };

        if result == AppReturn::Exit {
            events.close();
            break;
        }
    }
    Ok(())
}
