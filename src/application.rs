use std::{io, net::TcpListener, ops::Deref};

use crate::{
    db::DB,
    email_client::EmailClient,
    routes::{confirm, health_check, home, publish_newsletter, subscribe},
    settings::{ApplicationSettings, Settings},
};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};

use tracing_actix_web::TracingLogger;

#[derive(Debug)]
pub struct ApplicationBuilder {
    settings: Settings,
    db_pool: Option<PgPool>,
    email_client: Option<EmailClient>,
    tcp_listener: Option<TcpListener>,
}

impl ApplicationBuilder {
    pub fn from_settings(settings: Settings) -> Self {
        Self {
            settings,
            db_pool: None,
            email_client: None,
            tcp_listener: None,
        }
    }

    pub fn set_db_pool(mut self, db_pool: PgPool) -> Self {
        self.db_pool = Some(db_pool);
        self
    }

    pub fn set_email_client(mut self, email_client: EmailClient) -> Self {
        self.email_client = Some(email_client);
        self
    }

    pub fn set_tcp_listener(mut self, tcp_listener: TcpListener) -> Self {
        self.tcp_listener = Some(tcp_listener);
        self
    }

    pub fn build(self) -> Application {
        let Self {
            settings,
            db_pool,
            email_client,
            tcp_listener,
        } = self;

        let db_pool = db_pool.unwrap_or_else(|| {
            let db: DB = (&settings.database).into();

            PgPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_secs(2))
                .connect_lazy_with(db.connection_options())
        });

        let email_client = email_client.unwrap_or_else(|| {
            let email_client = &settings.email_client;
            EmailClient::new(
                email_client.base_url.clone(),
                settings
                    .email_client
                    .sender_email()
                    .expect("Invalid sender email address."),
                email_client.authorization_token.clone(),
                email_client.timeout(),
            )
        });

        let tcp_listener = tcp_listener.unwrap_or_else(|| {
            let Settings {
                application:
                    ApplicationSettings {
                        ref host, ref port, ..
                    },
                ..
            } = settings;

            let address = format!("{host}:{port}");

            TcpListener::bind(&address).unwrap_or_else(|e: std::io::Error| {
                panic!(
                    "Couldn't bind TCP listener to {}, because \"{}\"",
                    &address, e
                )
            })
        });

        let base_url = settings.application.base_url;

        Application {
            base_url,
            port: tcp_listener.local_addr().unwrap().port(),
            db_pool,
            email_client,
            tcp_listener,
        }
    }
}

pub struct ApplicationBaseUrl(pub String);

impl Deref for ApplicationBaseUrl {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Application {
    base_url: String,
    port: u16,
    db_pool: PgPool,
    tcp_listener: TcpListener,
    email_client: EmailClient,
}

impl Application {
    pub fn builder_from_settings(settings: Settings) -> ApplicationBuilder {
        ApplicationBuilder::from_settings(settings)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.run()?.await
    }

    pub fn run(self) -> Result<Server, io::Error> {
        let Self {
            base_url,
            tcp_listener,
            db_pool,
            email_client,
            ..
        } = self;

        let base_url = web::Data::new(ApplicationBaseUrl(base_url));
        let db_pool = web::Data::new(db_pool);
        let email_client = web::Data::new(email_client);

        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/newsletters", web::post().to(publish_newsletter))
                .route("/health_check", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .route("/subscriptions/confirm", web::get().to(confirm))
                .route("/", web::get().to(home))
                .app_data(base_url.clone())
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
        })
        .listen(tcp_listener)?
        .run();

        Ok(server)
    }
}
