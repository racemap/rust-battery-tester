use embassy_executor::Spawner;
use embedded_svc::http::Method;
use embedded_svc::utils::asyncify::Asyncify;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::http::server::{Configuration as HttpServerConfig, EspHttpServer};
use esp_idf_svc::wifi::WifiEvent;

use embassy_time::{Duration, Timer};
use esp_idf_svc::{eventloop::EspSystemEventLoop, wifi::EspWifi};
use log::*;

use crate::METHOD_SIG;
use crate::STORAGE;

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
    println!("IP info: {:?}", wifi.sta_netif().get_ip_info().unwrap());
    spawner.spawn(wifi_event_handler(wifi, sysloop)).ok();
    spawner.spawn(webserver()).ok();
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

#[embassy_executor::task]
pub async fn webserver() {
    let mut httpserver = match EspHttpServer::new(&HttpServerConfig::default()) {
        Ok(ser) => ser,
        Err(e) => {
            warn!("unable to start webserver {}", e);
            return;
        }
    };

    httpserver.fn_handler("/delete", Method::Post, |request| {
        METHOD_SIG.signal(1);
        let html = "Reseted!".to_string();
        let mut response = request.into_ok_response()?;
        // Return Requested Object (Index Page)
        response.write(html.as_bytes())?;
        Ok(())
    });

    loop {
        let content = index_html().await;
        httpserver.fn_handler("/", Method::Get, move |request| {
            // Retrieve html String
            let html = &content;
            // Respond with OK status
            let mut response = request.into_ok_response()?;
            // Return Requested Object (Index Page)
            response.write(html.as_bytes())?;
            Ok(())
        });
        Timer::after(Duration::from_secs(1000)).await;
    }
}

async fn index_html() -> String {
    let mutex = STORAGE.lock().await;
    let s = match mutex.as_ref() {
        Some(s) => s,
        _ => return "".to_string(),
    };
    format!(
        r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Title</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script src="https://unpkg.com/htmx.org@2.0.2"></script>
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous"></script>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH" crossorigin="anonymous">


</head>
<body>
<header data-bs-theme="dark">
  <div class="navbar navbar-dark bg-dark shadow-sm">
    <div class="container">
      <a class="navbar-brand d-flex align-items-center">
        <strong>Battery Health check</strong>
      </a>
    </div>
  </div>
</header>
<main>
<section class="py-5 text-center container">
<button hx-post="/delete" class="btn btn-danger">Rest!</button>
<div style="height: 50vh; width: 50%;">
        <canvas id="myChart{{ time }}"></canvas>
    <div>
        Amps
    </div>
    </div>
        <script>
        const labels = {:?};

        const data = {{ labels: labels,
            datasets: [{{
                label: 'Volts',
                backgroundColor: 'rgb(255, 99, 132)',
                borderColor: 'rgb(255, 99, 132)',
                data: {:?},
        }}]
        }};

        const config = {{
            type: 'line',
            data: data,
            options: {{ maintainAspectRatio: false }}
        }};

        const myChart = new Chart(
            document.getElementById('myChart{{ time }}'),
            config
        );

    </script>
    
</section>
</main>
</body>
</html>
"#,
        s.get_labels(),
        s.get_values()
    )
}
