use tokio::sync::mpsc::Sender;
pub async fn send_device_connection_information(
    tcp_writer_sender: Sender<String>,
    index: usize,
    device_type_name: &str,
) {
    tcp_writer_sender
        .send(format!("o|{}|[{}]", device_type_name, index))
        .await;
}
pub async fn send_bike_trainer_data(tcp_writer_sender: Sender<String>, power: u16, cadence: u16) {
    tcp_writer_sender.send(format!("p{}", power)).await;
    tcp_writer_sender.send(format!("c{}", cadence)).await;
}
