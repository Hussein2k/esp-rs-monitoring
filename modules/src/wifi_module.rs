use embedded_svc::wifi::AuthMethod;
use esp_idf_hal::modem;
use esp_idf_svc::wifi::BlockingWifi;

pub struct WiFiModule<'a> {
    pub mWiFi: esp_idf_svc::wifi::EspWifi<'a>,
}

impl<'a> WiFiModule<'_> {
    pub fn new_client(
        perph_modem: modem::Modem,
        (ssid, pass): (&str, &str),
    ) -> Result<WiFiModule<'a>, i32> {
        let ev_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take().unwrap();
        let nvs_partition = esp_idf_svc::nvs::EspDefaultNvsPartition::take().unwrap();
        let mut wifi = esp_idf_svc::wifi::EspWifi::new(perph_modem, ev_loop, Some(nvs_partition));
        if let Ok(mut wifi_ok) = wifi {
            if let Err(err_conf) =
                wifi_ok.set_configuration(&embedded_svc::wifi::Configuration::Client(
                    embedded_svc::wifi::ClientConfiguration {
                        ssid: ssid.into(),
                        password: pass.into(),

                        ..Default::default()
                    },
                ))
            {
                return Err(-1);
            }
            if let Err(e_wifi) = wifi_ok.start() {
                println!("Error ({}) starting wifi!", e_wifi);
                return Err(-2);
            }

            wifi_ok.connect();

            let mut rs = WiFiModule { mWiFi: wifi_ok };
            return Ok(rs);
        }
        Err(-3)
    }
    pub fn new_access_point(
        perph_modem: modem::Modem,
        (ssid, pass): (&str, &str),
    ) -> Result<WiFiModule<'a>, i32> {
        let ev_loop = esp_idf_svc::eventloop::EspSystemEventLoop::take().unwrap();
        let nvs_partition = esp_idf_svc::nvs::EspDefaultNvsPartition::take().unwrap();
        let mut wifi = esp_idf_svc::wifi::EspWifi::new(perph_modem, ev_loop, Some(nvs_partition));

        if let Ok(mut wifi_ok) = wifi {
            if let Err(err_conf) =
                wifi_ok.set_configuration(&embedded_svc::wifi::Configuration::AccessPoint(
                    embedded_svc::wifi::AccessPointConfiguration {
                        ssid: ssid.into(),
                        password: pass.into(),
                        auth_method: AuthMethod::WPA2Personal,
                        ..Default::default()
                    },
                ))
            {
                println!("Error dwwf");
                return Err(-1);
            }

            if let Err(e_wifi) = wifi_ok.start() {
                println!("Error ({}) starting wifi!", e_wifi);
                return Err(-2);
            }
            let mut rs = WiFiModule { mWiFi: wifi_ok };
            return Ok(rs);
        }
        Err(-3)
    }
}
