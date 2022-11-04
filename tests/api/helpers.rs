//! tests/health_check.rs
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use wiremock::MockServer;
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
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    /// Spin up an instance of our application
    /// and returns its address (i.e. http://localhost:XXXX)
    pub async fn spawn() -> Self {
        Lazy::force(&TRACING);

        // Mock email server
        let email_server = MockServer::start().await;

        // Load settings and mutate with mock server URL
        let settings = {
            let mut settings = Settings::load().expect("Failed to read configuration");

            settings.email_client.base_url = email_server.uri();
            settings
        };

        // Test TCP listener
        let tcp_listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
        let port = tcp_listener.local_addr().unwrap().port();
        let address = format!("http://localhost:{port}");

        // Test DB pool with UUID db name

        let mut db: DB = (&settings.database).into();

        db.name = Uuid::new_v4().to_string();

        let db_pool = configure_database(&db).await;

        // Build and launch application
        let application = Application::builder_from_settings(settings)
            .set_db_pool(db_pool.clone())
            .set_tcp_listener(tcp_listener)
            .build();

        let port = application.port();

        let _ = tokio::spawn(application.run_until_stopped());

        Self {
            address,
            port,
            db_pool,
            email_server,
        }
    }

    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_health_check(&self) -> reqwest::Response {
        reqwest::Client::new()
            .get(&format!("{}/health_check", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }
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
