use zero2prod::{
    settings::Settings,
    startup::build,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = Settings::load().expect("Failed to read configuration");
    let server = build(config).await?;
    server.await?;

    Ok(())
}
