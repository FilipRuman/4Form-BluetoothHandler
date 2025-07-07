use crate::ble_device_handlers;

use crate::tcp::device_tcp_parser;
use crate::{ble_device_handlers::BleDevice, tcp};
use anyhow::{Context, Result, anyhow};
use btleplug::api::{BDAddr, Peripheral};
use regex::{self, Regex};
use spdlog::prelude::*;
use std::collections::HashSet;
use tokio::net::TcpStream;

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

pub(super) async fn handle_slope_control(devices: &Vec<BleDevice>, data: &str) -> Result<()> {
    let slope: i16 = data[0..data.len()].parse()?; // slope% should be * 100 so 5.12% slope -> 512
    // maybe later replace with hashmap so you don't have to iterate thru whole array
    for device in devices {
        if let BleDevice::SmartTrainer {
            control_char,
            data_char: _,
            peripheral,
        } = device
        {
            ble_device_handlers::smart_bike_trainer::set_target_slope(
                slope,
                peripheral,
                control_char,
            )
            .await
            .context("setting target slope")?;
        }
    }
    Ok(())
}

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

    match device_type_name {
        "smart trainer" => {
            info!("the peripheral type is smart trainer!");

            match ble_device_handlers::smart_bike_trainer::get_device(peripheral.to_owned()).await {
                Ok(device) => {
                    info!(
                        "Successfully connected to device! sending connection information thru tcp"
                    );
                    device_tcp_parser::send_device_connection_information(
                        stream,
                        device_index,
                        device_type_name,
                    )
                    .await;
                    Ok(Some(device))
                }
                Err(err) => {
                    error!("getting smart trainer device returned error: {err}");
                    Ok(None)
                }
            }
        }
        "hr tracker" => {
            match ble_device_handlers::hr_tracker::get_device(peripheral.to_owned()).await {
                Ok(device) => {
                    device_tcp_parser::send_device_connection_information(
                        stream,
                        device_index,
                        device_type_name,
                    )
                    .await;

                    Ok(Some(device))
                }
                Err(err) => {
                    error!("getting hr tracker device returned error: {err}");
                    Ok(None)
                }
            }
        }
        default => Err(anyhow!("device name was not recognized: {default}")),
    }
}
