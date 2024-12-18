use anyhow::Result;

use bot_api::{telegram_post_multipart, Esp32Api};
use esp_idf_hal::{gpio::PinDriver, io::Write};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    http::{server::EspHttpServer, Method},
};
use esp_idf_sys::esp_restart;
use espcam::{config::get_config, espcam::Camera, wifi_handler::my_wifi};
use frankenstein::{
    ForwardMessageParams, GetUpdatesParams, SendChatActionParams, SendMessageParams, TelegramApi,
};
use log::{error, info};

mod bot_api;

struct BotConfiguration {
    should_use_flash: bool,
    public_use: bool,
}

const DEFAULT_CONFIG: BotConfiguration = BotConfiguration {
    should_use_flash: true,
    public_use: true,
};

struct BotState {
    config: BotConfiguration,
    owner_id: i64,
    bot_token: &'static str,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let sysloop = EspSystemEventLoop::take()?;

    let peripherals = Peripherals::take().unwrap();

    let mut flash_led = PinDriver::output(peripherals.pins.gpio4).unwrap();
    flash_led.set_low().unwrap();

    let config = get_config();

    let wifi = match my_wifi(
        config.wifi_ssid,
        config.wifi_psk,
        peripherals.modem,
        sysloop,
    ) {
        Ok(inner) => inner,
        Err(err) => {
            error!("Could not connect to Wi-Fi network: {:?}", err);

            for _ in 0..5 {
                flash_led.set_high().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(1));
                flash_led.set_low().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(80));
            }

            unsafe { esp_restart() };
        }
    };

    flash_led.set_high().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1));
    flash_led.set_low().unwrap();

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

    let camera = std::sync::Arc::new(camera);

    let mut server = EspHttpServer::new(&esp_idf_svc::http::server::Configuration::default())?;

    let camera2 = camera.clone();

    server.fn_handler("/camera.jpg", Method::Get, move |request| {
        camera2.get_framebuffer();
        // take two frames to get a fresh one
        let framebuffer = camera2.get_framebuffer();

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

        Ok::<(), esp_idf_hal::io::EspIOError>(())
    })?;

    let mut bot_state = BotState {
        config: DEFAULT_CONFIG,
        owner_id: config.bot_owner_id,
        bot_token: config.bot_token,
    };

    let api = Esp32Api::new(bot_state.bot_token);

    let send_owner_info = |bot_state: &BotState| {
        let mut rssi = 0;
        unsafe {
            esp_idf_sys::esp_wifi_sta_get_rssi(&mut rssi);
        }
        api.send_message(
            &SendMessageParams::builder()
                .chat_id(bot_state.owner_id)
                .text(format!(
                    "Camera OK!\nUse /publish to toggle public use!\nIP: {}\nRSSI: {}\nflash: {}\npublic use: {}",
                    wifi.sta_netif().get_ip_info().unwrap().ip,
                    rssi,
                    bot_state.config.should_use_flash,
                    bot_state.config.public_use
                ))
                .build(),
        )
        .ok();
    };

    send_owner_info(&bot_state);

    let updates = api
        .get_updates(&GetUpdatesParams::builder().limit(1u32).offset(-1).build())
        .unwrap();

    let mut offset = if let Some(update) = updates.result.first() {
        update.update_id as i64 + 1
    } else {
        0
    };

    loop {
        let updates = api
            .get_updates(
                &GetUpdatesParams::builder()
                    .timeout(120u32)
                    .limit(1u32)
                    .offset(offset)
                    .build(),
            )
            .unwrap();

        for update in updates.result {
            offset = update.update_id as i64 + 1;

            if let frankenstein::UpdateContent::Message(message) = update.content {
                info!(
                    "message id {} from chat {}",
                    message.message_id, message.chat.id
                );

                match message.text.unwrap_or_default().as_str() {
                    "/photo" => {
                        if message.chat.id != bot_state.owner_id && !bot_state.config.public_use {
                            continue;
                        }

                        api.send_chat_action(
                            &SendChatActionParams::builder()
                                .chat_id(message.chat.id)
                                .action(frankenstein::ChatAction::UploadPhoto)
                                .build(),
                        )
                        .ok();

                        if bot_state.config.should_use_flash {
                            flash_led.set_high().unwrap();
                        }

                        camera.get_framebuffer();
                        let framebuffer = camera.get_framebuffer();

                        flash_led.set_low().unwrap();

                        if let Some(framebuffer) = framebuffer {
                            let res = telegram_post_multipart(
                                format!(
                                    "https://api.telegram.org/bot{}/sendPhoto",
                                    bot_state.bot_token
                                ),
                                framebuffer.data(),
                                message.chat.id,
                            );

                            match res {
                                Ok(_) => {}
                                Err(err) => {
                                    error!("http_get error: {:?}", err);
                                }
                            }
                        } else {
                            log::info!("no framebuffer");
                        }
                    }

                    "/flash" => {
                        if message.chat.id != bot_state.owner_id && !bot_state.config.public_use {
                            continue;
                        }
                        if bot_state.config.should_use_flash {
                            bot_state.config.should_use_flash = false;

                            api.send_message(
                                &SendMessageParams::builder()
                                    .chat_id(message.chat.id)
                                    .text("Flash disabled!")
                                    .build(),
                            )
                            .unwrap();
                        } else {
                            bot_state.config.should_use_flash = true;

                            api.send_message(
                                &SendMessageParams::builder()
                                    .chat_id(message.chat.id)
                                    .text("Flash enabled!")
                                    .build(),
                            )
                            .unwrap();
                        }
                    }

                    "/publish" => {
                        if message.chat.id != bot_state.owner_id {
                            continue;
                        }
                        if bot_state.config.public_use {
                            bot_state.config.public_use = false;

                            api.send_message(
                                &SendMessageParams::builder()
                                    .chat_id(message.chat.id)
                                    .text("Public use disabled!")
                                    .build(),
                            )
                            .unwrap();
                        } else {
                            bot_state.config.public_use = true;

                            api.send_message(
                                &SendMessageParams::builder()
                                    .chat_id(message.chat.id)
                                    .text("Public use enabled!")
                                    .build(),
                            )
                            .unwrap();
                        }
                    }
                    "/start" | "/help" => {
                        api.send_message(
                            &SendMessageParams::builder()
                                .chat_id(message.chat.id)
                                .text("Hello!\nUse /photo to take a photo!\nUse /flash to toggle flash!")
                                .build(),
                        )
                        .ok();

                        if message.chat.id == bot_state.owner_id {
                            send_owner_info(&bot_state);
                        }
                    }
                    _ => {}
                }

                if message.chat.type_field == frankenstein::ChatType::Private {
                    api.forward_message(
                        &ForwardMessageParams::builder()
                            .chat_id(bot_state.owner_id)
                            .from_chat_id(message.chat.id)
                            .message_id(message.message_id)
                            .build(),
                    )
                    .ok();
                }
            }
        }
    }
}
