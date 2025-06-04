use crate::ble_device_handlers;
use crate::ble_device_handlers::smart_bike_trainer;

use crate::tcp::device_tcp_parser;
use crate::{ble_device_handlers::BleDevice, tcp, tcp_parser};
use anyhow::{Context, Result, anyhow};
use btleplug::{
    api::{BDAddr, Peripheral},
    platform::PeripheralId,
};
use regex::{self, Regex};
use spdlog::prelude::*;
use std::{collections::HashSet, time::Duration};
use tokio::{net::TcpStream, time::sleep};

pub async fn send_peripherals(
    stream: &mut TcpStream,
    peripherals: &[btleplug::platform::Peripheral],
    valid_peripherals: &mut Vec<btleplug::platform::Peripheral>,
    old_peripherals_adresses: &mut HashSet<BDAddr>,
) {
    for peripheral in peripherals {
        if !old_peripherals_adresses.insert(peripheral.address()) {
            continue;
        }
        // this index will be used to get that specific peripheral by tcp, from c# side, when eg.
        // connecting to it
        let peripheral_index = valid_peripherals.len();

        valid_peripherals.push(peripheral.to_owned());
        let properties = match peripheral.properties().await {
            Ok(o) => o.unwrap(),
            Err(e) => {
                error!(
                    "getting peripheral properties was not possible because: {}",
                    e
                );
                continue;
            }
        };

        let name = match properties.local_name {
            Some(txt) => txt,
            None => "unknown".to_string(),
        };

        tcp::send_tcp_data(stream, format!("i|{}|[{}]", name, peripheral_index)).await;
        println!(
            "send_peripherals: peripheral id:{} {:?}",
            peripheral.id(),
            peripheral
        );
    }
}

const SMART_TRAINER_DEVICE_TYPE: &str = "smart trainer";
pub(super) async fn handle_parsing_peripheral_connection(
    data: &str,
    valid_peripherals: &[btleplug::platform::Peripheral],
    stream: &mut TcpStream,
) -> Result<Option<BleDevice>> {
    info!("handle parsing peripheral connection");

    // extracts name and index from data: i|Smart trainer|[3]
    let regex = Regex::new(r"\|(?<name>.*)\|\[(?<index>.*)\]").context("compiling regex")?;

    let Some(captures) = regex.captures(data) else {
        return Err(anyhow!(""));
    };

    let device_type_name = &captures["name"];
    let device_index: usize = captures["index"]
        .parse()
        .context("parsing index did not succeed")?;
    let peripheral = &valid_peripherals[device_index];
    let device: BleDevice;

    match device_type_name {
        SMART_TRAINER_DEVICE_TYPE => {
            info!("the peripheral type is smart trainer!");

            match ble_device_handlers::smart_bike_trainer::get_smart_trainer_device(
                peripheral.to_owned(),
                device_index,
            )
            .await
            {
                Ok(val) => {
                    device = val;
                }
                Err(error) => {
                    error!("getting smart trainer device returned error: {error}");
                    return Ok(None);
                }
            }
        }
        default => {
            return Err(anyhow!("device name was not recognized: {default}"));
        }
    }
    // only if connection to device succeed
    info!("Successfully connected to device! sending connection information thru tcp");
    device_tcp_parser::send_device_connection_information(stream, device_index, device_type_name)
        .await;
    Ok(Some(device))
}
