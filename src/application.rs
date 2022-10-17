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

pub struct ApplicationBuilder {
    db_pool: Option<PgPool>,
    email_client: Option<EmailClient>,
    tcp_listener: Option<TcpListener>,
    host: Option<String>,
    port: Option<u16>,
}

impl ApplicationBuilder {
    pub fn from_settings(settings: Settings) -> Result<Self, std::io::Error> {
        let Settings {
            application: ApplicationSettings { host, port, .. },
            email_client,
            ref database,
            ..
        } = settings;

        let sender_email = email_client
            .sender_email()
            .expect("Invalid sender email address.");

        let timeout = email_client.timeout();

        let email_client = EmailClient::new(
            email_client.base_url,
            sender_email,
            email_client.authorization_token,
            timeout,
        );

        let db: DB = database.into();

        let db_pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(db.connection_options());

        Ok(Self {
            db_pool: Some(db_pool),
            email_client: Some(email_client),
            host: Some(host),
            port: Some(port),
            tcp_listener: None,
        })
    }

    pub fn set_db_pool(mut self, db_pool: PgPool) -> Self {
        self.db_pool = Some(db_pool);
        self
    }

    pub fn set_tcp_listener(mut self, tcp_listener: TcpListener) -> Self {
        self.tcp_listener = Some(tcp_listener);
        self
    }

    pub fn build(self) -> Application {
        let Self {
            db_pool,
            email_client,
            host,
            port,
            tcp_listener,
        } = self;

        // TODO: Do proper error handling here
        Application {
            db_pool: db_pool.unwrap(),
            tcp_listener: tcp_listener.unwrap_or_else(|| {
                let host = host.unwrap();
                let port = port.unwrap();

                let address = format!("{host}:{port}");

                TcpListener::bind(&address)
                    .unwrap_or_else(|_| panic!("Couldn't bind TCP listener to {}", &address))
            }),
            email_client: email_client.unwrap(),
        }
    }
}

pub struct Application {
    db_pool: PgPool,
    tcp_listener: TcpListener,
    email_client: EmailClient,
}

impl Application {
    pub fn builder_from_settings(settings: Settings) -> Result<ApplicationBuilder, std::io::Error> {
        ApplicationBuilder::from_settings(settings)
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
