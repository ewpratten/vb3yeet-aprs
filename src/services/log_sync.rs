use adif::AdifRecord;
use tokio::sync::mpsc::Receiver;


pub async fn handle_inbound_log_stream(queue: Receiver<AdifRecord>) {

}