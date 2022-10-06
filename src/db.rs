use reqwest::Url;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug)]
pub struct DB {
    pub name: String,
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
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
        let default_self = Self::default();

        let name = url
            .path()
            .strip_prefix('/')
            .unwrap_or(&default_self.name)
            .into();

        let password = Secret::new(
            url.password()
                .unwrap_or_else(|| default_self.password.expose_secret())
                .into(),
        );

        Self {
            name,
            username: url.username().into(),
            password,
            host: url.host_str().unwrap_or(&default_self.host).into(),
            port: url.port().unwrap_or(default_self.port),
        }
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
}

impl Default for DB {
    fn default() -> Self {
        Self {
            name: "".into(),
            username: "postgres".into(),
            password: Secret::new("".into()),
            host: "localhost".into(),
            port: 5432,
        }
    }
}
