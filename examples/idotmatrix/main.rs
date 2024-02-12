use core::panic;

use anyhow::Result;

use esp_idf_svc::hal::peripherals::Peripherals;
use espcam::espcam::Camera;
use log::error;

use crate::idotmatrixble::idotmatrix_stream_task;

mod idotmatrixble;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let camera = Camera::new(
        peripherals.pins.gpio32,
        peripherals.pins.gpio0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio18,
        peripherals.pins.gpio19,
        peripherals.pins.gpio21,
        peripherals.pins.gpio36,
        peripherals.pins.gpio39,
        peripherals.pins.gpio34,
        peripherals.pins.gpio35,
        peripherals.pins.gpio25,
        peripherals.pins.gpio23,
        peripherals.pins.gpio22,
        peripherals.pins.gpio26,
        peripherals.pins.gpio27,
        esp_idf_sys::camera::pixformat_t_PIXFORMAT_RGB565,
        esp_idf_sys::camera::framesize_t_FRAMESIZE_240X240,
    )
    .unwrap();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            tokio::select! {
                ret = idotmatrix_stream_task(camera) => {
                    if let Err(e) = ret {
                        error!("ble_task: {}", e)
                    }
                }
            }
        });

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
