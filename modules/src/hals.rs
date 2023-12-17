pub struct ADCWrapper {
    adc_num: i32,
    adc_chan: i32,
    adc_nbits: i32,
}
impl Default for ADCWrapper {
    fn default() -> Self {
        ADCWrapper::new(
            1,
            esp_idf_sys::adc_bits_width_t_ADC_WIDTH_BIT_12 as _,
            esp_idf_sys::adc_channel_t_ADC_CHANNEL_4 as _,
            esp_idf_sys::adc_atten_t_ADC_ATTEN_DB_11 as _,
        )
    }
}
impl ADCWrapper {
    fn initialize_adc1(
        nbits: esp_idf_sys::adc_bits_width_t,
        chan: esp_idf_sys::adc1_channel_t,
        atten: esp_idf_sys::adc_atten_t,
    ) {
        unsafe {
            esp_idf_sys::adc1_config_channel_atten(chan, atten);
            esp_idf_sys::adc1_config_width(nbits);
        }
    }
    fn initialize_adc2(chan: esp_idf_sys::adc2_channel_t, atten: esp_idf_sys::adc_atten_t) {
        unsafe {
            esp_idf_sys::adc2_config_channel_atten(chan, atten);
        }
    }

    pub fn new(adc_num: i32, adc_nbits: i32, adc_chan: i32, adc_atten: i32) -> Self {
        if adc_num == 1 {
            Self::initialize_adc1(adc_nbits as _, adc_chan as _, adc_atten as _);
        } else if adc_num == 2 {
            Self::initialize_adc2(adc_chan as _, adc_atten as _);
        }
        return Self {
            adc_num,
            adc_chan,
            adc_nbits,
        };
    }
    pub fn get_value(&self) -> i32 {
        if self.adc_num == 1 {
            return unsafe { esp_idf_sys::adc1_get_raw(self.adc_chan as _) as _ };
        } else {
            let mut tmp = 0i32;
            unsafe { esp_idf_sys::adc2_get_raw(self.adc_chan as _, self.adc_nbits as _, &mut tmp) };
            return tmp;
        }
    }
}


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