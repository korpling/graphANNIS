#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

use actix_web::{dev::ServiceRequest, web, App, HttpServer};
use actix_web_httpauth::{
    extractors::bearer, extractors::AuthenticationError, middleware::HttpAuthentication,
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
mod models;
mod schema;
mod settings;

pub struct AppState {
    cs: graphannis::CorpusStorage,
    settings: settings::Settings,
    sqlite_pool: DbPool,
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
    let data_dir = std::path::PathBuf::from(&settings.database.graphannis);
    let cs = graphannis::CorpusStorage::with_auto_cache_size(&data_dir, true)?;

    // Add a connection pool to the SQLite database
    let sqlite_manager = ConnectionManager::<SqliteConnection>::new(&settings.database.sqlite);
    let sqlite_pool = r2d2::Pool::builder()
        .build(sqlite_manager)
        .expect("Failed to create pool.");

    Ok(AppState {
        cs,
        settings,
        sqlite_pool,
    })
}

async fn validator(
    req: ServiceRequest,
    credentials: bearer::BearerAuth,
) -> std::result::Result<ServiceRequest, actix_web::error::Error> {
    let bearer_config = req
        .app_data::<bearer::Config>()
        .map(|data| data.get_ref().clone())
        .unwrap_or_else(Default::default);
    let state = req
        .app_data::<AppState>()
        .ok_or_else(|| actix_web::error::Error::from(()))?;
    match api::auth::validate_token(credentials.token(), state) {
        Ok(res) => {
            if res == true {
                Ok(req)
            } else {
                Err(AuthenticationError::from(bearer_config).into())
            }
        }
        Err(_) => Err(AuthenticationError::from(bearer_config).into()),
    }
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
    let app_state = web::Data::new(app_state);

    // Run server
    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validator);

        App::new()
            .app_data(app_state.clone())
            .route("/local-login", web::post().to(api::auth::local_login))
            .service(
                web::resource("/search/count")
                    .route(web::get().to(api::search::count))
                    .wrap(auth.clone()),
            )
            .service(
                web::resource("/search/find")
                    .route(web::get().to(api::search::find))
                    .wrap(auth),
            )
    })
    .bind(bind_address)?
    .run()
    .await
}
