use adif::AdifRecord;
use tokio::sync::mpsc::Sender;

use crate::services::message_handler::handle_message;

/// A REGEX pattern used for parsing APRS messages because I am lazy
const APRS_MESSAGE_RE_PATTERN: &str = r"^((?:[1-9][A-Z][A-Z]?+[0-9]|[A-Z][2-9A-Z]?[0-9])[A-Z]{1,5}+[-\dA-Z]*).*::((?:[1-9][A-Z][A-Z]?+[0-9]|[A-Z][2-9A-Z]?[0-9])[A-Z]{1,5}+[-\dA-Z]*)\s*:(.*)\{(.*)$";

/// Starts the APRS message listener task
pub async fn begin_aprs_listener(
    callsign: String,
    passcode: String,
    log_entry_queue: Sender<AdifRecord>,
) -> Result<(), std::io::Error> {
    // Open a TCP connection to the APRS-IS server
    let stream = tokio::net::TcpStream::connect("first.aprs.net:14580").await?;
    stream.readable().await?;

    // Build a buffer to hold incoming packets
    let mut buffer = [0u8; 2048];

    // Handle the login sequence
    stream.try_read(&mut buffer)?;
    log::info!("APRS-IS header: {}", String::from_utf8_lossy(&buffer));
    stream.writable().await?;
    stream.try_write(format!("user {} pass {} filter t/m\r\n", callsign, passcode).as_bytes())?;

    // Compile the regex pattern for parsing APRS messages
    let aprs_message_re = regex::Regex::new(APRS_MESSAGE_RE_PATTERN).unwrap();

    // Listen for incoming messages
    loop {
        // Read into the buffer
        buffer = [0u8; 2048];
        stream.readable().await?;
        match stream.try_read(&mut buffer) {
            Ok(bytes_read) => {
                // Convert the buffer to a string and split on newlines to get individual messages
                let message_clump = String::from_utf8_lossy(&buffer[0..bytes_read]);

                // Parse each message
                for message in message_clump.split('\n') {
                    // Search for APRS messages
                    if let Some(captures) = aprs_message_re.captures(&message) {
                        let respond_to = captures.get(1).unwrap().as_str();
                        let destination = captures.get(2).unwrap().as_str();
                        let message = captures.get(3).unwrap().as_str();
                        let ack_code = captures.get(4).unwrap().as_str();
                        log::debug!(
                            "APRS message: {} -> {}: {}",
                            respond_to,
                            destination,
                            message
                        );

                        // Check if this message was directed at us
                        if destination == &callsign {
                            log::info!("Got message from {}: {}", respond_to, message);

                            // Spawn a handler
                            if let Ok(response) = handle_message(
                                callsign.clone(),
                                respond_to.to_string(),
                                message.to_string(),
                                log_entry_queue.clone(),
                            )
                            .await
                            {
                                stream.writable().await?;

                                // Send a message acknowledgement
                                stream.try_write(
                                    format!(
                                        "{}>APZ412,TCPIP*::{}:ack{}\r\n",
                                        callsign, respond_to, ack_code
                                    )
                                    .as_bytes(),
                                )?;

                                // Send our own repsonse
                                stream.try_write(response.as_bytes())?;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        // We can skip these errors, as they are expected
                    }
                    _ => {
                        log::error!("Failed to read from stream: {}", e);
                        return Err(e);
                    }
                }
            }
        }
    }
}
