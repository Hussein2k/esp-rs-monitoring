use std::sync::{Arc, Mutex};

use embedded_svc::wifi::{AccessPointConfiguration, ClientConfiguration};
use esp_idf_hal::{
    delay::{self, FreeRtos},
    peripherals::Peripherals,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    httpd::{Configuration, Server},
    nvs::EspDefaultNvsPartition,
};
use esp_idf_sys::esp_now_peer_info;
use modules::monitoring_module::EspMonitorConfiguration;

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

    let mut ssd1306_ctx = esp_idf_sys::SSD1306_t {
        ..Default::default()
    };

    // let i2c_driver = esp_idf_hal::i2c::I2cDriver::new(
    //     periph.i2c0,
    //     periph.pins.gpio5,
    //     periph.pins.gpio4,
    //     &esp_idf_hal::i2c::config::Config {
    //         baudrate: 400000.into(),
    //         scl_pullup_enabled: true,
    //         sda_pullup_enabled: true,
    //         timeout: None,
    //         ..Default::default()
    //     },
    // );
    unsafe {
        esp_idf_sys::i2c_master_init(&mut ssd1306_ctx, 5, 4, 0);
        esp_idf_sys::ssd1306_init(&mut ssd1306_ctx, 128, 64);
        esp_idf_sys::ssd1306_clear_screen(&mut ssd1306_ctx, true);
        esp_idf_sys::ssd1306_contrast(&mut ssd1306_ctx, 0xff);
        let mut i = 0.1;
        let mut tmp = String::new();
        loop {
            let temp = format!("temp: {:2.2}", (i) as f32);
            let humi = format!("humi: {:2.2}", (10.0 * i + 0.1 as f32));
            esp_idf_sys::ssd1306_display_text(
                &mut ssd1306_ctx,
                1,
                temp.as_ptr() as _,
                temp.len() as _,
                false,
            );
            esp_idf_sys::ssd1306_display_text(
                &mut ssd1306_ctx,
                2,
                humi.as_ptr() as _,
                humi.len() as _,
                false,
            );
            i += 1.0;

            //esp_idf_sys::_ssd1306_pixel(&mut ssd1306_ctx, 10, 10, true);
            FreeRtos::delay_ms(100);
            // esp_idf_sys::ssd1306_clear_line(&mut ssd1306_ctx, 6, true);
        }
    }
}
