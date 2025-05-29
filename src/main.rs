mod ble_device_handlers;
mod logger;
mod tcp;
mod tcp_parser;

use btleplug::api::Characteristic;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};

use std::collections::HashSet;

use std::error::Error;
use std::time::Duration;
use tcp::create_stream;
use tcp::read_tcp_data;
use tokio::time;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = create_stream().await;
    let adapter = start_scan().await;
    //TODO: Add error handler + logger and remove unwrap

    let mut old_peripherals_len = 0;
    let mut old_peripherals_id = HashSet::new();
    loop {
        let peripherals = handle_scanning_for_peripherals(&adapter).await;

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

    Ok(())
}

async fn start_scan() -> Adapter {
    let manager = Manager::new().await.unwrap();
    // get the first bluetooth adapter
    let adapters = manager.adapters().await.unwrap();
    let central = adapters.into_iter().next().unwrap();

    central.start_scan(ScanFilter::default()).await.unwrap();

    println!("started scanning");
    central
}
async fn connect_to_peripheral(selected_peripheral: Peripheral) {
    selected_peripheral.connect().await.unwrap();
    selected_peripheral.discover_services().await.unwrap();
}

async fn handle_scanning_for_peripherals(adapter: &Adapter) -> Vec<Peripheral> {
    // wait a bit to scan
    time::sleep(Duration::from_secs(1)).await;
    println!("scan for peripherals ended");
    adapter.peripherals().await.unwrap()
}

fn get_characteristic_with_uuid(uuid: Uuid, peripheral: &Peripheral) -> Characteristic {
    peripheral
        .characteristics()
        .into_iter()
        .find(
            |c| c.uuid == uuid, /* && c.properties == CharPropFlags::WRITE */
        )
        .expect("Control Point characteristic not found")
}
