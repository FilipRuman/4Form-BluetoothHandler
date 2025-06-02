pub(crate) mod smart_bike_trainer;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use btleplug::{
    api::{Central, Characteristic, Manager, Peripheral, ScanFilter},
    platform::Adapter,
};
use smart_bike_trainer::handle_smart_trainer_peripheral;
use spdlog::prelude::*;
use tokio::{net::TcpStream, time};
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

pub async fn handle_devices(
    devices: &Vec<BleDevice>,
    valid_peripherals: &[btleplug::platform::Peripheral],
    stream: &mut TcpStream,
) {
    for device in devices {
        match device {
            BleDevice::SmartTrainer {
                control_char,
                data_char,
                peripheral_index,
            } => {
                match handle_smart_trainer_peripheral(
                    control_char,
                    data_char,
                    &valid_peripherals[peripheral_index.to_owned()],
                    stream,
                )
                .await
                {
                    Ok(_) => {}
                    Err(error) => {
                        error!("handler of smart trainer peripheral returned error: {error}");
                    }
                };
            }
        }
    }
}
pub async fn start_scan() -> Result<Adapter> {
    let manager = btleplug::platform::Manager::new().await?;
    // get the first bluetooth adapter
    let adapters = manager.adapters().await.context("getting adapters")?;
    let central = adapters.into_iter().next().unwrap();

    central
        .start_scan(ScanFilter::default())
        .await
        .context("starting scanning")?;

    info!("started scanning");
    Ok(central)
}
pub async fn connect_to_peripheral(
    selected_peripheral: &btleplug::platform::Peripheral,
) -> Result<()> {
    selected_peripheral
        .connect()
        .await
        .context("connecting to device did not succeed")?;
    selected_peripheral
        .discover_services()
        .await
        .context("discovering devices services did not succeed")?;
    Ok(())
}

pub async fn get_found_peripherals(adapter: &Adapter) -> Vec<btleplug::platform::Peripheral> {
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
        .ok_or(anyhow!(""))
}
