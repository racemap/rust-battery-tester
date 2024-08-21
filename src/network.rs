use embassy_executor::Spawner;
use embedded_svc::utils::asyncify::Asyncify;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::wifi::WifiEvent;
use esp_idf_svc::{eventloop::EspSystemEventLoop, wifi::EspWifi};
use log::*;

const SSID: &str = dotenv!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = dotenv!("RUST_ESP32_STD_DEMO_WIFI_PASS");

#[embassy_executor::task]
pub async fn init_network(
    sysloop: EspSystemEventLoop,
    spawner: Spawner,
    mut wifi: EspWifi<'static>,
) {
    info!("Wifi created, about to scan");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASS.into(),
        channel: Some(6),
        auth_method: AuthMethod::None,
        ..Default::default()
    }))
    .expect("Failed to init wifi!");

    wifi.start().ok();

    wifi.connect().ok();

    spawner.spawn(wifi_event_handler(wifi, sysloop)).ok();
}

#[embassy_executor::task]
pub async fn wifi_event_handler(mut wifi: EspWifi<'static>, mut sysloop: EspSystemEventLoop) {
    info!("Start the wifi event handler!");
    let mut sub = sysloop
        .as_async()
        .subscribe::<WifiEvent>()
        .expect("Failed to subscribe!");

    loop {
        let event = sub.recv().await;
        info!("Receive wifi event: {:?}", event);

        match event {
            WifiEvent::StaConnected => {
                print!("hallo");
            }
            WifiEvent::StaDisconnected => {
                {
                    print!("hallo");
                }
                let _ = wifi.connect();
            }
            _ => {}
        }
    }
}
