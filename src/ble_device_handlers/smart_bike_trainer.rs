use btleplug::api::{Characteristic, Peripheral, WriteType, bleuuid::uuid_from_u16};
use futures::StreamExt;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{ble_device_handlers::*, tcp_parser};
#[derive(Clone)]
pub struct SmartTrainer {
    pub control_char: Characteristic,
    // try changing that to value
    pub peripheral: btleplug::platform::Peripheral,
}

pub(crate) fn parse_data(data: &[u8]) -> (u16, u16, u16) {
    // Seems right but not accurate to ftms spec  XD
    let wheel_rotation = u16::from_le_bytes([data[2], data[3]]);
    let cadence = u16::from_le_bytes([data[4], data[5]]);

    let power = u16::from_le_bytes([data[6], data[7]]);

    (power, cadence, wheel_rotation)
}

pub async fn handle_peripheral(
    peripheral: &btleplug::platform::Peripheral,
    tcp_writer_sender: Sender<String>,
) -> Result<()> {
    let mut notifications = peripheral.notifications().await?;
    if let Some(notification) = notifications.next().await {
        let output_data = parse_data(&notification.value);
        let current_power = output_data.0;
        let cadence = output_data.1;
        let wheel_rotation = output_data.2;

        tcp_parser::send_bike_trainer_data(
            tcp_writer_sender,
            current_power,
            cadence,
            wheel_rotation,
        )
        .await;
    }
    Ok(())
}
const FTMS_CONTROL_POINT: Uuid = uuid_from_u16(0x2AD9);
pub async fn get_device(peripheral: btleplug::platform::Peripheral) -> Result<SmartTrainer> {
    connect_to_peripheral(&peripheral)
        .await
        .context("connecting to smart trainer")?;

    info!("Connected smart trainer! {peripheral:?}");

    let control_char = get_characteristic_with_uuid(FTMS_CONTROL_POINT, &peripheral)
        .context("Control Point characteristic not found")?;
    peripheral
        .write(&control_char, &[0x00], WriteType::WithResponse)
        .await
        .context("request control command:")?;
    peripheral
        .write(&control_char, &[0x07], WriteType::WithResponse)
        .await
        .context("starting training command:")?;

    Ok(SmartTrainer {
        control_char,
        peripheral,
    })
}
pub async fn set_target_slope(data: String, trainer_device: SmartTrainer) -> Result<()> {
    let slope: i16 = data[1..data.len()].parse()?; // slope% should be * 100 so 5.12% slope -> 512

    let target = slope;
    let payload = vec![
        0x05,                         // Set Target Inclination opcode
        (target & 0xFF) as u8,        // LSB
        ((target >> 8) & 0xFF) as u8, // MSB
    ];
    trainer_device
        .peripheral
        .write(
            &trainer_device.control_char,
            &payload,
            WriteType::WithoutResponse,
        )
        .await?;

    info!("successfully set target slope: {}% ", slope as f32 / 100f32,);
    Ok(())
}
