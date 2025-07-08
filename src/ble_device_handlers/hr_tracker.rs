use btleplug::api::Characteristic;
use tokio::sync::mpsc::Sender;

use crate::ble_device_handlers::connect_to_peripheral;
use crate::tcp::tcp_parser;
use anyhow::Context;
use anyhow::Ok;
use anyhow::Result;
use btleplug::api::Peripheral;
use futures::StreamExt;
use spdlog::info;
use uuid::Uuid;
#[derive(Clone)]
pub struct HRTracker {
    hr_char: Characteristic,
    // try changing that to value
    peripheral: btleplug::platform::Peripheral,
}

pub async fn get_device(peripheral: btleplug::platform::Peripheral) -> Result<HRTracker> {
    connect_to_peripheral(&peripheral)
        .await
        .context("connecting to smart watch")?;
    info!("Connected smart watch! {peripheral:?}");

    peripheral.discover_services().await?;
    // hr - heart rate
    let hr_service_uuid = Uuid::from_u128(0x0000180d_0000_1000_8000_00805f9b34fb);
    // find heart rate service
    let services = peripheral.services();

    let hr_service = services
        .iter()
        .find(|s| s.uuid == hr_service_uuid)
        .context("peripheral doesn't have heart rate service")?;

    let hr_measurement_char_uuid = Uuid::from_u128(0x00002a37_0000_1000_8000_00805f9b34fb);
    let hr_char = hr_service
        .characteristics
        .iter()
        .find(|c| c.uuid == hr_measurement_char_uuid)
        .context("heart rate service doesn't have needed characteristic")?;
    peripheral.subscribe(hr_char).await?;

    Ok(HRTracker {
        hr_char: hr_char.to_owned(),
        peripheral,
    })
}

pub async fn handle_peripheral(
    peripheral: &btleplug::platform::Peripheral,

    mut tcp_writer_sender: Sender<String>,
) -> Result<()> {
    let mut notifications = peripheral.notifications().await?;
    while let Some(notification) = notifications.next().await {
        let option_output = parse_data(&notification.value);

        if let Some(hr) = option_output {
            tcp_parser::send_smart_watch_data(&mut tcp_writer_sender, hr).await;
        }
    }
    Ok(())
}
pub(crate) fn parse_data(data: &[u8]) -> Option<u8> {
    if data.is_empty() {
        return None;
    }
    let flag = data[0];

    return if flag & 0x01 == 0 {
        data.get(1).copied() // 8-bit HR
    } else {
        // 16-bit HR, rarely used
        Some(u16::from_le_bytes([data[1], data[2]]) as u8)
    };
}
