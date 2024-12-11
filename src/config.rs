#[derive(Debug)]
pub struct Config {
    pub wifi_ssid: &'static str,
    pub wifi_psk: &'static str,
    pub bot_token: &'static str,
    pub bot_owner_id: i64,
}

include!("wifi_config.rs");
