use crate::ble_device_handlers::{self, DeviceContainer};
use crate::smart_bike_trainer::SmartTrainer;
use crate::tcp::device_tcp_parser;
use anyhow::{Context, Result, anyhow};
use btleplug::api::{BDAddr, Peripheral};
use regex::{self, Regex};
use spdlog::prelude::*;
use std::collections::HashSet;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;

pub async fn send_found_peripherals(
    tcp_writer_sender: Sender<String>,
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

        tcp_writer_sender
            .send(format!("i|{}|[{}]", name, peripheral_index))
            .await;
        println!(
            "send_peripherals: peripheral id:{} {:?}",
            peripheral.id(),
            peripheral
        );
    }
}

pub(super) async fn handle_parsing_peripheral_connection(
    data: &str,
    valid_peripherals: &[btleplug::platform::Peripheral],
    device_container: &mut DeviceContainer,
    tcp_writer_sender: Sender<String>,
) -> Result<()> {
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
            smart_trainer(
                device_container,
                device_type_name,
                device_index,
                peripheral,
                tcp_writer_sender,
            )
            .await;
        }
        "hr tracker" => {
            hr_tracker(
                tcp_writer_sender,
                device_container,
                device_type_name,
                device_index,
                peripheral,
            )
            .await;
        }
        default => return Err(anyhow!("device name was not recognized: {default}")),
    }

    Ok(())
}

async fn hr_tracker(
    tcp_writer_sender: Sender<String>,
    device_container: &mut DeviceContainer,
    device_type_name: &str,
    device_index: usize,
    peripheral: &btleplug::platform::Peripheral,
) {
    match ble_device_handlers::hr_tracker::get_device(peripheral.to_owned()).await {
        Ok(device) => {
            device_tcp_parser::send_device_connection_information(
                tcp_writer_sender,
                device_index,
                device_type_name,
            )
            .await;

            device_container.hr_tracker = Some(device);
        }
        Err(err) => {
            error!("getting hr tracker device returned error: {err}");
        }
    }
}

async fn smart_trainer(
    device_container: &mut DeviceContainer,
    device_type_name: &str,
    device_index: usize,
    peripheral: &btleplug::platform::Peripheral,
    tcp_writer_sender: Sender<String>,
) {
    match ble_device_handlers::smart_bike_trainer::get_device(peripheral.to_owned()).await {
        Ok(device) => {
            info!("Successfully connected to device! sending connection information thru tcp");
            device_tcp_parser::send_device_connection_information(
                tcp_writer_sender,
                device_index,
                device_type_name,
            )
            .await;

            device_container.smart_trainer = Some(device);
        }
        Err(err) => {
            error!("getting smart trainer device returned error: {err}");
        }
    }
}
