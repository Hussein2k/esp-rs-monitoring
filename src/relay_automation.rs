use std::sync::{Arc, Mutex};

use embedded_svc::wifi::{AccessPointConfiguration, ClientConfiguration};
use esp_idf_hal::{delay, gpio::PinDriver, peripheral::Peripheral, peripherals::Peripherals};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    httpd::{Configuration, Server},
    nvs::EspDefaultNvsPartition,
};
use esp_idf_sys::{esp_now_peer_info, esp_timer_get_time};
use modules::{
    automation_module,
    monitoring_module::{SensorArc, SensorPacket},
};
use modules::{
    automation_module::{RelayStateDependence, MEASUREMENT_TYPE_HUMIDITY},
    monitoring_module::{EspMonitorConfiguration, SensorValue},
};

//Testbench
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let mut sensors_ = vec![(SensorPacket::default(), 0u64); 9];
    for i in 0..sensors_.len() {
        sensors_[i].0.sensor_id = (i + 1) as _;
        sensors_[i].0.sensor_data = SensorValue::humidity(10.0 * i as f32 + 1.0);
    }
    let sensors = Arc::new(Mutex::new(sensors_));
    let mut auto_mod = automation_module::RelayStateController::new(
        &[20, 21, 22, 23, 24, 25, 26, 27u8],
        sensors.clone(),
    )
    .unwrap();

    for i in 0..8 {
        auto_mod.update_criteria(
            i as _,
            &RelayStateDependence {
                sensor_id: 1u8,
                measurement_type: MEASUREMENT_TYPE_HUMIDITY,
                range: (30.0, 50.0),
            },
        );
    }
    auto_mod.start_polling(10.0, 10.0);
    let status_mutex = auto_mod.get_status_instance();
    {
        let mut lock = auto_mod.get_relays_criteria_instance();
        let mut kk = lock.lock().unwrap();
        for i in 0..kk.len() {
            kk[i].sensor_id = (i + 1) as _;
        }
    }
    sensors.lock().unwrap()[1].0 = SensorPacket {
        sensor_id: 1,
        sensor_data: SensorValue::humidity(30.2),
        timestamp: unsafe { esp_timer_get_time() as u64 },
    };
    println!("Values: {:?}", sensors.lock().unwrap());
    loop {
        let t = unsafe { esp_timer_get_time() as u64 };
        sensors.lock().unwrap()[1].0 = SensorPacket {
            sensor_id: 1,
            sensor_data: SensorValue::humidity(50.0 * (1.0 + f32::sin(t as f32 / 1e6))),
            timestamp: unsafe { t },
        };

        // // println!("AHA\n");
        // {
        //     for v in sensors.lock().unwrap().iter_mut() {
        //         v.1 = unsafe { esp_idf_sys::esp_timer_get_time() } as u64;
        //     }
        //     let instace = status_mutex.lock().unwrap();
        //     // println!("Current Values:\n{:?}", instace);
        // }
        // println!("{:?}", sensors.lock().unwrap());

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
    let mut wifi = esp_idf_svc::wifi::EspWifi::new(periph.modem, event_loop, Some(nvs))
        .expect("Couldn't initiate wifi!\n");

    wifi.set_configuration(&embedded_svc::wifi::Configuration::Mixed(
        ClientConfiguration::default(),
        AccessPointConfiguration {
            ssid: "Hussein".into(),
            password: "OpeOpenoMi".into(),
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

    // Start HTTP Server
    let index =
        include_str!("/home/hussein/Dev/Espressif/monitoring-system/website/field/index.html");
    let stylesheet =
        include_str!("/home/hussein/Dev/Espressif/monitoring-system/website/field/style.css");
    let script =
        include_str!("/home/hussein/Dev/Espressif/monitoring-system/website/field/script.js");

    // let mut http_server = modules::http_backend::HTTPMonitoringServer::new(
    //     &index,
    //     &stylesheet,
    //     &script,
    //     espnow_module.get_sensors_instance(),
    // );

    loop {
        delay::Delay::delay_ms(100);
    }
}
