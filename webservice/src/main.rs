#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

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
    let db_pool = r2d2::Pool::builder().build(manager)?;

    info!(
        "Using database {}",
        PathBuf::from(&settings.database.sqlite)
            .canonicalize()?
            .to_string_lossy()
    );

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
            .wrap(Logger::default())
            .wrap(Compress::new(ContentEncoding::Gzip))
            .service(
                web::scope(&api_version)
                    .route("openapi.yml", web::get().to(get_api_spec))
                    .route("/local-login", web::post().to(api::auth::local_login))
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
                            .route("/frequency", web::post().to(api::search::frequency)),
                    )
                    .service(
                        web::scope("/corpora")
                            .route("", web::get().to(api::corpora::list))
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
                            .route("/{corpus}/files", web::get().to(api::corpora::files)),
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
