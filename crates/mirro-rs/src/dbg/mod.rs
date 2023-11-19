use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn log(skip_tui: bool) {
    let registry = tracing_subscriber::registry().with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "mirro_rs=debug".into()),
    );

    let err_fn = |e| {
        error!("couldn't connect to journald: {}", e);
    };

    match (tracing_journald::layer(), skip_tui) {
        (Ok(layer), true) => {
            registry
                .with(layer)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
        // journald is typically available on Linux systems, but nowhere else. Portable software
        // should handle its absence gracefully.
        (Err(e), true) => {
            registry.with(tracing_subscriber::fmt::layer()).init();
            err_fn(e);
        }
        (Ok(layer), false) => {
            registry
                .with(layer)
                .with(tui_logger::tracing_subscriber_layer())
                .init();
        }
        (Err(e), false) => {
            registry.with(tui_logger::tracing_subscriber_layer()).init();
            err_fn(e);
        }
    }

    let pkg_ver = env!("CARGO_PKG_VERSION");
    info!(version = pkg_ver, "mirro-rs has started");
}
