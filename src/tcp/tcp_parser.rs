use crate::{
    ble_device_handlers::BleDevice,
    tcp::{self, peripherals_tcp_parser},
};

use spdlog::prelude::*;
use tokio::net::TcpStream;

pub async fn send_smart_watch_data(stream: &mut TcpStream, hr: u8) {
    tcp::send_tcp_data(stream, format!("h hr:{};", hr)).await;
}

pub async fn send_bike_trainer_data(
    stream: &mut TcpStream,
    power: u16,
    cadence: u16,
    wheel_rotation: u16,
) {
    tcp::send_tcp_data(
        stream,
        format!(
            "t power:{};cadence:{};rotation:{};",
            power, cadence, wheel_rotation
        ),
    )
    .await;
}

pub async fn handle_data_input_from_tcp(
    data: &str,
    valid_peripherals: &[btleplug::platform::Peripheral],
    devices: &Vec<BleDevice>,

    stream: &mut TcpStream,
) -> Option<BleDevice> {
    if data.is_empty() || data.starts_with('\0') {
        return None;
    }

    let mut chars = data.chars();

    info!("parsing data from tcp: {}", data);

    // this would only fail if len == 0 but it is checked

    match chars.next().unwrap() {
        'i' => {
            match peripherals_tcp_parser::handle_parsing_peripheral_connection(
                data,
                valid_peripherals,
                stream,
            )
            .await
            {
                Ok(option_value) => option_value,
                Err(error) => {
                    error!("handling parsing peripheral: {error}");
                    None
                }
            }
        }
        's' => match peripherals_tcp_parser::handle_slope_control(devices, data).await {
            Ok(_) => None,
            Err(error) => {
                error!("error while setting slope {error}");
                None
            }
        },
        _ => {
            warn!("tcp input type not found");
            None
        }
    }
}
