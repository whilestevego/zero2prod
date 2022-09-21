use config::{Config, ConfigError, File};
use reqwest::Url;

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

impl Database {
    pub fn url(&self) -> Url {
        Url::parse(&self.url).expect("Invalid database URL")
    }

    pub fn name(&self) -> String {
        self.url()
            .path()
            .strip_prefix("/")
            .unwrap_or("")
            .to_string()
    }

    pub fn url_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.url().username(),
            self.url().password().unwrap_or(""),
            self.url().host_str().unwrap_or(""),
            self.url()
                .port()
                .map(|x| x.to_string())
                .unwrap_or("".to_string()),
        )
    }
}
