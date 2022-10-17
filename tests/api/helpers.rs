//! tests/health_check.rs
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    db::DB,
    email_client::EmailClient,
    settings::Settings,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

/// Spin up an instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let Settings {
        ref database,
        email_client,
        ..
    } = Settings::load().expect("Failed to read configuration");

    let mut db: DB = database.into();

    db.name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&db).await;

    // Build a new email client
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

    let server = run(listener, db_pool.clone(), email_client).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp { address, db_pool }
}

async fn configure_database(db: &DB) -> PgPool {
    // Create Database
    let mut connection = PgConnection::connect_with(&db.connection_options_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db.name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate Database
    let connection_pool = PgPool::connect(db.url().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
