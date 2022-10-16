use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

use zero2prod::{
    db::DB,
    email_client::EmailClient,
    settings::{ApplicationSettings, Settings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let Settings {
        application: ApplicationSettings { host, port, .. },
        email_client,
        ref database,
        ..
    } = Settings::load().expect("Failed to read configuration");

    let sender_email = email_client
        .sender_email()
        .expect("Invalid sender email address.");

    let email_client = EmailClient::new(
        email_client.base_url,
        sender_email,
        email_client.authorization_token,
    );

    let db: DB = database.into();

    let db_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(db.connection_options());

    let address = format!("{host}:{port}");

    let listener = TcpListener::bind(address)?;

    run(listener, db_pool, email_client)?.await
}
