use anyhow::Result;
use config::ConfigError;
use graphannis::corpusstorage::CacheStrategy;
use jsonwebtoken::DecodingKey;
use std::ops::Deref;

#[derive(Debug, Deserialize, Default)]
pub struct Logging {
    pub debug: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Bind {
    pub port: i16,
    pub host: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct Database {
    pub graphannis: String,
    pub sqlite: String,
    pub disk_based: bool,
    #[serde(default)]
    pub cache: CacheStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_timeout: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum JWTVerification {
    HS256 { secret: String },
    RS256 { public_key: String },
}

impl JWTVerification {
    pub fn create_decoding_key(&self) -> Result<DecodingKey> {
        let key = match &self {
            JWTVerification::HS256 { secret } => {
                jsonwebtoken::DecodingKey::from_secret(secret.as_bytes())
            }
            JWTVerification::RS256 { public_key } => {
                jsonwebtoken::DecodingKey::from_rsa_pem(public_key.as_bytes())?
            }
        };
        Ok(key)
    }

    pub fn as_algorithm(&self) -> jsonwebtoken::Algorithm {
        match &self {
            JWTVerification::HS256 { .. } => jsonwebtoken::Algorithm::HS256,
            JWTVerification::RS256 { .. } => jsonwebtoken::Algorithm::RS256,
        }
    }
}

impl Default for JWTVerification {
    fn default() -> Self {
        JWTVerification::HS256 {
            secret: "".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Auth {
    pub token_verification: JWTVerification,
    /// If true, all corpora can be accessed (read-only) without any authentication
    #[serde(default)]
    pub anonymous_access_all_corpora: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub auth: Auth,
    pub database: Database,
    pub logging: Logging,
    pub bind: Bind,
}

impl Settings {
    pub fn with_file<S: Deref<Target = str>>(config_file: Option<S>) -> Result<Self, ConfigError> {
        // Use the included default configuration
        let mut config = config::Config::builder().add_source(config::File::from_str(
            include_str!("default-settings.toml",),
            config::FileFormat::Toml,
        ));

        // TODO: load from default locations
        if let Some(config_file) = config_file {
            config = config.add_source(config::File::new(&config_file, config::FileFormat::Toml));
        }
        let config = config.build()?;
        Ok(config.try_deserialize()?)
    }
}
