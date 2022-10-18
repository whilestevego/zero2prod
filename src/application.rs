use std::{io, net::TcpListener};

use crate::{
    db::DB,
    email_client::EmailClient,
    routes::{health_check, subscribe},
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

            TcpListener::bind(&address)
                .unwrap_or_else(|_| panic!("Couldn't bind TCP listener to {}", &address))
        });

        Application {
            db_pool,
            email_client,
            tcp_listener,
        }
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self {
            // TODO: Don't impl default for an operation that may fail
            settings: Settings::load().expect("Failed to read configuration"),
            db_pool: None,
            email_client: None,
            tcp_listener: None,
        }
    }
}
pub struct Application {
    db_pool: PgPool,
    tcp_listener: TcpListener,
    email_client: EmailClient,
}

impl Application {
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::default()
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.run()?.await
    }

    pub fn run(self) -> Result<Server, io::Error> {
        let Self {
            tcp_listener,
            db_pool,
            email_client,
            ..
        } = self;

        let db_pool = web::Data::new(db_pool);
        let email_client = web::Data::new(email_client);

        let server = HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .route("/health_check", web::get().to(health_check))
                .route("/subscriptions", web::post().to(subscribe))
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
        })
        .listen(tcp_listener)?
        .run();

        Ok(server)
    }
}
