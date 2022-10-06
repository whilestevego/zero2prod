use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::{
    configuration::{get_configuration, Settings},
    db::DB,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let Settings {
        application_port,
        database,
        ..
    } = get_configuration().expect("Failed to read configuration");

    let db = DB::from_url(database.url.expose_secret());

    let db_pool = PgPool::connect(&db.url().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{application_port}");

    let listener = TcpListener::bind(address)?;

    run(listener, db_pool)?.await
}
