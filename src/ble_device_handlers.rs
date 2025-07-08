pub(crate) mod hr_tracker;
pub(crate) mod smart_bike_trainer;

use anyhow::{Context, Result, anyhow};
use btleplug::{
    api::{Central, Characteristic, Manager, Peripheral, ScanFilter},
    platform::Adapter,
};
use spdlog::prelude::*;
use uuid::Uuid;

#[derive(Clone)]
pub struct DeviceContainer {
    pub hr_tracker: Option<hr_tracker::HRTracker>,
    pub smart_trainer: Option<smart_bike_trainer::SmartTrainer>,
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
pub(super) async fn connect_to_peripheral(
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

pub fn get_characteristic_with_uuid(
    uuid: Uuid,
    peripheral: &btleplug::platform::Peripheral,
) -> Result<Characteristic> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == uuid)
        .ok_or(anyhow!(""))
}
