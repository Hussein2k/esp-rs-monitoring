use std::{
    ops::Sub,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use super::monitoring_module::SensorArc;
use super::monitoring_module::*;
use embedded_svc::wifi::ClientConfiguration;
use esp_idf_hal::{
    delay::{self, FreeRtos},
    peripherals::Peripherals,
};
use esp_idf_svc::{errors::EspIOError, eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys::{esp_now_peer_info, esp_timer_get_time, EspError};
use serde::{Deserialize, Serialize};

pub const MEASUREMENT_TYPE_HUMIDITY: u8 = 1;
pub const MEASUREMENT_TYPE_TEMPERATURE: u8 = 2;
pub const MEASUREMENT_TYPE_VOLTAGE: u8 = 4;
pub const MEASUREMENT_TYPE_NONE_ALWAYS_ON: u8 = 8;
pub const MEASUREMENT_TYPE_NONE_ALWAYS_OFF: u8 = 16;

#[derive(Debug, Default, Clone, Copy, serde::Serialize, serde::Deserialize)]

/*
Specifiy Relay state dependence
*/
pub struct RelayStateDependence {
    pub sensor_id: u8,
    //MEASUREMENT_TYPE_*
    pub measurement_type: u8,
    pub range: (f32, f32),
}

impl RelayStateDependence {
    pub fn validate(&self) -> bool {
        if self.sensor_id == 0 || self.measurement_type > 16 {
            return false;
        }
        match self.measurement_type {
            MEASUREMENT_TYPE_HUMIDITY => (self.range.0 < 100.0) && (self.range.1 > 0.0),
            MEASUREMENT_TYPE_TEMPERATURE => (self.range.0 < 125.0) && (self.range.1 > -40.0),
            MEASUREMENT_TYPE_VOLTAGE => (self.range.0 < 5.0) && (self.range.1 > 0.0),
            MEASUREMENT_TYPE_NONE_ALWAYS_ON => true,
            MEASUREMENT_TYPE_NONE_ALWAYS_OFF => true,
            _ => false,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RelayConfiguration {
    pub sensor_id: u8,
    pub relay_id: u8,
    pub mode: u8,
    pub auto_dependence: u8,
    pub min_value: f32,
    pub max_value: f32,
}

pub type RelayArc = Arc<Mutex<[RelayStateDependence; 8]>>; //not all value shall be shared!

pub struct RelayStateController {
    pins: [u8; 8],
    last_update: [i64; 8],
    sensors_instance: SensorArc,
    relay_states_criteria: RelayArc,
    past_sensor_values: Arc<Mutex<[(SensorPacket, u32, i64); 8]>>,
    // past_relay_states: [RelayStateDependence; 8],
}
#[derive(Clone, Copy)]
pub enum RelayOutput {
    ON,
    OFF,
    NoChange,
}
impl RelayStateController {
    pub fn set_gpio(pin_num: u8, enable: bool) {
        unsafe {
            let gpio_conf = esp_idf_sys::gpio_config_t {
                mode: if enable {
                    esp_idf_sys::gpio_mode_t_GPIO_MODE_OUTPUT
                } else {
                    0
                },
                intr_type: esp_idf_sys::gpio_int_type_t_GPIO_INTR_DISABLE,
                pin_bit_mask: 1 << pin_num,
                pull_down_en: esp_idf_sys::gpio_pulldown_t_GPIO_PULLDOWN_ENABLE,
                pull_up_en: 0,
            };
            esp_idf_sys::gpio_config(&gpio_conf);
        }
    }
    pub fn new(relay_pins: &[u8; 8], sensors_instance: SensorArc) -> Result<Self, i32> {
        for i in relay_pins {
            // if *i > 32 {
            //     return Err(-1);
            // }
            Self::set_gpio(*i, true);
            unsafe { esp_idf_sys::gpio_set_level(*i as _, 1) };
        }
        Ok(Self {
            pins: relay_pins.clone(),
            last_update: [0i64; 8],
            relay_states_criteria: Arc::new(Mutex::new([RelayStateDependence::default(); 8])),
            sensors_instance: sensors_instance,
            past_sensor_values: Arc::new(Mutex::new([(SensorPacket::default(), 0u32, 0i64); 8])), // past_relay_states: [RelayStateDependence::default(); 8],
        })
    }
    pub fn update_values(&self, new_vals: &[bool; 8]) {
        for i in 0..new_vals.len() {
            unsafe { esp_idf_sys::gpio_set_level(self.pins[i] as _, new_vals[i] as _) };
        }
    }
    pub fn update_a_value(&self, relay_id: u8, level: bool) {
        unsafe {
            esp_idf_sys::gpio_set_level(self.pins[(relay_id) as usize] as _, level as _);
        }
    }
    //  fn update_pins_levels(&self, new_vals: &[bool; 8]) {
    //     for i in 0..new_vals.len() {
    //         unsafe { esp_idf_sys::gpio_set_level(self.pins[i] as _, new_vals[i] as _) };
    //     }
    // }
    //  fn update_a_pin_level(&self, relay_id: u8, level: bool) {
    //     unsafe {
    //         esp_idf_sys::gpio_set_level(self.pins[(relay_id) as usize] as _, level as _);
    //     }
    // }

    pub fn get_relays_criteria_instance(&self) -> RelayArc {
        self.relay_states_criteria.clone()
    }
    pub fn get_status_instance(&self) -> Arc<std::sync::Mutex<[(SensorPacket, u32, i64); 8]>> {
        self.past_sensor_values.clone()
    }

    fn check_criteria(
        criteria: &RelayStateDependence,
        last_change_timestamp: i64,
        current_sensor_value: &SensorValue,
        past_sensor_value: &SensorValue,
        tolerance_percentage: f32,
        timeout_sec: f32,
    ) -> RelayOutput {
        // println!("============================================");
        let local_tolerance = tolerance_percentage.min(25.0).max(0.0) / 100.0;
        let humidity_tolerance = local_tolerance * (100.0 - 0.0);
        let temperature_tolerance = local_tolerance * (125.0 - (-40.0));
        let voltage_tolerance = local_tolerance * (5.0 - 0.0);

        // if unsafe { esp_idf_sys::esp_timer_get_time() - last_change_timestamp }
        //     < timeout_sec as i64 * 1000000
        // {
        //     return RelayOutput::NoChange;
        // }
        // println!(
        //     "Current sensor value:{:?}\nPast sensor value:{:?}",
        //     current_sensor_value, past_sensor_value
        // );
        println!("criteria: {:?}", criteria);
        match criteria.measurement_type {
            MEASUREMENT_TYPE_HUMIDITY => {
                println!(
                    "Humidity Test: Current_Humidity: {}\t Past_Humidity: {}\tTolerance: {}",
                    current_sensor_value.himidity, past_sensor_value.himidity, humidity_tolerance
                );
                // if f32::abs(current_sensor_value.himidity - past_sensor_value.himidity)
                //     > humidity_tolerance
                // {
                if (current_sensor_value.himidity > criteria.range.0)
                    && (current_sensor_value.himidity < criteria.range.1)
                {
                    return RelayOutput::ON;
                } else {
                    return RelayOutput::OFF;
                }
                // } else {
                //     return RelayOutput::NoChange;
                // }
            }
            MEASUREMENT_TYPE_TEMPERATURE => {
                // if (f32::abs(current_sensor_value.temperature - past_sensor_value.temperature)
                //     > temperature_tolerance)
                // {
                if (current_sensor_value.temperature > criteria.range.0)
                    && (current_sensor_value.temperature < criteria.range.1)
                {
                    return RelayOutput::ON;
                } else {
                    return RelayOutput::OFF;
                }
                // } else {
                //     return RelayOutput::NoChange;
                // }
            }
            MEASUREMENT_TYPE_VOLTAGE => {
                // if (f32::abs(current_sensor_value.voltage - past_sensor_value.voltage)
                //     > voltage_tolerance)
                // {
                if (current_sensor_value.voltage > criteria.range.0)
                    && (current_sensor_value.voltage < criteria.range.1)
                {
                    return RelayOutput::ON;
                } else {
                    return RelayOutput::OFF;
                }
                // } else {
                //     return RelayOutput::NoChange;
                // }
            }
            MEASUREMENT_TYPE_NONE_ALWAYS_ON => return RelayOutput::ON,
            MEASUREMENT_TYPE_NONE_ALWAYS_OFF => return RelayOutput::OFF,
            _ => return RelayOutput::NoChange,
        }
    }

    pub fn update_criteria(&mut self, relay_id: u8, criteria: &RelayStateDependence) {
        let local_relay_criteria = self.relay_states_criteria.clone();
        let mut cri_list = local_relay_criteria.lock().unwrap();
        cri_list[relay_id as usize] = *criteria;
    }
    pub fn start_polling(&mut self, tolerance_percentage: f32, timeout: f32) {
        let local_relay_criteria = self.relay_states_criteria.clone();
        let local_past_sensors = self.past_sensor_values.clone();
        let local_sensors = self.sensors_instance.clone();

        let mut past_relays_states = [RelayOutput::NoChange; 8];

        let relay_pins = self.pins;
        std::thread::spawn(move || loop {
            {
                let criteria = local_relay_criteria.lock().unwrap();
                let mut past_sensors = local_past_sensors.lock().unwrap();
                let sensors = local_sensors.lock().unwrap();
                // println!("=================================\ncriteria: {:?}\npast_sensor_values: {:?}\nsensors: {:?}",criteria,past_sensors,sensors);

                for relay_id in 0..criteria.len() {
                    let relay_criteria = &criteria[relay_id];
                    // let current_sensor_data = SensorPacket::default();//sensors[relay.0.sensor_id as usize].clone();
                    // println!(
                    //     "{:?}================{:?}====================={:?}",
                    //     criteria[relay_id],
                    //     relay_criteria,
                    //     sensors[relay_criteria.sensor_id as usize]
                    // );
                    if relay_criteria.sensor_id == 0 {
                        continue;
                    }
                    // println!(
                    //     "relay {} sensor {} humidity :{} | criteria_humidity: {:?}",
                    //     relay_id,
                    //     relay_criteria.sensor_id,
                    //     sensors[relay_criteria.sensor_id as usize]
                    //         .0
                    //         .sensor_data
                    //         .himidity,
                    //     relay_criteria.range
                    // );

                    match Self::check_criteria(
                        &criteria[relay_id],
                        past_sensors[relay_id].2 as _,
                        &sensors[relay_criteria.sensor_id as usize].0.sensor_data,
                        &past_sensors[relay_id].0.sensor_data,
                        tolerance_percentage,
                        timeout,
                    ) {
                        RelayOutput::ON => unsafe {
                            //ACTIVE LOW RELAYS
                            // if let RelayOutput::OFF = past_relays_states[relay_id] {
                            println!("Relay{}'s state changed to : {}", relay_id, "ON");
                            esp_idf_sys::gpio_set_level(relay_pins[relay_id] as _, 0);
                            past_sensors[relay_id].0 = sensors[relay_criteria.sensor_id as usize].0;
                            past_sensors[relay_id].2 = esp_idf_sys::esp_timer_get_time();
                            past_relays_states[relay_id] = RelayOutput::ON;
                            // }
                        },
                        RelayOutput::OFF => unsafe {
                            println!("Relay{}'s state changed to : {}", relay_id, "OFF");
                            esp_idf_sys::gpio_set_level(relay_pins[relay_id] as _, 1);
                            past_sensors[relay_id].0 = sensors[relay_criteria.sensor_id as usize].0;
                            past_sensors[relay_id].2 = esp_idf_sys::esp_timer_get_time();
                            past_relays_states[relay_id] = RelayOutput::OFF;
                        },
                        RelayOutput::NoChange => {
                            // println!("Relay{}'s state: {}", relay_id, "NoChange");
                        }
                    }
                }
            }
            FreeRtos::delay_ms(1000);
        });
    }
}

impl Drop for RelayStateController {
    fn drop(&mut self) {
        for i in self.pins.iter() {
            Self::set_gpio(*i, false);
        }
    }
}
