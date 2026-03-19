use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{delay::FreeRtos, modem::Modem},
    nvs::EspDefaultNvsPartition,
    sys::EspError,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

pub struct WifiHandler<'a> {
    wifi: BlockingWifi<EspWifi<'a>>,
}

impl<'a> WifiHandler<'a> {
    pub fn new(
        modem: Modem,
        wifi_ssid: &str,
        wifi_password: &str,
        nvs: EspDefaultNvsPartition,
    ) -> Result<Self, EspError> {
        let sys_loop = EspSystemEventLoop::take()?;

        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
            sys_loop.clone(),
        )?;

        wifi.start()?;

        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: wifi_ssid.try_into().unwrap(),
            password: wifi_password.try_into().unwrap(),
            ..Default::default()
        }))?;

        Ok(Self { wifi: wifi })
    }

    pub fn connect(&mut self) -> Result<(), EspError> {
        self.wifi.connect()?;

        // give the connection a moment
        FreeRtos::delay_ms(10_000);
        Ok(())
    }

    pub fn is_connected(&self) -> Result<bool, EspError> {
        self.wifi.is_connected()
    }

    pub fn ensure_connected(&mut self) -> Result<bool, EspError> {
        if !self.is_connected()? {
            log::warn!("WiFi disconnected, attempting reconnection...");
            match self.connect() {
                Ok(_) => {
                    log::info!("WiFi reconnected successfully");
                    Ok(true)
                }
                Err(e) => {
                    log::error!("WiFi reconnection failed: {:?}", e);
                    Err(e)
                }
            }
        } else {
            Ok(true)
        }
    }
}
