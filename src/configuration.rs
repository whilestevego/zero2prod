use std::env;

use config::{Config, ConfigError, File};
use secrecy::Secret;
// use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub url: Secret<String>,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(File::with_name("configuration"))
        .set_override_option("database.url", env::var("DATABASE_URL").ok())
        .unwrap()
        .build()
        .unwrap()
        .try_deserialize()
}
