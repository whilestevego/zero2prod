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

pub async fn build(config: Settings) -> Result<Server, std::io::Error> {
    let Settings {
        application: ApplicationSettings { host, port, .. },
        email_client,
        ref database,
        ..
    } = config;

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

    let address = format!("{host}:{port}");

    let listener = TcpListener::bind(address)?;

    run(listener, db_pool, email_client)
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, io::Error> {
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
    .listen(listener)?
    .run();

    Ok(server)
}
