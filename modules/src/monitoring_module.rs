use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use embedded_svc::wifi::ClientConfiguration;
use esp_idf_hal::{
    delay::{self, FreeRtos},
    peripherals::Peripherals,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys::{esp_now_peer_info, esp_timer_get_time, EspError};

pub type SensorArc = Arc<Mutex<Vec<(SensorPacket, u64)>>>;

const SENSOR_TYPE_HIMIDITY: u8 = 1;
const SENSOR_TYPE_TEMPERATURE: u8 = 2;
const SENSOR_TYPE_H_T: u8 = SENSOR_TYPE_HIMIDITY + SENSOR_TYPE_TEMPERATURE;
const SENSOR_TYPE_VOLTAGE: u8 = 4;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct SensorValue {
    //SENSOR_TYPE_*
    pub sensor_type: u8,
    pub himidity: f32,
    pub temperature: f32,
    pub voltage: f32,
}
impl Default for SensorValue {
    fn default() -> Self {
        Self {
            sensor_type: SENSOR_TYPE_H_T,
            himidity: 0.0,
            temperature: 0.0,
            voltage: 0.0,
        }
    }
}

impl SensorValue {
    pub fn humidity(h: f32) -> Self {
        Self {
            sensor_type: SENSOR_TYPE_HIMIDITY,
            himidity: h,
            ..Default::default()
        }
    }
    pub fn temperature(t: f32) -> Self {
        Self {
            sensor_type: SENSOR_TYPE_TEMPERATURE,
            temperature: t,
            ..Default::default()
        }
    }
    pub fn voltage(v: f32) -> Self {
        Self {
            sensor_type: SENSOR_TYPE_VOLTAGE,
            voltage: v,
            ..Default::default()
        }
    }
    pub fn humidity_temperature(h: f32, t: f32) -> Self {
        Self {
            sensor_type: SENSOR_TYPE_H_T,
            himidity: h,
            temperature: t,
            voltage: 0.0,
        }
    }
}
#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct SensorPacket {
    pub sensor_id: u8,
    pub sensor_data: SensorValue,
    pub timestamp: u64,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct EspMonitorConfiguration {
    pub max_nodes: u32,
    pub node_timeout: u32,
}

pub struct EspNowMonitor {
    pub esp_now: esp_idf_svc::espnow::EspNow,
    pub sensor_data: Arc<Mutex<Vec<(SensorPacket, u64)>>>, //(sensor_packet,cache_time_stamp)
    pub config: EspMonitorConfiguration,

    threads_handlers: Vec<JoinHandle<()>>,
    pools_running: Arc<Mutex<bool>>,
}

impl EspNowMonitor {
    pub fn new(config: &EspMonitorConfiguration) -> Result<Self, EspError> {
        let espnow_instance = esp_idf_svc::espnow::EspNow::take();
        if let Ok(esp_now) = espnow_instance {
            return Ok(Self {
                esp_now: esp_now,
                sensor_data: Arc::new(Mutex::new(vec![
                    (SensorPacket::default(), 0u64);
                    config.max_nodes as usize
                ])),
                config: *config,
                pools_running: Arc::new(Mutex::new(true)),
                threads_handlers: Vec::new(),
            });
        } else {
            return Err(espnow_instance.err().unwrap());
        }
    }

    //pool sensors and make exclude timed-out sensor
    pub fn start_pooling(&mut self) {
        let sensors = Arc::clone(&self.sensor_data);
        let still_polling = Arc::clone(&self.pools_running);
        let timeout = self.config.node_timeout;
        let mythread = std::thread::spawn(move || loop {
            {
                let running_flag = still_polling.lock().unwrap();
                if *running_flag == false {
                    break;
                }
                let mut sensors_arc = sensors.lock().unwrap();
                for (cached_packet, cache_time_stamp) in sensors_arc.iter_mut() {
                    if cached_packet.sensor_id != 0 {
                        let now_timestamp = unsafe { esp_idf_sys::esp_timer_get_time() } as u64;
                        if (*cache_time_stamp + timeout as u64 * 1000000) < now_timestamp {
                            println!("Sensor {} timout!", cached_packet.sensor_id);
                            *cached_packet = SensorPacket::default();
                        }
                    }
                }
            }
            FreeRtos::delay_ms(timeout * 1000);
        });
        self.threads_handlers.push(mythread);
    }
    // monitor sensors and update sensor cache
    pub fn start_monitoring(&mut self) {
        let sensors_arc = Arc::clone(&self.sensor_data);
        self.esp_now
            .register_recv_cb(move |_addr, payload| {
                let mut sensors = sensors_arc.lock().unwrap();
                let my_packet: SensorPacket = bincode::deserialize(&payload).unwrap_or_default();
                sensors[my_packet.sensor_id as usize].0 = my_packet;
                sensors[my_packet.sensor_id as usize].1 =
                    unsafe { esp_idf_sys::esp_timer_get_time() as _ };
                println!("received a packet: {:?}", my_packet);
            })
            .expect("Couldn't reset values");
    }
    pub fn get_sensors_instance(&self) -> Arc<Mutex<Vec<(SensorPacket, u64)>>> {
        Arc::clone(&self.sensor_data)
    }
}

impl Drop for EspNowMonitor {
    fn drop(&mut self) {
        *self.pools_running.lock().unwrap() = false;
        for i in self.threads_handlers.drain(..) {
            let tmp = i;
            if let Err(fk) = tmp.join() {
                println!("Error joining threads: {:?}", fk);
            }
        }
    }
}
