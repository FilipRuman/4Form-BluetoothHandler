use std::time::Duration;

use btleplug::{
    api::{Central, Characteristic, Manager, Peripheral, ScanFilter},
    platform::Adapter,
};
use tokio::time;
use uuid::Uuid;

pub(crate) async fn start_scan() -> Adapter {
    let manager = btleplug::platform::Manager::new().await.unwrap();
    // get the first bluetooth adapter
    let adapters = manager.adapters().await.unwrap();
    let central = adapters.into_iter().next().unwrap();

    central.start_scan(ScanFilter::default()).await.unwrap();

    println!("started scanning");
    central
}
pub(crate) async fn connect_to_peripheral(selected_peripheral: btleplug::platform::Peripheral) {
    selected_peripheral.connect().await.unwrap();
    selected_peripheral.discover_services().await.unwrap();
}

pub async fn handle_scanning_for_peripherals(
    adapter: &Adapter,
) -> Vec<btleplug::platform::Peripheral> {
    // wait a bit to scan
    time::sleep(Duration::from_secs(1)).await;
    println!("scan for peripherals ended");
    adapter.peripherals().await.unwrap()
}

pub fn get_characteristic_with_uuid(
    uuid: Uuid,
    peripheral: &btleplug::platform::Peripheral,
) -> Characteristic {
    peripheral
        .characteristics()
        .into_iter()
        .find(
            |c| c.uuid == uuid, /* && c.properties == CharPropFlags::WRITE */
        )
        .expect("Control Point characteristic not found")
}
