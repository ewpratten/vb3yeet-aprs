use regex::Regex;

const APRS_MESSAGE_RE_PATTERN: &str = r"^((?:[1-9][A-Z][A-Z]?+[0-9]|[A-Z][2-9A-Z]?[0-9])[A-Z]{1,5}+[-\dA-Z]*).*::((?:[1-9][A-Z][A-Z]?+[0-9]|[A-Z][2-9A-Z]?[0-9])[A-Z]{1,5}+[-\dA-Z]*)\s*:(.*)\{.*$";

/// Defines a simple container for an inbound message
#[derive(Debug, Clone)]
pub struct InboundMessage {
    /// Sender of the message
    pub respond_to: String,
    /// Message body
    pub message: String,
}

#[derive(Debug)]
pub struct AprsMessagingClient<CallbackFn> {
    inbound_buffer: [u8; 1024],
    stream: tokio::net::TcpStream,
    callback: CallbackFn,
    message_validator: Regex,
    callsign: String,
}

impl<CallbackFn> AprsMessagingClient<CallbackFn>
where
    CallbackFn: Fn(InboundMessage) -> String,
{
    pub async fn connect(
        callsign: &str,
        passcode: &str,
        handler_callback: CallbackFn,
    ) -> Result<Self, std::io::Error> {
        // Open TCP connection to APRS-IS
        let stream = tokio::net::TcpStream::connect("first.aprs.net:14580").await?;
        stream.readable().await?;

        // Build the buffer
        let mut buffer = [0u8; 1024];

        // Consume the welcome message
        stream.try_read(&mut buffer)?;
        log::info!(
            "APRS-IS server header: {}",
            String::from_utf8_lossy(&buffer[..]).trim()
        );

        // Send login message
        let login_message = format!("user {} pass {} filter t/m\r\n", callsign, passcode);
        stream.try_write(login_message.as_bytes())?;
        log::debug!("Sent login message and requested message-only filtering");

        Ok(Self {
            inbound_buffer: buffer,
            stream,
            callback: handler_callback,
            message_validator: Regex::new(APRS_MESSAGE_RE_PATTERN).unwrap(),
            callsign: callsign.to_string(),
        })
    }

    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        loop {
            // Wait for reading to be available
            self.stream.readable().await?;

            // Read the next message
            match self.stream.try_read(&mut self.inbound_buffer) {
                Ok(bytes_read) => {
                    // Seek through the buffer to find all messages (there may be more than one)
                    let mut message_start = 0;
                    for (byte, i) in self.inbound_buffer[..bytes_read].iter().enumerate() {
                        if *i == b'\n' {
                            let message = &self.inbound_buffer[message_start..byte];
                            self.handle_raw_inbound_message(message)?;
                            message_start = byte + 1;
                        }
                    }

                    // Clear the buffer
                    self.clear_buffer();
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

    fn clear_buffer(&mut self) {
        self.inbound_buffer = [0u8; 1024];
    }

    fn handle_raw_inbound_message(&self, raw_message: &[u8]) -> Result<(), std::io::Error> {
        let message = String::from_utf8_lossy(raw_message).trim().to_string();

        // Log the message
        log::trace!("APRS-IS message: {}", message);

        // Parse the message
        if let Some(captures) = self.message_validator.captures(&message) {
            let respond_to = captures.get(1).unwrap().as_str();
            let destination = captures.get(2).unwrap().as_str();
            let message = captures.get(3).unwrap().as_str();

            // Check if this is a message for us
            if destination == self.callsign {
                log::info!("Message received from {}", respond_to);

                // Callback the handler
                let response = (self.callback)(InboundMessage {
                    respond_to: respond_to.to_string(),
                    message: message.to_string(),
                });

                // Construct a new APRS packet with the response and send it
                let packet = format!("{}>APRS,TCPIP*::{}:{}\r\n", self.callsign, respond_to, response);
                self.stream.try_write(packet.as_bytes())?;
            }
        }

        Ok(())
    }
}
