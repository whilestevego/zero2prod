use zero2prod::{
    application::Application,
    settings::Settings,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    Application::builder_from_settings(Settings::load().expect("Failed to read configuration"))
        .build()
        .run_until_stopped()
        .await?;

    Ok(())
}
