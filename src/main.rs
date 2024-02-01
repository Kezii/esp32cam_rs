use anyhow::{bail, Result};

use esp_idf_hal::gpio::PinDriver;
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::peripherals::Peripherals};
use espcam_test::{espcam::Camera, wifi_handler::my_wifi};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let sysloop = EspSystemEventLoop::take()?;

    let peripherals = Peripherals::take().unwrap();

    let wifi_ssid = include_str!("../wifi_ssid.txt");
    let wifi_pass = include_str!("../wifi_pass.txt");

    let mut flash_led = PinDriver::output(peripherals.pins.gpio4).unwrap();
    flash_led.set_low().unwrap();

    let _wifi = match my_wifi(wifi_ssid, wifi_pass, peripherals.modem, sysloop) {
        Ok(inner) => inner,
        Err(err) => {
            bail!("Could not connect to Wi-Fi network: {:?}", err)
        }
    };

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
        esp_idf_sys::camera::pixformat_t_PIXFORMAT_JPEG,
        esp_idf_sys::camera::framesize_t_FRAMESIZE_UXGA,
    )
    .unwrap();

    loop {
        let framebuffer = camera.get_framebuffer();

        if let Some(framebuffer) = framebuffer {
            println!("Got framebuffer!");
            println!("width: {}", framebuffer.width());
            println!("height: {}", framebuffer.height());
            println!("len: {}", framebuffer.data().len());
            println!("format: {}", framebuffer.format());

            std::thread::sleep(std::time::Duration::from_millis(1000));
        } else {
            log::info!("no framebuffer");
        }
    }
}
