use esp_idf_hal::{delay, peripherals::Peripherals};
use esp_idf_svc::http::server::Configuration;
use esp_idf_sys::esp_timer_get_time;
use modules;
use serde::{Deserialize, Serialize};
use serde_json;
static mut value: u32 = 0;

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct sensor_data {
    sensor_type: u32,
    humidity: f32,
    temprature: f32,
    voltage: f32,
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let sen = sensor_data::default();
    println!("{:?}", serde_json::ser::to_string(&sen));

    let periph = Peripherals::take().unwrap();
    let mut wifi =
        modules::wifi_module::WiFiModule::new_client(periph.modem, (&"Curl", &"@OpenGL64@"))
            .unwrap();

    while !wifi.mWiFi.is_connected().unwrap() {
        delay::Delay::delay_ms(1000);
        println!("Not Connected !");
    }

    let mut webserver_reg = esp_idf_svc::httpd::ServerRegistry::new();
    let mut webserver = esp_idf_svc::http::server::EspHttpServer::new(&Configuration {
        http_port: 80,
        https_port: 443,
        stack_size: 10240,
        ..Default::default()
    })
    .expect("Couldn't start webserver!");

    let index_page = include_str!("../webpage/index.html");
    let script = include_str!("../webpage/script.js");

    webserver
        .fn_handler("/get_value", embedded_svc::http::Method::Get, move |r| {
            r.into_ok_response()
                .unwrap()
                .write(unsafe { esp_timer_get_time().to_string().as_bytes() })
                .unwrap();
            Ok(())
        })
        .expect("error /get_value");

    webserver
        .fn_handler("/inc", embedded_svc::http::Method::Get, move |r| {
            r.into_ok_response().unwrap();
            unsafe { value = value + 1 };
            Ok(())
        })
        .expect("error /inc");

    webserver
        .fn_handler("/", embedded_svc::http::Method::Get, |r| {
            r.into_ok_response()
                .unwrap()
                .write(index_page.as_bytes())
                .expect("Couldn't respond to /");
            Ok(())
        })
        .expect("Couldn't add handler");
    webserver
        .fn_handler("/script.js", embedded_svc::http::Method::Get, |r| {
            r.into_ok_response()
                .unwrap()
                .write(script.as_bytes())
                .expect("Couldn't respond to /script.js");
            Ok(())
        })
        .expect("Couldn't add handler");
    loop {
        println!("Loop2");
        delay::Delay::delay_ms(1000);
    }
}
