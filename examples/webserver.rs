use anyhow::{bail, Result};

use esp_idf_hal::io::Write;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    http::{server::EspHttpServer, Method},
};
use espcam::{espcam::Camera, wifi_handler::my_wifi};

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let sysloop = EspSystemEventLoop::take()?;

    let peripherals = Peripherals::take().unwrap();

    let wifi_ssid = include_str!("../wifi_ssid.txt");
    let wifi_pass = include_str!("../wifi_pass.txt");

    let _wifi = match my_wifi(wifi_ssid, wifi_pass, peripherals.modem, sysloop) {
        Ok(inner) => inner,
        Err(err) => {
            bail!("Could not connect to Wi-Fi network: {:?}", err)
        }
    };

    let camera = Camera::new(
        /* PWDN */ peripherals.pins.gpio33, // Adjust X as needed
        /* XCLK */ peripherals.pins.gpio21,
        /* D0 (Y2) */ peripherals.pins.gpio4,
        /* D1 (Y3) */ peripherals.pins.gpio5,
        /* D2 (Y4) */ peripherals.pins.gpio18,
        /* D3 (Y5) */ peripherals.pins.gpio19,
        /* D4 (Y6) */ peripherals.pins.gpio36,
        /* D5 (Y7) */ peripherals.pins.gpio39,
        /* D6 (Y8) */ peripherals.pins.gpio34,
        /* D7 (Y9) */ peripherals.pins.gpio35,
        /* VSYNC */ peripherals.pins.gpio25,
        /* HREF */ peripherals.pins.gpio23,
        /* PCLK */ peripherals.pins.gpio22,
        /* SDA (SIOD) */ peripherals.pins.gpio26,
        /* SCL (SIOC) */ peripherals.pins.gpio27,
        esp_idf_sys::camera::pixformat_t_PIXFORMAT_JPEG,
        esp_idf_sys::camera::framesize_t_FRAMESIZE_UXGA,
    ).unwrap();

    let mut server = EspHttpServer::new(&esp_idf_svc::http::server::Configuration::default())?;

    server.fn_handler("/camera.jpg", Method::Get, |request| {
        let framebuffer = camera.get_framebuffer();

        if let Some(framebuffer) = framebuffer {
            let data = framebuffer.data();

            let headers = [
                ("Content-Type", "image/jpeg"),
                ("Content-Length", &data.len().to_string()),
            ];
            let mut response = request.into_response(200, Some("OK"), &headers).unwrap();
            response.write_all(data)?;
        } else {
            let mut response = request.into_ok_response()?;
            response.write_all("no framebuffer".as_bytes())?;
        }

        Ok::<(), anyhow::Error>(())
    })?;

    server.fn_handler("/", Method::Get, |request| {
        let mut response = request.into_ok_response()?;
        response.write_all("ok".as_bytes())?;
        Ok::<(), anyhow::Error>(())
    })?;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
