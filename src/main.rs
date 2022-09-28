use sqlx::PgPool;
use std::net::TcpListener;
use tracing::dispatcher::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::{
    configuration::{get_configuration, Settings},
    startup::run,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber.into()).expect("Failed to set subscriber");

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
