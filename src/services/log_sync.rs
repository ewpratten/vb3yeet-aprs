use std::{path::PathBuf, io::Write};

use adif::AdifRecord;
use tokio::sync::mpsc::Receiver;

pub async fn handle_inbound_log_stream(log_file_path: String, mut queue: Receiver<AdifRecord>) {

    let log_file_path = PathBuf::from(log_file_path);

    // If the file does not exist, create it and inject out program header
    if !log_file_path.exists() {
        let mut log_file = std::fs::File::create(&log_file_path).unwrap();
        log_file.write_all(b"LIGMA BALLS OM\n<adif_ver:5>2.2.7\n<programid:12>VB3YEET-APRS\n\n<EOH>\nUwU Data:\n").unwrap();
    }

    // Wait for incoming log entries
    loop {
        match queue.try_recv(){
            Ok(log_entry) => {
                // Write the log entry to the file
                let mut log_file = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&log_file_path)
                    .unwrap();
                log_file.write_all(log_entry.serialize().unwrap().as_bytes()).unwrap();
                log_file.write_all(b"\n").unwrap();
            },
            Err(_) => {},
        }
    }
}
