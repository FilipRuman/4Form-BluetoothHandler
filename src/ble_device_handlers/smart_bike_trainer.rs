use anyhow::Ok;
use btleplug::api::{Characteristic, Peripheral, WriteType, bleuuid::uuid_from_u16};
use futures::StreamExt;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::{ble_device_handlers::*, tcp_parser};

pub(crate) fn parse_data(data: &[u8]) -> (u16, u16, u16) {
    // Seems right but not accurate to ftms spec  XD
    let wheel_rotation = u16::from_le_bytes([data[2], data[3]]);
    let cadence = u16::from_le_bytes([data[4], data[5]]);

    let power = u16::from_le_bytes([data[6], data[7]]);

    (power, cadence, wheel_rotation)
}

pub async fn handle_peripheral(
    peripheral: &btleplug::platform::Peripheral,
    stream: &mut TcpStream,
    control_char: &Characteristic,
) -> Result<()> {
    let mut notifications = peripheral.notifications().await?;
    while let Some(notification) = notifications.next().await {
        let output_data = parse_data(&notification.value);
        let current_power = output_data.0;
        let cadence = output_data.1;
        let wheel_rotation = output_data.2;

        tcp_parser::send_bike_trainer_data(stream, current_power, cadence, wheel_rotation).await;
    }
    Ok(())
}
const FTMS_CONTROL_POINT: Uuid = uuid_from_u16(0x2AD9);
const FTMS_DATA_READ_POINT: Uuid = uuid_from_u16(0x2AD2);
pub async fn get_device(peripheral: btleplug::platform::Peripheral) -> Result<BleDevice> {
    connect_to_peripheral(&peripheral)
        .await
        .context("connecting to smart trainer")?;

    info!("Connected smart trainer! {peripheral:?}");

    let control_char = get_characteristic_with_uuid(FTMS_CONTROL_POINT, &peripheral)
        .context("Control Point characteristic not found")?;
    let data_char = get_characteristic_with_uuid(FTMS_DATA_READ_POINT, &peripheral)
        .context("Data read characteristic not found")?;

    peripheral
        .subscribe(&data_char)
        .await
        .context("subscribing to trainer data notifications")?;

    peripheral
        .write(&control_char, &[0x00], WriteType::WithResponse)
        .await
        .context("request control command:")?;
    peripheral
        .write(&control_char, &[0x07], WriteType::WithResponse)
        .await
        .context("starting training command:")?;
    set_target_slope(20, &peripheral, &control_char)
        .await
        .context("setting target slope")?;
    Ok(BleDevice::SmartTrainer {
        control_char,
        data_char,
        peripheral,
    })
}
pub async fn set_target_slope(
    slope_percent: i16,
    peripheral: &btleplug::platform::Peripheral,
    control_char: &Characteristic,
) -> Result<()> {
    let target = slope_percent;
    let payload = vec![
        0x05,                         // Set Target Inclination opcode
        (target & 0xFF) as u8,        // LSB
        ((target >> 8) & 0xFF) as u8, // MSB
    ];
    let response = peripheral
        .write(control_char, &payload, WriteType::WithResponse)
        .await?;
    info!(
        "successfully set target slope: {}% response: {:?}",
        slope_percent, response
    );
    Ok(())
}
