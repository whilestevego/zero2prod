//! tests/health_check.rs
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    application::Application,
    db::DB,
    settings::Settings,
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

    let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = tcp_listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let settings = Settings::load().expect("Failed to read configuration");

    let mut db: DB = (&settings.database).into();

    db.name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&db).await;

    let application = Application::builder()
        .set_db_pool(db_pool.clone())
        .set_tcp_listener(tcp_listener)
        .build();

    let _ = tokio::spawn(application.run_until_stopped());

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
    let db_pool = PgPool::connect(db.url().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database");

    db_pool
}
