mod actions;
mod inputs;
mod io;
mod state;
mod ui;
pub mod view;

use anyhow::Result;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, LevelFilter};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tokio::sync::Mutex;

use tui::{backend::CrosstermBackend, Terminal};

use crate::config::Configuration;

use self::{
    inputs::{event::Events, InputEvent},
    io::{handler::IoAsyncHandler, IoEvent},
    state::{App, AppReturn, PopUpState},
    ui::ui,
};

pub async fn start(configuration: Arc<std::sync::Mutex<Configuration>>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);
    let app = Arc::new(Mutex::new(App::new(sync_io_tx, Arc::clone(&configuration))));
    let inner = Arc::clone(&app);

    tui_logger::init_logger(LevelFilter::Trace).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    let popup_state = Arc::new(Mutex::new(PopUpState::new()));
    {
        let popup_state = Arc::clone(&popup_state);
        tokio::spawn(async move {
            let mut handler = IoAsyncHandler::new(inner, popup_state);
            debug!("Getting Arch Linux mirrors. Please wait");
            while let Some(io_event) = sync_io_rx.recv().await {
                handler
                    .handle_io_event(io_event, Arc::clone(&configuration))
                    .await;
            }
        });
    }
    let res = run_app(&mut terminal, Arc::clone(&app), popup_state).await;

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
    popup_state: Arc<Mutex<PopUpState>>,
) -> std::io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut events = Events::new(tick_rate);

    // Trigger state change from Init to Initialised
    {
        let mut app = app.lock().await;
        // Here we assume the the first load is a long task
        app.dispatch(IoEvent::Initialise).await;
    }

    let exporting = Arc::new(AtomicBool::new(false));
    let (pos_tx, pos_rx) = std::sync::mpsc::channel();

    loop {
        let mut app = app.lock().await;
        let popup = popup_state.lock().await;

        terminal.draw(|f| ui(f, &mut app, &popup, Arc::clone(&exporting), &pos_rx))?;

        let result = match events.next().await {
            InputEvent::Input(key) => {
                app.dispatch_action(key, Arc::clone(&exporting), pos_tx.clone())
                    .await
            }
            InputEvent::Tick => app.update_on_tick().await,
        };

        if result == AppReturn::Exit {
            events.close();
            break;
        }
    }
    Ok(())
}
