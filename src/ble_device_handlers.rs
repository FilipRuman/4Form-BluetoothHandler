pub(crate) mod smart_bike_trainer;
use std::time::Duration;

use anyhow::{Context, Result};
use btleplug::{
    api::{Central, Characteristic, Manager, Peripheral, ScanFilter},
    platform::Adapter,
};
use spdlog::prelude::*;
use tokio::time;
use uuid::Uuid;
#[derive(Debug)]
pub enum BleDevice {
    SmartTrainer {
        control_char: Characteristic,
        data_char: Characteristic,
        // try changing that to value
        peripheral_index: usize,
    },
}

pub async fn start_scan() -> Result<Adapter> {
    let manager = btleplug::platform::Manager::new().await?;
    // get the first bluetooth adapter
    let adapters = manager.adapters().await.context("geting adapters")?;
    let central = adapters.into_iter().next().unwrap();

    central
        .start_scan(ScanFilter::default())
        .await
        .context("starting scanning")?;

    info!("started scanning");
    Ok(central)
}
pub async fn connect_to_peripheral(selected_peripheral: btleplug::platform::Peripheral) {
    if selected_peripheral.connect().await.is_err() {
        error!("connecting to device did not succed");
    }
    if selected_peripheral.discover_services().await.is_err() {
        error!("discovering devices services did not suceed");
    }
}

pub async fn get_found_peripherials(adapter: &Adapter) -> Vec<btleplug::platform::Peripheral> {
    // wait a bit to scan
    time::sleep(Duration::from_secs(1)).await;
    adapter.peripherals().await.unwrap()
}

pub fn get_characteristic_with_uuid(
    uuid: Uuid,
    peripheral: &btleplug::platform::Peripheral,
) -> Result<Characteristic> {
    peripheral
        .characteristics()
        .into_iter()
        .find(
            |c| c.uuid == uuid, /* && c.properties == CharPropFlags::WRITE */
        )
        .context("Control Point characteristic not found")
}
