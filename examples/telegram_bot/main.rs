use anyhow::Result;

use bot_api::{telegram_post_multipart, Esp32Api};
use esp_idf_hal::gpio::PinDriver;
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::peripherals::Peripherals};
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
    should_use_flash: false,
    public_use: false,
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

    let _wifi = match my_wifi(
        config.wifi_ssid,
        config.wifi_psk,
        peripherals.modem,
        sysloop,
    ) {
        Ok(inner) => inner,
        Err(err) => {
            flash_led.set_high().unwrap();

            error!("Could not connect to Wi-Fi network: {:?}", err);
            unsafe { esp_restart() };
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

    let mut bot_state = BotState {
        config: DEFAULT_CONFIG,
        owner_id: config.bot_owner_id,
        bot_token: config.bot_token,
    };

    let api = Esp32Api::new(bot_state.bot_token);

    api.send_message(
        &SendMessageParams::builder()
            .chat_id(bot_state.owner_id)
            .text("Starting!")
            .build(),
    )
    .ok();

    let updates = api
        .get_updates(&GetUpdatesParams::builder().limit(1u32).offset(-1).build())
        .unwrap();

    let mut offset = if let Some(update) = updates.result.first() {
        update.update_id + 1
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
            offset = update.update_id + 1;

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
                    "/start" => {
                        api.send_message(
                            &SendMessageParams::builder()
                                .chat_id(message.chat.id)
                                .text("Hello!")
                                .build(),
                        )
                        .ok();
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
