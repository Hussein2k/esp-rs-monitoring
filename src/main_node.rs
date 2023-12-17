use std::sync::{Arc, Mutex};

use embedded_svc::wifi::ClientConfiguration;
use esp_idf_hal::{delay, peripherals::Peripherals};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys::esp_now_peer_info;

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

    let a = SensorPacket::default();

    println!("{:?}", serde_json::to_string(&a));
    loop {
        delay::Delay::delay_ms(100);
    }
}

fn main2() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let periph = Peripherals::take().unwrap();
    let event_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let mut wifi = esp_idf_svc::wifi::WifiDriver::new(periph.modem, event_loop, Some(nvs))
        .expect("Couldn't initiate wifi!\n");
    wifi.set_configuration(&embedded_svc::wifi::Configuration::Client(
        ClientConfiguration {
            ..Default::default()
        },
    ))
    .unwrap();
    wifi.start().unwrap();
    wifi.disconnect();

    let espnow = esp_idf_svc::espnow::EspNow::take().unwrap();
    let sensor_values = Arc::new(Mutex::new([SensorPacket::default(); 4]));
    let callback_arc = Arc::clone(&sensor_values);

    espnow
        .register_recv_cb(move |a, b| {
            let tmp: SensorPacket = bincode::deserialize(&b).unwrap_or_default();
            let mut sensors_data = callback_arc.lock().unwrap();
            sensors_data[tmp.sensor_id as usize] = tmp;

        })
        .expect("Couldn't register recv cb");
    
    espnow
        .register_send_cb(|a, b| println!("send: {:?}\t{:?}", a, b))
        .expect("Couldn't register send cb");

    loop {
        let cached_values = sensor_values.lock().unwrap();
        println!("==========================");
        for i in cached_values.iter() {
            match i.sensor_data {
                SensorValue::HumidityAndTemperature(h, t) => print!("H:{},T:{}\n", h, t),
                _ => {}
            }
        }

        delay::Delay::delay_ms(1000);
    }
}
