#[macro_use]
extern crate log;

use actix_web::{App, HttpServer};
use clap::Arg;
use simplelog::{LevelFilter, SimpleLogger, TermLogger};
use std::io::{Error, ErrorKind, Result};

mod search;

struct AppState {
    cs: graphannis::CorpusStorage,
}

#[actix_rt::main]
async fn main() -> Result<()> {
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
                .takes_value(true)
                .default_value("graphannis-webservice.toml"),
        )
        .get_matches();

    // Load configuration file(s)
    let mut config = config::Config::default();
    if let Some(config_file_list) = matches.args.get("config") {
        for config_file in &config_file_list.vals {
            config
                .merge(config::File::new(
                    config_file.to_string_lossy().as_ref(),
                    config::FileFormat::Toml,
                ))
                .map_err(|e| {
                    Error::new(
                        ErrorKind::Other,
                        format!("Could not add configuration file: {:?}", e),
                    )
                })?;
        }
    }

    let log_filter = if config.get_bool("logging.debug").unwrap_or(false) {
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
    let cs = graphannis::CorpusStorage::with_auto_cache_size(&data_dir, true).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Could not open corpus storage: {:?}", e),
        )
    })?;
    let app_state = actix_web::web::Data::new(AppState { cs });

    // Run server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(search::count)
    })
    .bind("127.0.0.1:5711")?
    .run()
    .await
}
