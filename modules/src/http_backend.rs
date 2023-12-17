use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use super::monitoring_module::*;
use embedded_svc::{io::Write, wifi::ClientConfiguration};
use esp_idf_hal::{
    delay::{self, FreeRtos},
    peripherals::Peripherals,
};
use esp_idf_svc::{
    errors::EspIOError,
    eventloop::EspSystemEventLoop,
    http::server::{EspHttpConnection, EspHttpServer},
    nvs::EspDefaultNvsPartition,
};
use esp_idf_sys::{esp_now_peer_info, esp_timer_get_time, EspError};
use serde::{Deserialize, Serialize};

use super::automation_module::*;
use super::monitoring_module::SensorArc;

///////////////////////////////////////////////////////////// HTTP MODULE //////////////////////////////////////////////////////////////////////////
///
///
///
///
const RELAY_MODE_AUTO: u8 = 0;
const RELAY_MODE_ALWAYS_ON: u8 = 1;
const RELAY_MODE_ALWAYS_OFF: u8 = 2;
#[derive(Default, Clone, Copy, Serialize, Deserialize)]
struct RelayStateScreen(u8, u8, bool);
pub struct HTTPMonitoringServer {
    pub m_http_server: EspHttpServer,
    m_sensors_instance: SensorArc,
    m_relay_crit_instance: RelayArc,
    m_relays_states: Arc<Mutex<[RelayStateScreen; 6]>>, //relay_id, MODE (ON,OFF,AUTO),STATE
}

impl HTTPMonitoringServer {
    // fn set_callbacks(http_server: &mut EspHttpServer) {
    //     http_server
    //         .fn_handler("/", embedded_svc::http::Method::Get, |res| Ok(()))
    //         .expect("Couldn't add function handler");
    // }

    fn setCallbacks(
        http_server: &mut EspHttpServer,
        sensors: SensorArc,
        relays: RelayArc,
        stats: Arc<Mutex<[RelayStateScreen; 6]>>,
    ) {
        //tarheem
        http_server
            .fn_handler(
                "/get_sensors",
                embedded_svc::http::Method::Get,
                move |res| {
                    let mut filtered: Vec<SensorPacket> = Vec::new();
                    {
                        let current_data = sensors.lock().unwrap();
                        filtered = current_data
                            .iter()
                            .filter_map(|elem| {
                                if elem.0.sensor_id != 0 {
                                    Some(elem.0)
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                    let json_data = serde_json::to_string(&filtered);
                    res.into_ok_response()
                        .unwrap()
                        .write_all(json_data.unwrap().as_bytes())
                        .expect("Error sending json data!");
                    Ok(())
                },
            )
            .expect("Error handling /get_sensors");
        let mitis = stats.clone();
        http_server
            .fn_handler(
                "/set_relays",
                embedded_svc::http::Method::Post,
                move |mut res| {
                    let mut buf = [0u8; 1024];
                    if let Ok(sz) = res.read(&mut buf) {
                        // println!("{:?}", std::str::from_utf8(&buf));
                        let ptr: &[u8] = unsafe { std::slice::from_raw_parts(buf.as_ptr(), sz) };
                        let RelayConf: RelayConfiguration =
                            serde_json::from_str(std::str::from_utf8(ptr).unwrap_or_default())
                                .unwrap_or_default();
                        println!("{:?}", RelayConf);
                        let mut relays_crit = relays.lock().unwrap();
                        let tmp_conf = RelayConfiguration::default();

                        //Stubd err
                        let measurement_type = match (RelayConf.auto_dependence, RelayConf.mode) {
                            (_, RELAY_MODE_ALWAYS_ON) => MEASUREMENT_TYPE_NONE_ALWAYS_ON,
                            (_, RELAY_MODE_ALWAYS_OFF) => MEASUREMENT_TYPE_NONE_ALWAYS_OFF,
                            (sensing_type, _) => sensing_type,
                        };

                        // println!("set measurement type: {:?}=>{}", RelayConf, measurement_type);
                        relays_crit[RelayConf.relay_id as usize] = RelayStateDependence {
                            measurement_type: measurement_type,
                            sensor_id: RelayConf.sensor_id,
                            range: (RelayConf.min_value, RelayConf.max_value),
                        };
                    }

                    Ok(())
                },
            )
            .expect("Error handling /set_relays");

        http_server
            .fn_handler(
                "/get_status",
                embedded_svc::http::Method::Get,
                move |mut res| {
                    let mut my_stats = vec![RelayStateScreen::default(); 6];
                    {
                        //LOCK
                        let tmp = stats.lock().unwrap();
                        for i in 0..tmp.len() {
                            my_stats[i] = tmp[i];
                        }
                    } //UNLOCK
                    res.into_ok_response()
                        .unwrap()
                        .write_all(serde_json::to_string(&my_stats).unwrap().as_bytes());
                    Ok(())
                },
            )
            .expect("Error handling /set_relays");
    }

    fn setRelayCallbacks(http_server: &mut EspHttpServer) {
        http_server
            .fn_handler("/set_relays", embedded_svc::http::Method::Get, move |res| {
                println!("{:?}", res.uri());

                Ok(())
            })
            .expect("Coudn't handle /");
    }
    pub fn new(
        index: &'static str,
        stylesheet: &'static str,
        script: &'static str,
        // relay_image: &'static [u8],
        binary_files: Vec<(&str, &'static [u8])>,
        sensors_instance: SensorArc,
        relay_instance: RelayArc,
    ) -> Result<Self, EspIOError> {
        let mut http_server = esp_idf_svc::http::server::EspHttpServer::new(
            &esp_idf_svc::http::server::Configuration {
                ..Default::default()
            },
        );

        if let Ok(mut httpserver) = http_server {
            httpserver
                .fn_handler("/", embedded_svc::http::Method::Get, move |res| {
                    res.into_ok_response()
                        .unwrap()
                        .write_all(index.as_bytes())
                        .expect("error responding to /");
                    Ok(())
                })
                .expect("Coudn't handle /");
            httpserver
                .fn_handler(
                    "/style.css",
                    embedded_svc::http::Method::Get,
                    move |mut res| {
                        unsafe {
                            // let ptr = &mut res as *mut _ as *mut EspHttpConnection;
                            // let mut tmp1 = *ptr;

                            //     as *mut *mut esp_idf_sys::httpd_req_t;
                            // let reqt = (ptr) as *const esp_idf_sys::httpd_req_t
                            //     as *mut esp_idf_sys::httpd_req_t;

                            println!("{:?}", res.0.get_t());
                            println!(
                                "{:?}",
                                esp_idf_sys::httpd_resp_set_type(
                                    res.0.get_t(),
                                    "text/css\0".as_ptr() as _
                                )
                            );
                        }

                        res.into_ok_response()
                            .unwrap()
                            .write_all(stylesheet.as_bytes())
                            /* .expect("error responding to /") */;
                        Ok(())
                    },
                )
                .expect("Coudn't handle /");
            httpserver
                .fn_handler("/script.js", embedded_svc::http::Method::Get, move |res| {
                    res.into_ok_response()
                        .unwrap()
                        .write_all(script.as_bytes())
                       /*  .expect("error responding to /") */;
                    Ok(())
                })
                .expect("Coudn't handle /");

            ////////////////////???RELAY IMAGE
            // httpserver
            //     .fn_handler(
            //         "/relay.png",
            //         embedded_svc::http::Method::Get,
            //         move |mut res| {
            //             unsafe {
            //                 // let ptr = &mut res as *mut _ as *mut EspHttpConnection;
            //                 // let mut tmp1 = *ptr;
            //                 //     as *mut *mut esp_idf_sys::httpd_req_t;
            //                 // let reqt = (ptr) as *const esp_idf_sys::httpd_req_t
            //                 //     as *mut esp_idf_sys::httpd_req_t;
            //                 // println!("{:?}", res.0.get_t());
            //                 // println!(
            //                 //     "{:?}",
            //                 //     esp_idf_sys::httpd_resp_set_type(
            //                 //         res.0.get_t(),
            //                 //         "text/css\0".as_ptr() as _
            //                 //     )
            //                 // );
            //             }
            //             res.into_ok_response()
            //                 .unwrap()
            //                 .write_all(relay_image)
            //                 .expect("error responding to /");
            //             Ok(())
            //         },
            //     )
            //     .expect("Coudn't handle /");

            for bin_file in binary_files {
                httpserver
                    .fn_handler(
                        bin_file.0,
                        embedded_svc::http::Method::Get,
                        move |mut res| {
                            res.into_ok_response()
                                .unwrap()
                                .write_all(bin_file.1)
                                .expect("error responding to /");
                            Ok(())
                        },
                    )
                    .expect("Coudn't handle /");
            }

            let status = Arc::new(Mutex::new([
                RelayStateScreen(1, RELAY_MODE_ALWAYS_OFF, false),
                RelayStateScreen(2, RELAY_MODE_ALWAYS_OFF, false),
                RelayStateScreen(3, RELAY_MODE_ALWAYS_OFF, false),
                RelayStateScreen(4, RELAY_MODE_ALWAYS_OFF, false),
                RelayStateScreen(5, RELAY_MODE_ALWAYS_OFF, false),
                RelayStateScreen(6, RELAY_MODE_ALWAYS_OFF, false),
            ]));
            //SET ADDITIOANL CALLBACKS
            Self::setCallbacks(
                &mut httpserver,
                Arc::clone(&sensors_instance),
                Arc::clone(&relay_instance),
                Arc::clone(&status),
            );
            return Ok(Self {
                m_http_server: httpserver,
                m_sensors_instance: Arc::clone(&sensors_instance),
                m_relay_crit_instance: relay_instance.clone(),
                m_relays_states: status.clone(),
            });
        }
        return Err(http_server.err().unwrap());
    }
}
