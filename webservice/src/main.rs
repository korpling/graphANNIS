#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

use actix_web::{App, HttpServer};
use clap::Arg;
use simplelog::{LevelFilter, SimpleLogger, TermLogger};
use std::io::{Error, ErrorKind, Result};

mod search;
mod settings;

struct AppState {
    cs: graphannis::CorpusStorage,
    settings: settings::Settings,
}

fn init_app() -> anyhow::Result<AppState> {
    // Parse CLI arguments
    let matches = clap::App::new("graphANNIS web service")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Web service line interface to graphANNIS.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Configuration file location")
                .takes_value(true),
        )
        .get_matches();

    // Load configuration file(s)
    let settings = settings::Settings::with_file(matches.value_of_lossy("config"))?;

    let log_filter = if settings.logging.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    let log_config = simplelog::ConfigBuilder::new()
        .add_filter_ignore_str("rustyline:")
        .build();

    if let Err(e) = TermLogger::init(
        log_filter,
        log_config.clone(),
        simplelog::TerminalMode::Mixed,
    ) {
        println!("Error, can't initialize the terminal log output: {}.\nWill degrade to a more simple logger", e);
        if let Err(e_simple) = SimpleLogger::init(log_filter, log_config) {
            println!("Simple logging failed too: {}", e_simple);
        }
    }

    info!("Logging with level {}", log_filter);

    // Create a graphANNIS corpus storage as shared state
    let data_dir = std::path::PathBuf::from("data/");
    let cs = graphannis::CorpusStorage::with_auto_cache_size(&data_dir, true)?;
    Ok(AppState { cs, settings })
}

#[actix_rt::main]
async fn main() -> Result<()> {
    // Initialize application and its state
    let app_state = init_app().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Could not initialize graphANNIS service: {:?}", e),
        )
    })?;

    let bind_address = format!(
        "{}:{}",
        &app_state.settings.bind.host, &app_state.settings.bind.port
    );
    let app_state = actix_web::web::Data::new(app_state);

    // Run server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(search::count)
    })
    .bind(bind_address)?
    .run()
    .await
}
