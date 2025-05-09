use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web, App,
};
use diesel::{r2d2::ConnectionManager, SqliteConnection};
use diesel_migrations::MigrationHarness;
use graphannis::{corpusstorage::ImportFormat, CorpusStorage};
use insta::assert_snapshot;
use jsonwebtoken::EncodingKey;
use log::{Level, Log, RecordBuilder};
use tempfile::NamedTempFile;

use crate::{
    api::administration::BackgroundJobs,
    auth::Claims,
    create_logger,
    settings::{JWTVerification, Settings},
};

pub const JWT_SECRET: &str = "not-a-secret";

pub fn create_empty_dbpool() -> r2d2::Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let db_pool = r2d2::Pool::builder().build(manager).unwrap();
    let mut conn = db_pool.get().unwrap();
    conn.run_pending_migrations(crate::MIGRATIONS).unwrap();

    db_pool
}

pub fn create_test_app(
    cs: web::Data<CorpusStorage>,
    mut settings: Settings,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    // Create an app that uses a string as secret so we can sign our own JWT
    // token.
    settings.auth.token_verification = JWTVerification::HS256 {
        secret: JWT_SECRET.to_string(),
    };

    let db_pool = create_empty_dbpool();

    let settings = web::Data::new(settings);
    let db_pool = web::Data::new(db_pool);
    let background_jobs = web::Data::new(BackgroundJobs::default());

    let app = crate::create_app(cs, settings, db_pool, background_jobs);
    app
}

pub fn create_auth_header() -> (&'static str, String) {
    // Create an auth header for an admin
    let in_sixty_minutes = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .checked_add(Duration::from_secs(3600))
        .unwrap();
    let admin_claims = Claims {
        sub: "admin".to_string(),
        exp: Some(in_sixty_minutes.as_millis() as i64),
        roles: vec!["admin".to_string()],
        groups: vec![],
    };
    let bearer_token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &admin_claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .unwrap();
    ("Authorization", format!("Bearer {bearer_token}"))
}

pub(crate) fn import_test_corpora(cs: &CorpusStorage) {
    let cargo_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Import three corpora A,B and C
    cs.import_from_fs(
        &cargo_dir.join("../graphannis/tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        Some("A".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();

    cs.import_from_fs(
        &cargo_dir.join("../graphannis/tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        Some("B".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();

    cs.import_from_fs(
        &cargo_dir.join("../graphannis/tests/SaltSampleCorpus.graphml"),
        ImportFormat::GraphML,
        Some("C".into()),
        false,
        true,
        |_| {},
    )
    .unwrap();
}

fn standard_filter() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    // Remove any color ASCII codes
    settings.add_filter("\x1b", "");
    settings.add_filter("\\[[0-9]+m", "");

    // Filter out the time stamps
    settings.add_filter("[0-9]+:[0-9]+:[0-9]+ ", "12:00:00");
    // The loaded and also total available RAM size can vary
    settings.add_filter("[0-9.]+[MG]B / [0-9.]+[MG]B", "100MB / 300MB");
    // The loading and time can vary
    settings.add_filter("in [0-9]+ ms", "in 10 ms");
    settings
}

#[test]
fn test_logfile() -> Result<(), Box<dyn std::error::Error>> {
    let logfile = NamedTempFile::new()?;
    let mut settings = Settings::default();
    settings.logging.file = Some(logfile.path().to_string_lossy().to_string());

    // Get a logger
    let (logger, _) = create_logger(&settings)?;

    let record = RecordBuilder::new()
        .level(Level::Info)
        .args(format_args!("Hello World"))
        .build();
    logger.log(&record);

    let record = RecordBuilder::new()
        .level(Level::Debug)
        .args(format_args!("Debug Message"))
        .build();
    logger.log(&record);

    let logfile_content = std::fs::read_to_string(logfile.path())?;

    let snapshot_settings = standard_filter();
    snapshot_settings.bind(|| assert_snapshot!(logfile_content));

    Ok(())
}

#[test]
fn test_logfile_debug() -> Result<(), Box<dyn std::error::Error>> {
    let logfile = NamedTempFile::new()?;
    let mut settings = Settings::default();
    settings.logging.file = Some(logfile.path().to_string_lossy().to_string());
    settings.logging.debug = true;

    // Get a logger
    let (logger, _) = create_logger(&settings)?;

    let record = RecordBuilder::new()
        .level(Level::Info)
        .args(format_args!("Hello World"))
        .build();
    logger.log(&record);

    let record = RecordBuilder::new()
        .level(Level::Debug)
        .args(format_args!("Debug Message"))
        .build();
    logger.log(&record);

    let logfile_content = std::fs::read_to_string(logfile.path())?;

    let snapshot_settings = standard_filter();
    snapshot_settings.bind(|| assert_snapshot!(logfile_content));

    Ok(())
}
