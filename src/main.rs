use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
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
        application: ApplicationSettings { host, port, .. },
        database,
        ..
    } = Settings::load().expect("Failed to read configuration");

    let db = DB::from_url(database.url.expose_secret());

    let db_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(db.connection_options());

    let address = format!("{host}:{port}");

    let listener = TcpListener::bind(address)?;

    run(listener, db_pool)?.await
}
