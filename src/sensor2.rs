use bincode;
use embedded_svc::wifi::ClientConfiguration;
use esp_idf_hal::{
    adc::{AdcChannelDriver, Atten0dB, Atten11dB, Atten2p5dB, ADC1, ADC2},
    delay,
    gpio::Gpio34,
    peripherals::Peripherals,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys::{esp_now_peer_info, esp_timer_get_time};

use modules::{
    hals::*,
    monitoring_module::{SensorPacket, SensorValue},
};

// #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
// enum SensorValue {
//     Voltage(f32),
//     Humidity(f32),
//     Temperature(f32),
//     HumidityAndTemperature(f32, f32),
// }
// #[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
// struct SensorPacket {
//     sensor_id: u8,
//     sensor_data: SensorValue,
// }

// struct SensorPacket{
//     sensor_id:u32,
// }

fn main2() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut adc = ADCWrapper::new(
        1,
        esp_idf_sys::adc_bits_width_t_ADC_WIDTH_BIT_12 as _,
        esp_idf_sys::adc_channel_t_ADC_CHANNEL_4 as _,
        esp_idf_sys::adc_atten_t_ADC_ATTEN_DB_11 as _,
    );

    //}
    loop {
        println!("{:?}", adc.get_value());
        delay::Delay::delay_ms(500);
    }
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
    let mut wifi = esp_idf_svc::wifi::WifiDriver::new(periph.modem, event_loop, Some(nvs))
        .expect("Couldn't initiate wifi!\n");
    {
        wifi.set_configuration(&embedded_svc::wifi::Configuration::Client(
            ClientConfiguration {
                ..Default::default()
            },
        ))
        .unwrap();
        wifi.start().unwrap();
        wifi.disconnect();
    }
    const ADDR: [u8; 6] = [0xc8, 0xf0, 0x9e, 0x4e, 0xc6, 0x94];
    let espnow = esp_idf_svc::espnow::EspNow::take().unwrap();
    {
        espnow
            .register_recv_cb(|a, b| println!("recv: {:?}\t{:?}", a, b))
            .expect("Couldn't register recv cb");
        espnow
            .register_send_cb(|a, b| println!("send: {:?}\t{:?}", a, b))
            .expect("Couldn't register send cb");

        //root address

        espnow
            .add_peer(esp_now_peer_info {
                peer_addr: ADDR,
                channel: 0,
                encrypt: false,
                ..Default::default()
            })
            .expect("Couldn't add peer");
    }

    //initialize shtc3
    // let mut sensor_packet = SensorPacket {
    //     sensor_id: 3,
    //     sensor_data: SensorValue::voltage(0.0),
    //     timestamp: unsafe { esp_timer_get_time()  as _},
    // };

    let mut sensor_packet = SensorPacket {
        sensor_id: 3,
        sensor_data: SensorValue::humidity_temperature(0.0, 0.0),
        ..Default::default()
    };
    unsafe {
        esp_idf_sys::shtc3_init(25, 26, 100000);
    }

    let mut adc = ADCWrapper::new(
        1,
        esp_idf_sys::adc_bits_width_t_ADC_WIDTH_BIT_12 as _,
        esp_idf_sys::adc_channel_t_ADC_CHANNEL_4 as _,
        esp_idf_sys::adc_atten_t_ADC_ATTEN_DB_11 as _,
    );

    loop {
        unsafe {
            let mut t = 0.0f32;
            let mut h = 0.0f32;
            esp_idf_sys::shtc3_wake_up(0x70);
            esp_idf_sys::shtc3_read(0x70, esp_idf_sys::T_FIRST_N as _, &mut t, &mut h);
            esp_idf_sys::shtc3_sleep(0x70);
            sensor_packet.sensor_data = SensorValue::voltage(adc.get_value() as f32 * 3.9 / 4095.0);
        }

        espnow
            .send(ADDR, bincode::serialize(&sensor_packet).unwrap().as_slice())
            .expect("Error sending Hello");
        delay::Delay::delay_ms(1000);
    }
}
