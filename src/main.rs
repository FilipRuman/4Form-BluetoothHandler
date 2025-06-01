mod ble_device_handlers;
mod logs;
mod tcp;
mod tcp_parser;
use spdlog::prelude::*;
use std::collections::HashSet;
use tcp::create_stream;
use tcp::read_tcp_data;
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
            error!("scanning for peripherials did not succeed because:{e:?}");
            panic!()
        }
    };

    let mut old_peripherals_len = 0;
    let mut old_peripherals_id = HashSet::new();
    loop {
        let peripherals = ble_device_handlers::get_found_peripherals(&adapter).await;

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
                // println!("tcp_output {:?} \n \n ", data);
                tcp_parser::handle_data_input_from_tcp(data, &peripherals, &mut stream);
            }
            None => {}
        }
    }
}
