use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use esp_idf_sys::{esp_partition_get_sha256, esp_partition_t};
use esp_ota::*;
use log::info;
use std::ptr;

const WRITE_DATA_BUF_SIZE: usize = 4096;
const TX_BUF_SIZE: usize = 4096;

const UPDATE_URL: &str = dotenv!("OTA_UPDATE_URL");

/// ota function
///
/// pull the given firmaware via tag to the esp and sets it as bootpartition
///
/// * `firmware` - the firmware struct pulled via Tag so tag mast be set
pub fn ota_wifi(firmware_meta: &str) {
    info!("Start OTA update!");
    let mut ota_write_data: [u8; WRITE_DATA_BUF_SIZE] = [0; WRITE_DATA_BUF_SIZE];

    let httpconnection = EspHttpConnection::new(&HttpConfig {
        buffer_size: Some(WRITE_DATA_BUF_SIZE),
        buffer_size_tx: Some(TX_BUF_SIZE),
        ..Default::default()
    })
    .expect("Failed to create http connection");
    let mut url: String = UPDATE_URL.to_string();
    url.push_str(firmware_meta);

    // Prepare request
    let mut httpclient = Client::wrap(httpconnection);

    let request = httpclient.get(&url).expect("Failed to get url");
    let mut response = request.submit().expect("Failed to submit request");
    if response.status() != 200 {
        info!("Failed to get url: {:x}", response.status());
        ()
    }
    let data_length = response.header("content-length").unwrap();
    info!("data length {}", data_length);
    let mut ota = OtaUpdate::begin().expect("Failed to start OTA update!");
    let mut total_size: u32 = 0;
    loop {
        // TODO: check the headers content currently just checking for size
        let read_bytes = response
            .read(&mut ota_write_data)
            .expect("Failed to read from response");
        if read_bytes > 0 {
            ota.write(&ota_write_data)
                .expect("Failed to write to OTA update!");
        }
        total_size += read_bytes as u32;
        if read_bytes < ota_write_data.len() {
            break;
        }
    }
    let mut completed_ota = ota.finalize().expect("Failed to finalize OTA update!");
    completed_ota
        .set_as_boot_partition()
        .expect("failed top set as boot partition");
    info!("guess that should have worked");
    completed_ota.restart();
}

pub fn check_hash(partition: *const esp_partition_t, hash: String) -> bool {
    let mut sha256_str = String::new();

    let mut sha256: [u8; 32] = [0; 32];

    for i in 0..32 {
        unsafe {
            esp_partition_get_sha256(partition, &mut sha256[i]);
        }
        sha256_str.push_str(&format!("{:02x}", sha256[i]));
    }

    info!("sha256: {}", sha256_str);
    if sha256_str == hash {
        return true;
    }
    false
}
