mod ble_device_handlers;
mod logs;
mod tcp;
use std::collections::HashSet;
use std::time::Duration;

use ble_device_handlers::DeviceContainer;
use ble_device_handlers::hr_tracker;
use ble_device_handlers::smart_bike_trainer;
use btleplug::api::BDAddr;
use btleplug::api::Central;
use spdlog::prelude::*;
use tcp::read_tcp_data;
use tcp::tcp_parser;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
#[tokio::main]
async fn main() {
    logs::setup_logger();
    info!("Init rust ble handler");

    let (mut tcp_reader, tcp_writer_sender) = tcp::setup_tcp().await;

    let adapter = match ble_device_handlers::start_scan().await {
        Ok(o) => o,
        Err(e) => {
            error!("scanning for peripherals did not succeed because:{e:?}");
            panic!()
        }
    };
    let mut old_peripherals_mac_address: HashSet<BDAddr> = HashSet::new();
    let mut valid_peripherals: Vec<btleplug::platform::Peripheral> = Vec::new();
    let mut device_container = ble_device_handlers::DeviceContainer {
        smart_trainer: None,
        hr_tracker: None,
    };

    loop {
        let peripherals = adapter.peripherals().await.unwrap();

        tokio::spawn(handle_devices(
            device_container.clone(),
            tcp_writer_sender.clone(),
        ));

        tcp::peripherals_tcp_parser::send_found_peripherals(
            tcp_writer_sender.clone(),
            &peripherals,
            &mut valid_peripherals,
            &mut old_peripherals_mac_address,
        )
        .await;
        let tcp_output = read_tcp_data(&mut tcp_reader);
        if let Some(raw_data) = tcp_output {
            let splitted_data = raw_data.split('\n');

            for data in splitted_data {
                tcp_parser::handle_data_input_from_tcp(
                    data,
                    &valid_peripherals,
                    &mut device_container,
                    tcp_writer_sender.clone(),
                )
                .await
            }
        }

        sleep(Duration::from_millis(1)).await;
    }
}
async fn handle_devices(device_container: DeviceContainer, tcp_writer_sender: Sender<String>) {
    if let Some(smart_trainer) = device_container.smart_trainer.to_owned() {
        if let Err(err) =
            smart_bike_trainer::handle_peripheral(&smart_trainer.peripheral, tcp_writer_sender.to_owned())
                .await
        {
            error!(
                "error occurred while handling smart trainer device peripheral{}",
                err
            );
        }
    }

    if let Some(hr_tracker) = device_container.hr_tracker.to_owned() {
        if let Err(err) =
            hr_tracker::handle_peripheral(&hr_tracker.peripheral, tcp_writer_sender.to_owned())
                .await
        {
            error!(
                "error occurred while handling hr tracking device peripheral{}",
                err
            );
        }
    }
}
