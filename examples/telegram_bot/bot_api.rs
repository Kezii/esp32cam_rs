use embedded_svc::http::{client::Client, Method};
use esp_idf_hal::io::EspIOError;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};

pub fn http_post(url: impl AsRef<str>, data: &[u8]) -> Result<Vec<u8>, EspBotError> {
    // 1. Create a new EspHttpConnection with default Configuration. (Check documentation)
    let configuration = Configuration {
        timeout: Some(core::time::Duration::from_secs(130)),
        ..Default::default()
    };

    let connection = EspHttpConnection::new(&configuration)?;
    // 2. Get a client using the embedded_svc Client::wrap method. (Check documentation)
    let mut client = Client::wrap(connection);

    let headers = [("Content-Type", "application/json")];

    let mut request = client.request(Method::Post, url.as_ref(), &headers)?;

    request.write(data)?;

    // 4. Submit the request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let mut response = request.submit()?;
    let status = response.status();
    match status {
        200..=299 => {
            let mut buf = [0_u8; 4];
            let mut output = Vec::new();

            loop {
                match response.read(&mut buf)? {
                    0 => break,
                    b => {
                        output.extend_from_slice(&buf[..b]);
                    }
                }
            }

            Ok(output)
        }
        _ => {
            let mut buf = [0_u8; 256];
            response.read(buf.as_mut())?;
            let resp_string =
                core::str::from_utf8(&buf).unwrap_or("invalid utf8 when parsing error");

            log::error!("{}\n", resp_string);

            Err(EspBotError::Http(HttpError {
                _code: status,
                _message: format!("response code: {}", status),
            }))
        }
    }
}

pub fn telegram_post_multipart(
    url: impl AsRef<str>,
    data: &[u8],
    chat_id: i64,
) -> Result<Vec<u8>, EspBotError> {
    // 1. Create a new EspHttpConnection with default Configuration. (Check documentation)
    let config = Configuration::default();

    let connection = EspHttpConnection::new(&config)?;
    // 2. Get a client using the embedded_svc Client::wrap method. (Check documentation)
    let mut client = Client::wrap(connection);

    let chat_id = chat_id.to_string();

    let boundary = "esp32esp32esp32";

    let head = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"chat_id\"; \r\n\r\n{chat_id}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"esp32-cam.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n");
    let tail = format!("\r\n--{boundary}--\r\n");

    let datalen = head.len() + data.len() + tail.len();

    // 3. Open a GET request to `url`
    let headers = [
        (
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}") as &str,
        ),
        ("Content-Length", &datalen.to_string()),
    ];
    // ANCHOR: request
    let mut request = client.post(url.as_ref(), &headers)?;
    // ANCHOR_END: request

    request.write(head.as_bytes())?;
    request.write(data)?;
    request.write(tail.as_bytes())?;

    request.flush()?;

    //esp!(unsafe { esp_http_client_set_post_field(raw_handle, data.as_ptr() as *const i8, data.len() as i32) })?;

    // 4. Submit the request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let mut response = request.submit()?;
    let status = response.status();
    println!("Response code: {}\n", status);
    match status {
        200..=299 => {
            let mut buf = Vec::new();
            let mut output = Vec::new();

            loop {
                match response.read(&mut buf)? {
                    0 => break,
                    b => {
                        output.extend_from_slice(&buf[..b]);
                        buf.clear();
                    }
                }
            }

            Ok(output)
        }
        _ => {
            let mut buf = [0_u8; 256];
            response.read(buf.as_mut())?;
            let resp_string =
                core::str::from_utf8(&buf).unwrap_or("invalid utf8 when parsing error");

            log::error!("{}\n", resp_string);

            Err(EspBotError::Http(HttpError {
                _code: status,
                _message: format!("response code: {}", status),
            }))
        }
    }
}

use frankenstein::ErrorResponse;
use frankenstein::TelegramApi;
use std::path::PathBuf;
use thiserror::Error;

pub struct Esp32Api {
    pub api_url: String,
}

#[derive(Error, Debug)]
pub enum EspBotError {
    #[error("HTTP error")]
    Http(HttpError),
    #[error("API error")]
    Api(ErrorResponse),
    #[error("ESP error")]
    Esp(#[from] esp_idf_sys::EspError),
    #[error("IO error")]
    Io(#[from] EspIOError),
    #[error("utf8 error")]
    Json(#[from] core::str::Utf8Error),
    #[error("serde error")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct HttpError {
    pub _code: u16,
    pub _message: String,
}

static BASE_API_URL: &str = "https://api.telegram.org/bot";

impl Esp32Api {
    #[must_use]
    pub fn new(api_key: &str) -> Self {
        let api_url = format!("{BASE_API_URL}{api_key}");
        Self { api_url }
    }
}

impl From<std::io::Error> for EspBotError {
    fn from(error: std::io::Error) -> Self {
        let message = format!("{error:?}");
        let error = HttpError {
            _code: 500,
            _message: message,
        };
        Self::Http(error)
    }
}

impl TelegramApi for Esp32Api {
    type Error = EspBotError;

    fn request<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, EspBotError> {
        let url = format!("{}/{method}", self.api_url);

        let response = match params {
            None => http_post(url, &[])?,
            Some(data) => {
                let json = serde_json::to_string(&data)?;
                http_post(url, json.as_bytes())?
            }
        };

        let text = core::str::from_utf8(&response)?;

        let parsed_result: Result<T2, serde_json::Error> = serde_json::from_str(text);

        parsed_result.map_err(|_| {
            let parsed_error: Result<ErrorResponse, serde_json::Error> = serde_json::from_str(text);

            match parsed_error {
                Ok(result) => EspBotError::Api(result),
                Err(error) => {
                    let message = format!("{error:?}");
                    let error = HttpError {
                        _code: 500,
                        _message: message,
                    };
                    EspBotError::Http(error)
                }
            }
        })
    }

    // isahc doesn't support multipart uploads
    // https://github.com/sagebind/isahc/issues/14
    fn request_with_form_data<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        _method: &str,
        _params: T1,
        _files: Vec<(&str, PathBuf)>,
    ) -> Result<T2, EspBotError> {
        let error = HttpError {
            _code: 500,
            _message: "isahc doesn't support form data requests".to_string(),
        };

        Err(EspBotError::Http(error))
    }
}
