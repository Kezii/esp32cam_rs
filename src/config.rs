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
    CONFIG
}
