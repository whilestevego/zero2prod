use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

use zero2prod::{
    db::DB,
    settings::{ApplicationSettings, Settings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let Settings {
        application: ApplicationSettings { host, port },
        database,
        ..
    } = Settings::load().expect("Failed to read configuration");

    let db = DB::from_url(database.url.expose_secret());

    let db_pool =
        PgPool::connect_lazy(db.url().expose_secret()).expect("Failed to connect to Postgres");

    let address = format!("{host}:{port}");

    let listener = TcpListener::bind(address)?;

    run(listener, db_pool)?.await
}
