//! tests/health_check.rs
use linkify::{LinkFinder, LinkKind};
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

use crate::test_user::TestUser;

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

#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub text: reqwest::Url,
}

pub struct TestApp {
    pub test_user: TestUser,
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

        let app = Self {
            test_user: TestUser::generate(),
            address,
            port,
            db_pool,
            email_server,
        };

        app.test_user.insert(&app.db_pool).await;

        app
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

    pub async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.address))
            .basic_auth(&self.test_user.username, Some(&self.test_user.password))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |haystack: &str| {
            let links: Vec<_> = LinkFinder::new()
                .links(haystack)
                .filter(|link| *link.kind() == LinkKind::Url)
                .collect();

            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let html = get_link(body["content"][0]["value"].as_str().unwrap());
        let text = get_link(body["content"][1]["value"].as_str().unwrap());

        ConfirmationLinks { html, text }
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
