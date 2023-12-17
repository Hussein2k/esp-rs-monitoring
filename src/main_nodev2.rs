use std::sync::{Arc, Mutex};

use embedded_svc::wifi::{ClientConfiguration, AccessPointConfiguration};
use esp_idf_hal::{delay, peripherals::Peripherals};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use modules::{monitoring_module::EspMonitorConfiguration, automation_module::{self, RelayConfiguration}};


#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
enum SensorValue {
    Voltage(f32),
    Humidity(f32),
    Temperature(f32),
    HumidityAndTemperature(f32, f32),
}
impl Default for SensorValue {
    fn default() -> Self {
        Self::HumidityAndTemperature(0.0, 0.0)
    }
}
#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
struct SensorPacket {
    sensor_id: u8,
    sensor_data: SensorValue,
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let periph = Peripherals::take().unwrap();
    let event_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let mut wifi = esp_idf_svc::wifi::EspWifi::new(periph.modem, event_loop, Some(nvs))
        .expect("Couldn't initiate wifi!\n");
   
    println!("{:?}",serde_json::from_str::<RelayConfiguration>(&"{\"sensor_id\":1,\"relay_id\":1,\"mode\":0,\"min_value\":0,\"max_value\":1,\"auto_dependence\":1}"));

    wifi.set_configuration(&embedded_svc::wifi::Configuration::Mixed(
        ClientConfiguration::default(),
        AccessPointConfiguration {
            ssid: "Monitoring System".into(),
            password: "1234012340".into(),
            auth_method: embedded_svc::wifi::AuthMethod::WPA2Personal,
            ..Default::default()
        },
    ))
    .unwrap();

    wifi.start().unwrap();

    let mut espnow_module =
        modules::monitoring_module::EspNowMonitor::new(&EspMonitorConfiguration {
            max_nodes: 10,
            node_timeout: 10,
        })
        .unwrap();

    espnow_module.start_pooling();
    espnow_module.start_monitoring();

    let mut relay_controller = automation_module::RelayStateController::new(

       &[0u8, 13, 12, 14, 26, 25, 27, 0u8],
        espnow_module.get_sensors_instance(),
    )
    .unwrap();
    relay_controller.start_polling(3.0, 1.0);

    // Start HTTP Server
    let index =
        include_str!("/home/hussein/Dev/Espressif/monitoring-system/website/field/index.html");
    let stylesheet =
        include_str!("/home/hussein/Dev/Espressif/monitoring-system/website/field/style.css");
    let script =
        include_str!("/home/hussein/Dev/Espressif/monitoring-system/website/field/script.js");
    let relay_image =
        include_bytes!("/home/hussein/Dev/Espressif/monitoring-system/website/field/relay.png");
    let sensor_image =
        include_bytes!("/home/hussein/Dev/Espressif/monitoring-system/website/field/sensor.png");

    let mut http_server = modules::http_backend::HTTPMonitoringServer::new(
        &index,
        &stylesheet,
        &script,
        vec![("/relay.png", relay_image), ("/sensor.png", sensor_image)],
        espnow_module.get_sensors_instance(),
        relay_controller.get_relays_criteria_instance(),
        /* relay_controller.get_current_state_instance(), */
    );

    loop {

        delay::Delay::delay_ms(100);
    }
}
