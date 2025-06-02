use crate::ble_device_handlers;
use crate::ble_device_handlers::smart_bike_trainer;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;

use btleplug::api::{BDAddr, Peripheral};
use regex::Regex;
use spdlog::prelude::*;
use std::{collections::HashSet, time::Duration};
use tokio::{net::TcpStream, time::sleep};

use crate::{ble_device_handlers::BleDevice, tcp};

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

        valid_peripherals.push(peripherals[peripheral_index].to_owned());
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

        tcp::send_tcp_data(stream, format!("i[{}]|{}|", name, peripheral_index)).await;
        // have to wait some time when sending multiple packages so they don't stack up to one on
        // the c# side
        sleep(Duration::from_millis(5));
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

            match ble_device_handlers::smart_bike_trainer::get_smart_trainer_device(
                peripheral.to_owned(),
                device_index,
            )
            .await
            {
                Ok(val) => Ok(Some(val)),
                Err(error) => {
                    error!("getting smart trainer device returned error: {error}");
                    Ok(None)
                }
            }
        }
        default => Err(anyhow!("device name was not recognized: {default}")),
    }
}
