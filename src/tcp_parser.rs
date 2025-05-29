use std::{collections::HashSet, default, thread::sleep, time::Duration};

use crate::{ble_device_handlers, tcp};
use btleplug::{api::Peripheral, platform::PeripheralId};
use regex::{self, Regex};
use tokio::net::TcpStream;

pub async fn send_bike_trainer_data(stream: &mut TcpStream, power: u16, cadence: u16) {
    tcp::send_tcp_data(stream, format!("p{}", power)).await;
    tcp::send_tcp_data(stream, format!("c{}", cadence)).await;
}
pub async fn send_peripherals(
    stream: &mut TcpStream,
    old_peripherals_len: &mut usize,
    peripherals: &[btleplug::platform::Peripheral],
    old_peripherals_ids: &mut HashSet<PeripheralId>,
) {
    for i in old_peripherals_len.to_owned()..peripherals.len() {
        let peripheral = &peripherals[i];
        if !old_peripherals_ids.insert(peripheral.id()) {
            continue;
        }
        let properties = peripheral.properties().await.unwrap().unwrap();

        let name = match properties.local_name {
            Some(txt) => txt,
            None => "unknown".to_string(),
        };

        tcp::send_tcp_data(stream, format!("i[{}]|{}|", name, i)).await;
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
pub fn handle_data_input_from_tcp(
    data: String,
    peripherals: &[btleplug::platform::Peripheral],
    stream: &mut TcpStream,
) {
    let mut chars = data.chars();
    match chars.next().unwrap() {
        'i' => {
            handle_parsing_peripheral_connection(data, peripherals, stream);
        }
        _ => {}
    }
}

fn handle_parsing_peripheral_connection(
    data: String,
    peripherals: &[btleplug::platform::Peripheral],
    stream: &mut TcpStream,
) {
    println!("handle_parsing_smart_trainer");

    // extracts name and index from data: i|Smart trainer|[3]
    let regex = Regex::new(r"\|(?<name>.*)\|\[(?<index>.*)\]").unwrap();
    let Some(captures) = regex.captures(&data) else {
        return;
    };
    let device_type_name = &captures["name"];
    let device_index: usize = captures["index"].parse().unwrap();
    let peripheral = &peripherals[device_index];
    match device_type_name {
        "smart trainer" => {
            ble_device_handlers::smart_bike_trainer::handle_smart_trainer_peripheral(
                stream, peripheral,
            );
        }
        default => {
            panic!("device type not found! {:?}", default);
        }
    }
}
