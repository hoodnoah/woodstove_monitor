use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::FreeRtos,
        gpio::*,
        peripherals::Peripherals,
        spi::{
            config::{Config, DriverConfig, Mode, Phase},
            *,
        },
        units::*,
    },
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use max31855::{Max31855, Unit};
use woodstove_lib::Temperature;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // setup SPI for thermocouple reads
    let peripherals = Peripherals::take().unwrap();

    let freq: Hertz = 4.MHz().into();
    let mode = Mode {
        polarity: config::Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let bus_config = DriverConfig::new();

    let config = Config::new().baudrate(freq).data_mode(mode);

    let mut spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio48,       // CLK
        peripherals.pins.gpio38,       // DO (dummy, just put one randomly)
        Some(peripherals.pins.gpio47), // sensor -> MCU
        Option::<AnyIOPin>::None,      // handled elsewhere, CS
        &bus_config,
        &config,
    )
    .unwrap();

    let mut cs = PinDriver::output(peripherals.pins.gpio21).unwrap();

    log::info!("SPI and CS configured successfully!");

    // Setup wifi
    log::info!("SSID: '{}'", WIFI_SSID);
    log::info!(
        "Password: '{}' (len: {})",
        WIFI_PASSWORD,
        WIFI_PASSWORD.len()
    );

    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop.clone(),
    )?;

    wifi.start()?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: WIFI_SSID.try_into().unwrap(),
        password: WIFI_PASSWORD.try_into().unwrap(),
        ..Default::default()
    }))?;

    wifi.connect()?;
    log::info!("wifi connected");

    loop {
        match Max31855::read_thermocouple(&mut spi, &mut cs, Unit::Celsius) {
            Ok(temp_c) => {
                let temp = Temperature::from_celsius(temp_c);
                log::info!(
                    "Temperature: {:.1}°F ({:.1}°C)",
                    temp.fahrenheit(),
                    temp.celsius()
                );
            }
            Err(e) => {
                log::error!("Sensor error: {:?}", e);
            }
        }

        FreeRtos::delay_ms(2000);
    }
}
