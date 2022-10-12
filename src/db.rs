use reqwest::Url;
use secrecy::{ExposeSecret, Secret};
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};

use crate::settings::DatabaseSettings;

#[derive(Debug)]
pub struct DB {
    pub name: String,
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub ssl_mode: PgSslMode,
}

impl DB {
    /// The general form for `database_uri` is:
    ///
    /// `postgresql://[userspec@][hostspec][/dbname][?paramspec]`
    ///     where `userspec` is: `user[:password]`
    ///     and `hostspec` is: `[host][:port][,...]`
    ///     and `paramspec` is: `name=value[&...]`
    ///
    /// https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING
    pub fn from_url<S: Into<String> + Sized>(database_uri: S) -> Self {
        let url = Url::parse(&database_uri.into()).expect("Couldn't parse url");
        let Self {
            name,
            password,
            host,
            port,
            ssl_mode,
            ..
        } = Self::default();

        let name = url.path().strip_prefix('/').unwrap_or(&name).into();

        let password = Secret::new(
            url.password()
                .unwrap_or_else(|| password.expose_secret())
                .into(),
        );

        Self {
            name,
            username: url.username().into(),
            password,
            host: url.host_str().unwrap_or(&host).into(),
            port: url.port().unwrap_or(port),
            ssl_mode,
        }
    }

    pub fn set_ssl_mode(self, ssl_mode: PgSslMode) -> Self {
        Self { ssl_mode, ..self }
    }

    pub fn url(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name
        ))
    }

    pub fn url_without_db(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
        ))
    }

    pub fn connection_options_without_db(&self) -> PgConnectOptions {
        let mut options = PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port);

        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }

    pub fn connection_options(&self) -> PgConnectOptions {
        self.connection_options_without_db().database(&self.name)
    }
}

impl Default for DB {
    fn default() -> Self {
        Self {
            name: "".into(),
            username: "postgres".into(),
            password: Secret::new("".into()),
            host: "localhost".into(),
            port: 5432,
            ssl_mode: PgSslMode::Prefer,
        }
    }
}

impl From<&DatabaseSettings> for DB {
    fn from(database_settings: &DatabaseSettings) -> Self {
        Self::from_url(database_settings.url.expose_secret()).set_ssl_mode(
            if database_settings.require_ssl {
                PgSslMode::Require
            } else {
                PgSslMode::Prefer
            },
        )
    }
}
