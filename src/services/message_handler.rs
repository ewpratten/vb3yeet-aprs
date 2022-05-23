use adif::{AdifRecord, AdifType};
use chrono::{Datelike, Timelike};
use indexmap::IndexMap;
use tokio::sync::mpsc::Sender;

/// Regex pattern for stripping SSID from callsigns
const SSID_RE_PATTERN: &str =
    r"((?:[1-9][A-Z][A-Z]?+[0-9]|[A-Z][2-9A-Z]?[0-9])[A-Z]{1,5}+)[-\dA-Z]*";

/// Inbound message handler
pub async fn handle_message(
    callsign: String,
    sender: String,
    message: String,
    log_entry_queue: Sender<AdifRecord>,
) -> Result<String, reqwest::Error> {
    // The message we shall send
    let message = "QSL, log updated. More info at va3zza.com/yeet";
    log::info!("Building response: {}", message);

    // Build our response
    let current_time = chrono::Utc::now();
    let response = format!(
        "{}>APZ412,TCPIP*::{:9}:{}{{{}\r\n",
        callsign,
        sender,
        message,
        current_time.timestamp() % 10000
    );

    // Strip the SSIDs from the callsigns for logging
    let sender_re = regex::Regex::new(SSID_RE_PATTERN).unwrap();
    let sender_no_ssid = sender_re.replace_all(&sender, "$1").to_string();
    let callsign_no_ssid = sender_re.replace_all(&callsign, "$1").to_string();

    // Build a log entry
    let mut log_data = IndexMap::new();
    log_data.insert(
        "STATION_CALLSIGN".to_string(),
        AdifType::Str(callsign_no_ssid),
    );
    log_data.insert("CALL".to_string(), AdifType::Str(sender_no_ssid));
    log_data.insert("BAND".to_string(), AdifType::Str("2M".to_string()));
    log_data.insert("MODE".to_string(), AdifType::Str("FM".to_string()));
    log_data.insert("FREQ".to_string(), AdifType::Number(144.39));
    log_data.insert(
        "QSO_DATE".to_string(),
        AdifType::Str(format!(
            "{:04}{:02}{:02}",
            current_time.year(),
            current_time.month(),
            current_time.day()
        )),
    );
    log_data.insert(
        "TIME_ON".to_string(),
        AdifType::Str(format!(
            "{:02}{:02}{:02}",
            current_time.hour(),
            current_time.minute(),
            current_time.second()
        )),
    );
    let adif_record = AdifRecord::from(log_data);
    log::debug!("Log entry: {}", adif_record.to_string());

    // Send the log entry to the queue
    log_entry_queue
        .send(adif_record)
        .await
        .unwrap();

    Ok(response)
}
