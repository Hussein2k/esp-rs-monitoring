use embedded_svc::wifi::AccessPointConfiguration;
use esp_idf_hal::{delay, peripherals::Peripherals};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys;
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
    wifi.set_configuration(&embedded_svc::wifi::Configuration::AccessPoint(
        AccessPointConfiguration {
            ..Default::default()
        },
    ))
    .unwrap();
    wifi.start().unwrap();

    let mut vals = [0u8; 6];

    loop {
        unsafe {
            println!(
                "esp_wifi_get_mac res: {:?}",
                esp_idf_sys::esp_wifi_get_mac(0, &mut vals as _)
            );
            println!("{:x?}", vals);
        }
        delay::Delay::delay_ms(1000);
    }
}
