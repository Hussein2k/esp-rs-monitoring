use bincode;
use embedded_svc::wifi::ClientConfiguration;
use esp_idf_hal::{delay, peripherals::Peripherals};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys::esp_now_peer_info;

use modules::monitoring_module::*;
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
    unsafe {
        esp_idf_sys::esp_wifi_set_ps(esp_idf_sys::wifi_ps_type_t_WIFI_PS_MIN_MODEM);
    }
    const ADDR: [u8; 6] = [0xc8, 0xf0, 0x9e, 0x4e, 0xc6, 0x94]; //[0xc8, 0xf0, 0x9e, 0x4c, 0xdd, 0xa0];
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
    let mut sensor_packet = SensorPacket {
        sensor_id: 2,
        sensor_data: SensorValue::humidity_temperature(0.0, 0.0),
        ..Default::default()
    };

    unsafe {
        esp_idf_sys::shtc3_init(25, 26, 100000);
    }

    loop {
        unsafe {
            let mut t = 0.0f32;
            let mut h = 0.0f32;
            esp_idf_sys::shtc3_wake_up(0x70);
            esp_idf_sys::shtc3_read(0x70, esp_idf_sys::T_FIRST_N as _, &mut t, &mut h);
            esp_idf_sys::shtc3_sleep(0x70);
            sensor_packet.sensor_data = SensorValue::humidity_temperature(h, t);
            sensor_packet.timestamp = esp_idf_sys::esp_timer_get_time() as u64;
        }

        espnow
            .send(ADDR, bincode::serialize(&sensor_packet).unwrap().as_slice())
            .expect("Error sending Hello");
        delay::Delay::delay_ms(1000);
    }
}
