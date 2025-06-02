use std::{collections::HashSet, default, io::Error, thread::sleep, time::Duration};

use crate::{
    ble_device_handlers::{self, BleDevice},
    tcp::{self, peripherals_tcp_parser},
};
use anyhow::{Context, Result, anyhow};
use btleplug::{
    api::{BDAddr, Peripheral},
    platform::PeripheralId,
};
use regex::{self, Regex};
use spdlog::prelude::*;
use tokio::net::TcpStream;

pub async fn send_bike_trainer_data(stream: &mut TcpStream, power: u16, cadence: u16) {
    tcp::send_tcp_data(stream, format!("p{}", power)).await;
    tcp::send_tcp_data(stream, format!("c{}", cadence)).await;
}

pub async fn handle_data_input_from_tcp(
    data: &str,
    valid_peripherals: &[btleplug::platform::Peripheral],

    stream: &mut TcpStream,
) -> Option<BleDevice> {
    if data.is_empty() || data.starts_with('\0') {
        return None;
    }

    let mut chars = data.chars();

    info!("parsing data from tcp: {}", data);

    // this would only fail if len == 0 but it is checked

    match chars.next().unwrap() {
        'i' => match peripherals_tcp_parser::handle_parsing_peripheral_connection(
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
        },
        _ => {
            warn!("tcp input type not found");
            None
        }
    }
}
