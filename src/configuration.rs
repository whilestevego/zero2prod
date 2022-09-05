use config::{Config, ConfigError, File};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: Database,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct Database {
    pub url: String,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(File::with_name("configuration"))
        .build()
        .unwrap()
        .try_deserialize()
}
