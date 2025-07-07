mod ble_device_handlers;
mod logs;
mod tcp;
use ble_device_handlers::BleDevice;
use btleplug::api::BDAddr;
use btleplug::api::Peripheral;
use spdlog::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use tcp::create_stream;
use tcp::read_tcp_data;
use tcp::tcp_parser;
#[tokio::main]
async fn main() {
    logs::setup_logger();
    info!("Init rust ble handler");

    let mut stream = match create_stream().await {
        Ok(o) => o,
        Err(e) => {
            error!("creating tcp listener did not succeed because:{e:?}");
            panic!()
        }
    };

    let adapter = match ble_device_handlers::start_scan().await {
        Ok(o) => o,
        Err(e) => {
            error!("scanning for peripherals did not succeed because:{e:?}");
            panic!()
        }
    };
    let mut old_peripherals_mac_address: HashSet<BDAddr> = HashSet::new();
    let mut valid_peripherals: Vec<btleplug::platform::Peripheral> = Vec::new();
    let mut devices: Vec<BleDevice> = Vec::new();
    loop {
        let peripherals = ble_device_handlers::get_found_peripherals(&adapter).await;

        ble_device_handlers::handle_devices(&devices, &valid_peripherals, &mut stream).await;

        tcp::peripherals_tcp_parser::send_peripherals(
            &mut stream,
            &peripherals,
            &mut valid_peripherals,
            &mut old_peripherals_mac_address,
        )
        .await;
        let tcp_output = read_tcp_data(&mut stream);

        if let Some(raw_data) = tcp_output {
            let splitted_data = raw_data.split('\n');

            for data in splitted_data {
                if let Some(device) = tcp_parser::handle_data_input_from_tcp(
                    data,
                    &valid_peripherals,
                    &devices,
                    &mut stream,
                )
                .await
                {
                    info!("new device was added: {:?}", devices);
                    devices.push(device);
                };
            }
        }
    }
}
