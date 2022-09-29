use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::{
    configuration::{get_configuration, Settings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let Settings {
        application_port,
        database,
        ..
    } = get_configuration().expect("Failed to read configuration");

    let db_pool = PgPool::connect(&database.url)
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{application_port}");

    let listener = TcpListener::bind(address)?;

    run(listener, db_pool)?.await
}
