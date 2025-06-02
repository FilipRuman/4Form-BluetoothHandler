use crate::tcp;
use tokio::net::TcpStream;
pub async fn send_device_connection_information(
    stream: &mut TcpStream,
    index: usize,
    device_type_name: &str,
) {
    tcp::send_tcp_data(stream, format!("o|{}|[{}]", device_type_name, index)).await;
}
pub async fn send_bike_trainer_data(stream: &mut TcpStream, power: u16, cadence: u16) {
    tcp::send_tcp_data(stream, format!("p{}", power)).await;
    tcp::send_tcp_data(stream, format!("c{}", cadence)).await;
}
