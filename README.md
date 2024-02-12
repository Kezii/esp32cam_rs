# esp32cam-rs
Rust esp32-cam examples

### Usage

populate the required .txt files, then run the examples

```
./examples/telegram_bot/bot_token.txt
./examples/telegram_bot/owner_id.txt
./wifi_ssid.txt
./wifi_pass.txt
```


## Telegram bot

```bash
cargo run --example telegram_bot

```

Insert the correct token and owner id, then use the /photo command to take a picture

<img width="480" alt="image" src="https://github.com/Kezii/esp32cam_rs/assets/3357750/5a61974f-a0dc-4bdd-94ad-81225c53ba59">

## Webserver

```bash
cargo run --example webserver
```

Connect to the ip in the log output, then access the /camera.jpg path to take a picture and have it delivered to your browser

## IDotMatrix

```bash
cargo run --example idotmatrix
```

If you have an idotmatrix display, the esp32-cam will deliver an image to it every few seconds

<img width="480" alt="image" src="https://github.com/Kezii/esp32cam_rs/assets/3357750/148e0a0e-3c06-47f0-9916-6f1ec76d67e5">


## credits:
https://github.com/esp-rs/std-training

https://github.com/jlocash/esp-camera-rs
