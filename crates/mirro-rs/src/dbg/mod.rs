use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn log() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mirro_rs=debug".into()),
        )
        //    .with(tracing_subscriber::fmt::layer())
        .with(tui_logger::tracing_subscriber_layer())
        .init();
}
