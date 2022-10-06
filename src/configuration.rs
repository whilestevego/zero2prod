use config::{Config, ConfigError, File};
// use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub url: String,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(File::with_name("configuration"))
        .build()
        .unwrap()
        .try_deserialize()
}
