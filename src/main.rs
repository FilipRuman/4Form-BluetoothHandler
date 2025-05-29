mod ble_device_handlers;
mod logger;
mod tcp;
mod tcp_parser;

use btleplug::api::Characteristic;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};

use std::collections::HashSet;

use std::time::Duration;
use tcp::create_stream;
use tcp::read_tcp_data;
use tokio::time;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let mut stream = create_stream().await;
    let adapter = ble_device_handlers::general::start_scan().await;
    //TODO: Add error handler + logger and remove unwrap

    let mut old_peripherals_len = 0;
    let mut old_peripherals_id = HashSet::new();
    loop {
        let peripherals =
            ble_device_handlers::general::handle_scanning_for_peripherals(&adapter).await;

        tcp_parser::send_peripherals(
            &mut stream,
            &mut old_peripherals_len,
            &peripherals,
            &mut old_peripherals_id,
        )
        .await;
        old_peripherals_len = peripherals.len();
        let tcp_output = read_tcp_data(&mut stream);
        match tcp_output {
            Some(data) => {
                println!("tcp_output {:?} \n \n ", data);
                tcp_parser::handle_data_input_from_tcp(data, &peripherals, &mut stream);
            }
            None => println!("tcp_output None \n \n "),
        }
    }
}
