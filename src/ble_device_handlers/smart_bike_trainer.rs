use anyhow::Error;
use btleplug::api::{Characteristic, Peripheral, WriteType, bleuuid::uuid_from_u16};
use futures::StreamExt;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::{ble_device_handlers::*, tcp_parser};
use spdlog::prelude;

pub(crate) fn parse_indoor_bike_data(data: &[u8]) -> (u16, u16) {
    // println!("data: {:?}", data);

    let flags = u16::from_le_bytes([data[0], data[1]]);
    //     println!("Flags: {:016b}", flags);

    // cadence -> obviously not right has to be something like flywheal rotation speed / resistance
    let cadence = u16::from_le_bytes([data[2], data[3]]);

    // Power -> Seems right but not accurate to ftms spec  XD
    let power = u16::from_le_bytes([data[6], data[7]]);

    (power, cadence)
}

pub async fn handle_smart_trainer_peripheral(
    control_char: &Characteristic,
    data_char: &Characteristic,
    peripheral: &btleplug::platform::Peripheral,
    stream: &mut TcpStream,
) -> Result<()> {
    let mut notifications = peripheral.notifications().await?;
    while let Some(notification) = notifications.next().await {
        let output_data = parse_indoor_bike_data(&notification.value);
        let current_power = output_data.0;
        let cadence = output_data.1;

        // println!(
        //     "output_data: current_power: {} cadence: {}",
        //     current_power, cadence
        // );
        tcp_parser::send_bike_trainer_data(stream, current_power, cadence).await;

        // println!("Send all data over tcp!");
    }
    Ok(())
}
const FTMS_CONTROL_POINT: Uuid = uuid_from_u16(0x2AD9);
const FTMS_DATA_READ_POINT: Uuid = uuid_from_u16(0x2AD2);
pub async fn get_smart_trainer_device(
    peripheral: btleplug::platform::Peripheral,
    peripheral_index: usize,
) -> Result<BleDevice> {
    info!("Connected smart trainer! {peripheral:?}");
    peripheral
        .connect()
        .await
        .context("connecting to smart trainer")?;

    peripheral
        .discover_services()
        .await
        .context("discovering smart trainer's services")?;

    let control_char = get_characteristic_with_uuid(FTMS_CONTROL_POINT, &peripheral)
        .context("Control Point characteristic not found")?;
    let data_char = get_characteristic_with_uuid(FTMS_DATA_READ_POINT, &peripheral)
        .context("Data read characteristic not found")?;

    let start_cmd = vec![0x07]; // 0x07 = Start or Resume Training
    peripheral
        .write(&control_char, &start_cmd, WriteType::WithResponse)
        .await
        .context("starting training command:")?;

    peripheral
        .subscribe(&data_char)
        .await
        .context("subscribing to trainer data notifications")?;
    Ok(BleDevice::SmartTrainer {
        control_char,
        data_char,
        peripheral_index,
    })
}

async fn set_target_power(
    target_power: u16,

    peripheral: &btleplug::platform::Peripheral,
    control_char: &Characteristic,
) {
    let cmd = vec![
        0x04, // Opcode: Set Target Power
        (target_power & 0xFF) as u8,
        (target_power >> 8) as u8,
    ];
    println!("Sending resistance command");
    let res = peripheral
        .write(&control_char, &cmd, WriteType::WithResponse)
        .await;
    match res {
        Ok(_) => println!("Write successful"),
        Err(e) => println!("Write failed: {:?}", e),
    }
}
