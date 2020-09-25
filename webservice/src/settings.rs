use anyhow::Result;
use config::ConfigError;
use jsonwebtoken::{DecodingKey, EncodingKey};
use std::{collections::HashMap, ops::Deref};

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
}

#[derive(Debug, Deserialize)]
pub enum JWTVerification {
    HS256(String),
    RS256(String),
}

impl JWTVerification {
    pub fn create_encoding_key(&self) -> Result<EncodingKey> {
        let key = match &self {
            JWTVerification::HS256(secret) => {
                jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())
            }
            JWTVerification::RS256(public_key) => {
                jsonwebtoken::EncodingKey::from_rsa_pem(public_key.as_bytes())?
            }
        };
        Ok(key)
    }

    pub fn create_decoding_key(&self) -> Result<DecodingKey> {
        let key = match &self {
            JWTVerification::HS256(secret) => {
                jsonwebtoken::DecodingKey::from_secret(secret.as_bytes())
            }
            JWTVerification::RS256(public_key) => {
                jsonwebtoken::DecodingKey::from_rsa_pem(public_key.as_bytes())?
            }
        };
        Ok(key)
    }

    pub fn as_algorithm(&self) -> jsonwebtoken::Algorithm {
        match &self {
            JWTVerification::HS256(_) => jsonwebtoken::Algorithm::HS256,
            JWTVerification::RS256(_) => jsonwebtoken::Algorithm::RS256,
        }
    }
}

impl Default for JWTVerification {
    fn default() -> Self {
        JWTVerification::HS256("".to_string())
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Auth {
    pub token_verification: JWTVerification,
    pub expiration_minutes: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct LocalUser {
    pub password: String,
    #[serde(default)]
    pub corpus_groups: Vec<String>,
    #[serde(default)]
    pub admin: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct Settings {
    pub auth: Auth,
    pub database: Database,
    pub logging: Logging,
    pub bind: Bind,
    pub users: HashMap<String, LocalUser>,
}

impl Settings {
    pub fn with_file<S: Deref<Target = str>>(config_file: Option<S>) -> Result<Self, ConfigError> {
        let mut config = config::Config::default();

        // Use the included default configuration
        config.merge(config::File::from_str(
            include_str!("default-settings.toml",),
            config::FileFormat::Toml,
        ))?;

        // TODO: load from default locations

        if let Some(config_file) = config_file {
            config.merge(config::File::new(&config_file, config::FileFormat::Toml))?;
        }
        config.try_into()
    }
}
