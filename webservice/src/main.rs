#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use actix_cors::Cors;
use actix_web::{
    http::{self, ContentEncoding},
    middleware::{Compress, Logger},
    web, App, HttpRequest, HttpServer,
};
use administration::BackgroundJobs;
use api::administration;
use clap::Arg;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use simplelog::{LevelFilter, SimpleLogger, TermLogger};
use std::{
    io::{Error, ErrorKind, Result},
    path::PathBuf,
};

mod actions;
mod api;
mod auth;
mod errors;
mod extractors;
mod models;
mod schema;
mod settings;

embed_migrations!("migrations");
type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

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

    let mut log_config = simplelog::ConfigBuilder::new();
    log_config.add_filter_ignore_str("rustyline:");
    if settings.logging.debug {
        warn!("Enabling request logging to console in debug mode");
    } else {
        log_config.add_filter_ignore_str("actix_web:");
    }

    let log_config = log_config.build();

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
    let cs = graphannis::CorpusStorage::with_cache_strategy(
        &data_dir,
        settings.database.cache.clone(),
        true,
    )?;

    // Add a connection pool to the SQLite database

    let manager = ConnectionManager::<SqliteConnection>::new(&settings.database.sqlite);
    let db_pool = r2d2::Pool::builder().build(manager)?;

    // Make sure the database has all migrations applied
    let conn = db_pool.get()?;
    embedded_migrations::run(&conn)?;

    info!(
        "Using database {} with at most {} of RAM for the corpus cache.",
        PathBuf::from(&settings.database.sqlite)
            .canonicalize()?
            .to_string_lossy(),
        &settings.database.cache
    );
    if let Some(timeout) = &settings.database.query_timeout {
        info!("Queries timeout set to {} seconds", timeout);
    }

    Ok((cs, settings, db_pool))
}

async fn get_api_spec(_req: HttpRequest) -> web::HttpResponse {
    web::HttpResponse::Ok()
        .content_type("application/x-yaml")
        .body(include_str!("openapi.yml"))
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

    // Create a list of background jobs behind a Mutex
    let background_jobs = web::Data::new(BackgroundJobs::default());

    let api_version = format!("/v{}", env!("CARGO_PKG_VERSION_MAJOR"),);

    // Run server
    HttpServer::new(move || {
        let logger = if settings.logging.debug {
            // Log all requests in debug
            Logger::default()
        } else {
            Logger::default().exclude_regex(".*")
        };

        App::new()
            .wrap(
                Cors::new()
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .finish(),
            )
            .app_data(cs.clone())
            .app_data(settings.clone())
            .app_data(db_pool.clone())
            .app_data(background_jobs.clone())
            .wrap(logger)
            .wrap(Compress::new(ContentEncoding::Gzip))
            .service(
                web::scope(&api_version)
                    .route("openapi.yml", web::get().to(get_api_spec))
                    .route(
                        "/import",
                        web::post().to(api::administration::import_corpus),
                    )
                    .route(
                        "/export",
                        web::post().to(api::administration::export_corpus),
                    )
                    .route("/jobs/{uuid}", web::get().to(api::administration::jobs))
                    .service(
                        web::scope("/search")
                            .route("/count", web::post().to(api::search::count))
                            .route("/find", web::post().to(api::search::find))
                            .route("/frequency", web::post().to(api::search::frequency))
                            .route(
                                "/node-descriptions",
                                web::get().to(api::search::node_descriptions),
                            ),
                    )
                    .service(
                        web::scope("/corpora")
                            .route("", web::get().to(api::corpora::list))
                            .route("/{corpus}", web::delete().to(api::corpora::delete))
                            .route(
                                "/{corpus}/configuration",
                                web::get().to(api::corpora::configuration),
                            )
                            .route(
                                "/{corpus}/node-annotations",
                                web::get().to(api::corpora::node_annotations),
                            )
                            .route(
                                "/{corpus}/components",
                                web::get().to(api::corpora::list_components),
                            )
                            .route(
                                "/{corpus}/edge-annotations/{type}/{layer}/{name}/",
                                web::get().to(api::corpora::edge_annotations),
                            )
                            .route("/{corpus}/subgraph", web::post().to(api::corpora::subgraph))
                            .route(
                                "/{corpus}/subgraph-for-query",
                                web::get().to(api::corpora::subgraph_for_query),
                            )
                            .route(
                                "/{corpus}/files/{name}",
                                web::get().to(api::corpora::file_content),
                            )
                            .route("/{corpus}/files", web::get().to(api::corpora::list_files)),
                    )
                    .service(
                        web::scope("/groups")
                            .route("", web::get().to(administration::list_groups))
                            .route("/{name}", web::delete().to(administration::delete_group))
                            .route("/{name}", web::put().to(administration::put_group)),
                    ),
            )
    })
    .bind(bind_address)?
    .run()
    .await
}
