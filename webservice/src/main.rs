#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

use actix_web::{
    middleware::{Compress, Logger},
    web, App, HttpServer,
};
use clap::Arg;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use simplelog::{LevelFilter, SimpleLogger, TermLogger};
use std::io::{Error, ErrorKind, Result};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

mod actions;
mod api;
mod errors;
mod extractors;
mod models;
mod schema;
mod settings;

fn init_app() -> anyhow::Result<(graphannis::CorpusStorage, settings::Settings, DbPool)> {
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
    let data_dir = std::path::PathBuf::from(&settings.database.graphannis);
    let cs = graphannis::CorpusStorage::with_auto_cache_size(&data_dir, true)?;

    // Add a connection pool to the SQLite database
    let manager = ConnectionManager::<SqliteConnection>::new(&settings.database.sqlite);
    let db_pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    Ok((cs, settings, db_pool))
}

#[actix_rt::main]
async fn main() -> Result<()> {
    // Initialize application and its state
    let (cs, settings, db_pool) = init_app().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Could not initialize graphANNIS service: {:?}", e),
        )
    })?;

    let bind_address = format!("{}:{}", &settings.bind.host, &settings.bind.port);
    let cs = web::Data::new(cs);
    let settings = web::Data::new(settings);
    let db_pool = web::Data::new(db_pool);

    // Run server
    HttpServer::new(move || {
        App::new()
            .app_data(cs.clone())
            .app_data(settings.clone())
            .app_data(db_pool.clone())
            .wrap(Logger::default())
            .wrap(Compress::default())
            .route("/local-login", web::post().to(api::auth::local_login))
            .route("/search/count", web::post().to(api::search::count))
            .route("/search/find", web::post().to(api::search::find))
            .route("/corpora", web::get().to(api::corpora::list))
            .route(
                "/corpora/{corpus}/configuration",
                web::get().to(api::corpora::configuration),
            )
            .route(
                "/corpora/{corpus}/node_annotations",
                web::get().to(api::corpora::node_annotations),
            )
            .route(
                "/corpora/{corpus}/components",
                web::get().to(api::corpora::list_components),
            )
            .route(
                "/corpora/{corpus}/edge_annotations",
                web::get().to(api::corpora::edge_annotations),
            )
            .route(
                "/corpora/{corpus}/subgraph",
                web::get().to(api::corpora::subgraph),
            )
            .route(
                "/corpora/{corpus}/files/{filename:.*}",
                web::get().to(api::corpora::files),
            )
    })
    .bind(bind_address)?
    .run()
    .await
}
