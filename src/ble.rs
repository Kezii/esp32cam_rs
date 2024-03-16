use std::time::Duration;

use anyhow::bail;
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::{BLEAdvertising, BLEScan, BLEServer, NimbleProperties};
use lazy_static::lazy_static;
use log::{info, warn};
use tokio::select;
use tokio::time::sleep;

use anyhow::anyhow;

pub static UUID_BLE_SERVICE_STR: &str = "io.test.ble"; // up-to 16 bytes
pub static UUID_BLE_UPTIME_CHARA_STR: &str = "uptime"; // up-to 16 bytes

lazy_static! {
    pub static ref UUID_BLE_SERVICE: BleUuid = str_to_uuid(UUID_BLE_SERVICE_STR);
    pub static ref UUID_BLE_UPTIME_CHARA: BleUuid = str_to_uuid(UUID_BLE_UPTIME_CHARA_STR);
}

pub fn str_to_uuid(s: &str) -> BleUuid {
    let mut arr = [0u8; 16];
    for (idx, char) in s.as_bytes().iter().enumerate() {
        if idx < 16 {
            arr[idx] = *char
        } else {
            warn!("uuid string is longer than 16 bytes!");
            break;
        }
    }
    BleUuid::from_uuid128(arr)
}

pub async fn ble_advertise_task(
    name: &str,
    server: &mut BLEServer,
    advertising: &mut BLEAdvertising,
) {
    server.on_connect(|server, desc| {
        server
            .update_conn_params(desc.conn_handle(), 24, 48, 0, 60)
            .expect("server.update_conn_params");
    });
    //    server.on_disconnect(|desc, reason| {
    //        info!("Client disconnected ({:X})", reason);
    //    });
    let service = server.create_service(*UUID_BLE_SERVICE);

    let notifying_characteristic = service.lock().create_characteristic(
        *UUID_BLE_UPTIME_CHARA,
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );
    notifying_characteristic.lock().set_value(b"uptime: 0");

    advertising.name(name).add_service_uuid(*UUID_BLE_SERVICE);
    // advertising
    //     .set_data(
    //         BLEAdvertisementData::new()
    //             .name(name)
    //             .add_service_uuid(*UUID_BLE_SERVICE),
    //     )
    //     .unwrap();

    advertising.start().expect("ble_advertising.start()");

    let mut counter: u128 = 0;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let mut guard = notifying_characteristic.lock();
        guard
            .set_value(format!("uptime: {counter}").as_bytes())
            .notify();
        drop(guard);

        counter += 1;
    }
}

pub async fn do_ble_scan(
    ble_scan: &mut BLEScan,
) -> Result<Vec<esp32_nimble::BLEAdvertisedDevice>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let mut tx = Some(tx);

    ble_scan
        .active_scan(true)
        .filter_duplicates(true)
        .limited(false)
        .interval(100)
        .window(99)
        .on_completed(move || {
            let tx = tx.take();
            if let Some(tx) = tx {
                let _ = tx.send(());
            }
        });
    ble_scan
        .start(10000)
        .await
        .map_err(|e| anyhow!("ble_scan.start: {:?}", e))?;

    select! {
        _ = sleep(Duration::from_secs(15)) => {
            bail!("ble scan timed out!");
        }
        _ = rx => {
            info!("Scan finished");
        }
    };
    let result = ble_scan.get_results().cloned().collect::<Vec<_>>();

    Ok(result)
}
