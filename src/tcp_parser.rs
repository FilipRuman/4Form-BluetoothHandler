use std::{collections::HashSet, default, thread::sleep, time::Duration};

use crate::{handle_smart_trainer_peripheral, tcp};
use btleplug::{api::Peripheral, platform::PeripheralId};
use regex::{self, Regex};
use tokio::net::TcpStream;
pub async fn send_peripherals(
    stream: &mut TcpStream,
    old_peripherals_len: &mut usize,
    peripherals: &[btleplug::platform::Peripheral],
    old_peripherals_ids: &mut HashSet<PeripheralId>,
) {
    for i in old_peripherals_len.to_owned()..peripherals.len() {
        let peripheral = &peripherals[i];
        if !old_peripherals_ids.insert(peripheral.id()) {
            continue;
        }
        let properties = peripheral.properties().await.unwrap().unwrap();

        let name = match properties.local_name {
            Some(txt) => txt,
            None => "unknown".to_string(),
        };

        tcp::send_tcp_data(stream, format!("i[{}]|{}|", name, i)).await;
        // have to wait some time when sending multiple packages so they don't stack up to one on
        // the c# side
        sleep(Duration::from_millis(5));
        println!(
            "send_peripherals: peripheral id:{} {:?}",
            peripheral.id(),
            peripheral
        );
    }
}
