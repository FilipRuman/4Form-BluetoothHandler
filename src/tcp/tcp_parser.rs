use crate::{
    ble_device_handlers::DeviceContainer, smart_bike_trainer, tcp::peripherals_tcp_parser,
};

use spdlog::prelude::*;
use tokio::{net::TcpStream, sync::mpsc::Sender};

pub async fn send_smart_watch_data(tcp_writer_sender: &mut Sender<String>, hr: u8) {
    tcp_writer_sender.send(format!("h hr:{};", hr)).await;
}

pub async fn send_bike_trainer_data(
    tcp_writer_sender: Sender<String>,
    power: u16,
    cadence: u16,
    wheel_rotation: u16,
) {
    tcp_writer_sender
        .send(format!(
            "t power:{};cadence:{};rotation:{};",
            power, cadence, wheel_rotation
        ))
        .await;
}

pub async fn handle_data_input_from_tcp(
    data: &str,
    valid_peripherals: &[btleplug::platform::Peripheral],
    device_container: &mut DeviceContainer,
    tcp_writer_sender: Sender<String>,
) {
    if data.is_empty() || data.starts_with('\0') {
        return;
    }

    let mut chars = data.chars();

    info!("parsing data from tcp: {}", data);

    // this would only fail if len == 0 but it is checked
    match chars.next().unwrap() {
        'i' => {
            match peripherals_tcp_parser::handle_parsing_peripheral_connection(
                data,
                valid_peripherals,
                device_container,
                tcp_writer_sender,
            )
            .await
            {
                Ok(option_value) => option_value,
                Err(error) => {
                    error!("handling parsing peripheral: {error}");
                }
            }
        }
        's' => {
            if let Some(smart_trainer) = device_container.smart_trainer.to_owned() {
                tokio::spawn(smart_bike_trainer::set_target_slope(
                    data.to_owned(),
                    smart_trainer,
                ));
            }
        }
        _ => {
            warn!("tcp input type not found");
        }
    }
}
