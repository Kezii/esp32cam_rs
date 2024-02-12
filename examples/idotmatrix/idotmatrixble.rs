use crate::Camera;
use anyhow::Result;
use bstr::ByteSlice;
use esp32_nimble::{uuid128, BLEClient, BLEDevice, BLEReturnCode};
use esp_idf_sys::camera;
use espcam::espcam::FrameBuffer;
use image::{ImageBuffer, ImageOutputFormat, Rgb};
use log::{error, info};

pub struct IDMBle<'a> {
    characteristic: &'a mut esp32_nimble::BLERemoteCharacteristic,
}

impl<'a> IDMBle<'a> {
    pub async fn new(
        ble_device: &'a BLEDevice,
        client: &'a mut BLEClient,
    ) -> Result<Self, BLEReturnCode> {
        let ble_scan = ble_device.get_scan();

        info!("Scanning for BLE devices...");

        let device = ble_scan
            .active_scan(true)
            .interval(100)
            .window(99)
            .find_device(1000, |device| device.name().contains_str("IDM"))
            .await?;

        if let Some(device) = device {
            client.on_connect(|client| {
                client.update_conn_params(120, 120, 0, 60).unwrap();
            });

            client.connect(device.addr()).await?;

            let service = client
                .get_service(uuid128!("000000fa-0000-1000-8000-00805f9b34fb"))
                .await?;

            let uuid = uuid128!("0000fa02-0000-1000-8000-00805f9b34fb");
            let characteristic = service.get_characteristic(uuid).await?;

            Ok(Self { characteristic })
        } else {
            error!("No device found");
            Err(BLEReturnCode::fail().unwrap_err())
        }
    }

    pub async fn send_data(&mut self, bytes: &[u8]) -> Result<(), BLEReturnCode> {
        for (counter, chunk) in bytes.chunks(512).enumerate() {
            let succ = self.characteristic.write_value(chunk, true).await;
            info!("progress: {}%", (counter * chunk.len()) * 100 / bytes.len());

            if let Err(succ) = succ {
                error!("upload png command error: {:?}", succ);
                return Err(succ);
            }
        }

        Ok(())
    }
}

pub async fn idotmatrix_stream_task(camera: Camera<'_>) -> Result<()> {
    let ble_device = BLEDevice::take();
    let mut ble_client = BLEClient::new();

    let mut ble_handler = IDMBle::new(ble_device, &mut ble_client).await.unwrap();

    ble_handler
        .send_data(&idotmatrix::IDMCommand::ImageMode(1).to_bytes())
        .await
        .unwrap();

    loop {
        camera.get_framebuffer();
        let framebuffer = camera.get_framebuffer();

        if let Some(framebuffer) = framebuffer {
            info!("Creating image");

            let img = framebuffer_to_img(framebuffer);

            info!("Resizing image");
            let scaled =
                image::imageops::resize(&img, 32, 32, image::imageops::FilterType::Lanczos3);

            let mut c = std::io::Cursor::new(Vec::new());

            info!("Writing png");
            scaled.write_to(&mut c, ImageOutputFormat::Png).unwrap();

            info!("Creating command");
            let command = idotmatrix::IDMCommand::UploadPng(c.into_inner());

            info!("Sending command");

            ble_handler.send_data(&command.to_bytes()).await.unwrap();

            //tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        } else {
            log::info!("no framebuffer");
        }
    }
    //client.disconnect().unwrap();

    //ble::ble_advertise_task(name, ble_server, ble_advertising).await;
}

fn framebuffer_to_img(framebuffer: FrameBuffer<'_>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let data = framebuffer.data();
    ImageBuffer::from_fn(
        framebuffer.width() as u32,
        framebuffer.height() as u32,
        |x, y| match framebuffer.format() {
            camera::pixformat_t_PIXFORMAT_RGB565 => {
                let pix_addr = (x + y * framebuffer.width() as u32) as usize * 2;
                let raw_pixel = u16::from_be_bytes([data[pix_addr], data[pix_addr + 1]]);

                let decoded = rgb565::Rgb565::unpack_565(raw_pixel);

                Rgb([decoded.0, decoded.1, decoded.2])
            }

            camera::pixformat_t_PIXFORMAT_GRAYSCALE => {
                let pix_addr = (x + y * framebuffer.width() as u32) as usize;
                let raw_pixel = data[pix_addr];

                Rgb([raw_pixel, raw_pixel, raw_pixel])
            }

            _ => {
                panic!("unsupported format");
            }
        },
    )
}
