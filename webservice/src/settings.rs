use config::ConfigError;
use std::{collections::HashMap, ops::Deref};

#[derive(Debug, Deserialize)]
pub struct Logging {
    pub debug: bool,
}

#[derive(Debug, Deserialize)]
pub struct Bind {
    pub port: i16,
    pub host: String,
}

#[derive(Debug, Deserialize)]
pub struct LocalUser {
    pub password: String,
    #[serde(default)]
    pub corpus_groups: Vec<String>,
    #[serde(default)]
    pub admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
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
