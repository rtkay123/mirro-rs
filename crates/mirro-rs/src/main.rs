mod cli;
mod config;
mod tui;

#[tokio::main]
async fn main() {
    let args = <cli::Args as clap::Parser>::parse();

    let (config, _file) = config::read_config_file();

    if !check_outfile(&args) && !check_outfile(&config) {
        exit();
    }

    let outfile = args.outfile.unwrap_or_else(|| config.outfile.unwrap());
    let export = args.export.unwrap_or_else(|| config.export.unwrap());
    let filters = args.filters.unwrap_or_else(|| config.filters.unwrap());
    let view = args.view.unwrap_or_else(|| config.view.unwrap());
    let sort = args.sort.unwrap_or_else(|| config.sort.unwrap());
    let countries = args.country.unwrap_or_else(|| config.country.unwrap());
    let ttl = args.ttl.unwrap_or_else(|| config.ttl.unwrap());
    let url = args.url.unwrap_or_else(|| config.url.unwrap());

    let config =
        config::Configuration::new(outfile, export, filters, view, sort, countries, ttl, url);

    let _ = tui::start(config).await;
}

fn exit() -> ! {
    let cmd = clap::Command::new("mirro-rs");
    let mut err = clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(&cmd);
    err.insert(
        clap::error::ContextKind::InvalidArg,
        clap::error::ContextValue::String("--outfile".to_string()),
    );

    err.insert(
        clap::error::ContextKind::InvalidValue,
        clap::error::ContextValue::String(String::default()),
    );
    err.exit();
}

fn check_outfile(config: &cli::Args) -> bool {
    if let Some(ref outfile) = config.outfile {
        if outfile.to_string_lossy().ends_with('/') || outfile.to_string_lossy().is_empty() {
            exit();
        }
        true
    } else {
        false
    }
}
