mod logger;
mod tcp;

use btleplug::api::CharPropFlags;
use btleplug::api::Characteristic;

use btleplug::api::{
    Central, Manager as _, Peripheral as _, ScanFilter, WriteType, bleuuid::uuid_from_u16,
};
use btleplug::platform::{Adapter, Manager, Peripheral};

use futures::stream::StreamExt;
use std::error::Error;
use std::io::stdin;
use std::time::Duration;
use tcp::create_stream;
use tcp::send_tcp_data;
use tokio::time;
use uuid::Uuid;

pub struct OutputData {
    current_power: u16, // instantaneous watts
    cadence: u16,       //rpm
}

const FTMS_CONTROL_POINT: Uuid = uuid_from_u16(0x2AD9);
const FTMS_DATA_READ_POINT: Uuid = uuid_from_u16(0x2AD2);
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let selected_peripheral = handle_peripherals_selection().await;
    selected_peripheral.connect().await?;
    selected_peripheral.discover_services().await?;

    println!("Connected!");

    let control_char = get_characteristic_with_uuid(FTMS_CONTROL_POINT, &selected_peripheral);
    let data_char = get_characteristic_with_uuid(FTMS_DATA_READ_POINT, &selected_peripheral);

    println!("Created characteristics");

    let start_cmd = vec![0x07]; // 0x07 = Start or Resume Training
    selected_peripheral
        .write(&control_char, &start_cmd, WriteType::WithResponse)
        .await?;

    selected_peripheral.subscribe(&data_char).await.unwrap();
    println!("Subscribed to Indoor Bike Data notifications.");

    let mut stream = create_stream().await;
    loop {
        let mut notifications = selected_peripheral.notifications().await?;

        println!("Waiting for data...");

        while let Some(notification) = notifications.next().await {
            let output_data = parse_indoor_bike_data(&notification.value);
            println!(
                "output_data: current_power: {} cadence: {}",
                output_data.current_power, output_data.cadence
            );

            send_tcp_data(&mut stream, format!("p{}", output_data.current_power)).await;
            send_tcp_data(&mut stream, format!("c{}", output_data.cadence)).await;

            println!("Send all data over tcp!");
        }
    }

    Ok(())
}
async fn set_target_power(
    target_power: u16,
    selected_peripheral: &Peripheral,
    control_char: &Characteristic,
) {
    let cmd = vec![
        0x04, // Opcode: Set Target Power
        (target_power & 0xFF) as u8,
        (target_power >> 8) as u8,
    ];
    println!("Sending resistance command");
    let res = selected_peripheral
        .write(&control_char, &cmd, WriteType::WithResponse)
        .await;
    match res {
        Ok(_) => println!("Write successful"),
        Err(e) => println!("Write failed: {:?}", e),
    }
}
async fn handle_peripherals_selection() -> Peripheral {
    let manager = Manager::new().await.unwrap();

    // get the first bluetooth adapter
    let adapters = manager.adapters().await.unwrap();
    let central = adapters.into_iter().next().unwrap();

    central.start_scan(ScanFilter::default()).await.unwrap();
    // wait a bit to scan
    time::sleep(Duration::from_secs(1)).await;

    let peripherals = central.peripherals().await.unwrap();
    let mut i = 0;
    for peripheral in &peripherals {
        println!("{i}: {} ", peripheral);
        i += 1;
    }
    println!("enter peripheral index");
    let mut buffer = String::new();

    stdin().read_line(&mut buffer).unwrap();
    let index = buffer.trim().parse::<usize>().unwrap();
    peripherals[index].to_owned()
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
fn parse_indoor_bike_data(data: &[u8]) -> OutputData {
    println!("data: {:?}", data);

    let flags = u16::from_le_bytes([data[0], data[1]]);
    //     println!("Flags: {:016b}", flags);

    // cadence -> obviously not right has to be something like flywheal rotation speed / resistance
    let cadence = u16::from_le_bytes([data[2], data[3]]);

    // Power -> Seems right but not accurate to ftms spec  XD
    let power = u16::from_le_bytes([data[6], data[7]]);

    OutputData {
        current_power: power,
        cadence,
    }
}
