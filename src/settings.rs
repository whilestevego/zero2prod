use std::{env, time::Duration};

use config::{Config, ConfigError, Environment, File};
use secrecy::Secret;
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::domain::SubscriberEmail;

#[derive(serde::Deserialize, Debug)]
pub enum Env {
    Local,
    Production,
}

impl Env {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl From<&str> for Env {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" => Self::Production,
            _ => Self::Local,
        }
    }
}

impl From<String> for Env {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<Result<String, env::VarError>> for Env {
    fn from(s: Result<String, env::VarError>) -> Self {
        s.unwrap_or_else(|_| "".into()).into()
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Debug)]
pub struct ApplicationSettings {
    env: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

impl ApplicationSettings {
    pub fn env(&self) -> Env {
        self.env.as_str().into()
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn sender_email(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub url: Secret<String>,
    pub require_ssl: bool,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let base_path = env::current_dir().expect("Failed to determine the current directory");
        let config_dir = base_path.join("config");
        let app_env: Env = env::var("APPLICATION_ENV").into();

        Config::builder()
            .add_source(File::from(config_dir.join("base")).required(true))
            .add_source(File::from(config_dir.join(app_env.as_str())).required(true))
            .add_source(Environment::default().separator("_"))
            .set_override("application.env", app_env.as_str())
            .unwrap()
            .build()
            .unwrap()
            .try_deserialize()
    }
}
