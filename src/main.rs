use aprs::{AprsMessagingClient, InboundMessage};
use config::Config;
#[macro_use] extern crate rocket;

mod aprs;
mod config;
mod router;

#[tokio::main]
pub async fn main() {
    // Set up fern
    fern::Dispatch::new()
        .level(log::LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    // This application is largely configured by environment variables.
    let config_file_path =
        std::env::var("CONFIG_FILE").unwrap_or_else(|_| "config.toml".to_string());

    // Load the configuration file
    let config = Config::load(config_file_path).unwrap_or_else(|e| {
        eprintln!("Failed to load config file: {}", e);
        std::process::exit(1);
    });

    // Open a connection to APRS-IS
    let mut aprs_client = AprsMessagingClient::connect(
        &config.callsign,
        &config.aprs_passcode,
        handle_inbound_message,
    )
    .await
    .unwrap();

    // Use tokio to spawn the webserver as its own task
    tokio::spawn(async move {
        // Start the webserver
        let _ = rocket::build().mount("/", routes![router::route_meme]).launch().await.unwrap();
    });

    // Loop forever
    aprs_client.run().await.unwrap();
}

fn handle_inbound_message(message: InboundMessage) -> String {
    println!("{:?}", message);
    "Pong!".to_string()
}
