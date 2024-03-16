use log::info;
#[derive(Debug)]
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    bot_token: &'static str,
    #[default(0)]
    bot_owner_id: i64,
}

pub fn get_config() -> Config {
    info!("got config {:#?}", CONFIG);

    if CONFIG.wifi_ssid.is_empty() && CONFIG.wifi_psk.is_empty() {
        panic!("WiFi SSID and PSK are not set\nif you use --target-dir the toml was not loaded, check this issue https://github.com/jamesmunns/toml-cfg/issues/7");
    }

    CONFIG
}
