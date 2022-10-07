use std::env;

use config::{Config, ConfigError, File};
use secrecy::Secret;
// use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub url: Secret<String>,
}

pub enum AppEnv {
    Local,
    Production,
}

impl AppEnv {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for AppEnv {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            env_name => Err(format!(
                "{env_name} is not a supported environment. Use either `local` or `production`."
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = env::current_dir().expect("Failed to determine the current directory");
    let config_dir = base_path.join("config");
    let app_env: AppEnv = env::var("APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENV.");

    Config::builder()
        .add_source(File::from(config_dir.join("base")).required(true))
        .add_source(File::from(config_dir.join(app_env.as_str())).required(true))
        .set_override_option("database.url", env::var("DATABASE_URL").ok())
        .unwrap()
        .build()
        .unwrap()
        .try_deserialize()
}
