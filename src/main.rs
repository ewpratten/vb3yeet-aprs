mod services;

#[tokio::main]
pub async fn main() {
    // Read configuration from the environment
    let cfg_enable_verbose_logging = std::env::var("VERBOSE_LOGGING").is_ok();
    let cfg_callsign = std::env::var("CALLSIGN").expect("$CALLSIGN is not set");
    let cfg_passcode = std::env::var("PASSCODE").expect("$PASSCODE is not set");

    // Enable logging
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(if cfg_enable_verbose_logging {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .chain(std::io::stdout())
        .apply()
        .expect("Failed to initialize logging");

    // Create the I/O channels needed for passing log entries between tasks
    let (log_entry_tx, log_entry_rx) = tokio::sync::mpsc::channel(100);

    // Start the APRS listener
    let aprs_listener_task = tokio::spawn(services::aprs_listener::begin_aprs_listener(
        cfg_callsign,
        cfg_passcode,
        log_entry_tx,
    ));

    // Start the log handler
    let log_handler_task = tokio::spawn(services::log_sync::handle_inbound_log_stream(log_entry_rx));

    // For now, we will just join the aprs listener task
    aprs_listener_task.await.unwrap().unwrap();
}
