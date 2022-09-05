use std::net::TcpListener;

use zero2prod::{
    configuration::{get_configuration, Settings},
    run,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let Settings {
        application_port, ..
    } = get_configuration().expect("Failed to read configuration");

    let address = format!("127.0.0.1:{application_port}");

    run(TcpListener::bind(address)?)?.await
}
