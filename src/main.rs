use esp_idf_hal::{delay, peripherals::Peripherals};
use esp_idf_svc::{espnow::SendStatus, eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys::{self as _, esp_now_peer_info}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

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
    wifi.set_configuration(&embedded_svc::wifi::Configuration::None)
        .unwrap();
    wifi.start().unwrap();

    unsafe {
        // esp_idf_sys::esp_mesh
        // esp_idf_sys::esp!(esp_idf_sys::esp_wifi_start()).unwrap();
        // esp_idf_sys::esp!(esp_idf_sys::esp_now_init()).unwrap();
    }

    let other_node_mac = [0xc8u8, 0xf0, 0x9e, 0x50, 0x24, 0x4]; //[0xc8u8, 0xf0, 0x9e, 0x4c, 0xdd, 0xa0];
    let lmk = [0u8; 16];
    let mut espnow = esp_idf_svc::espnow::EspNow::take().unwrap();
    espnow
        .add_peer(esp_now_peer_info {
            peer_addr: other_node_mac,
            lmk: lmk,
            channel: 0,
            ifidx: 0,
            encrypt: false,
            priv_: 0 as _,
        })
        .expect("Couldn't add node!\n");

    espnow
        .register_send_cb(|a: &[u8], b: SendStatus| {
            println!("Send Callback: {:?}", (a, b));
        })
        .unwrap();

    espnow
        .register_recv_cb(|a, b| {
            println!("{:?}", (a, b));
        })
        .expect("msg");
    /////////////////////////////////////////

    unsafe {
        let mut vals: [u8; 6] = [0u8; 6];
        println!(
            "esp_wifi_get_mac res: {:?}",
            esp_idf_sys::esp_wifi_get_mac(0, &mut vals as _)
        );
        println!("vals: {:x?}", vals);
        // println!("esp_wifi_get_mac res: {:?}",esp_idf_sys::esp_netif_get_mac(0, &mut vals as _));
        // println!("vals: {:?}",vals);

        esp_idf_sys::shtc3_init(25, 26, 100000);
        let mut temp: f32 = 0.0;
        let mut hum: f32 = 0.0;
        loop {
            // println!(
            //     "Peer exists: {}",
            //     espnow.peer_exists(other_node_mac).unwrap()
            // );
            //espnow.send(other_node_mac, "Hello".as_bytes());

            esp_idf_sys::shtc3_wake_up(0x70);
            esp_idf_sys::shtc3_read(0x70, esp_idf_sys::T_FIRST_N as _, &mut temp, &mut hum);
            esp_idf_sys::shtc3_sleep(0x70);
            println!("Temp: {}C\tH: {}\n", temp, hum);
            delay::Delay::delay_ms(1000);
        }
    }

    info!("Hello, world!");
}
