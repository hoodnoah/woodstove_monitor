use esp_idf_svc::hal::{
    delay::FreeRtos,
    gpio::*,
    peripherals::Peripherals,
    spi::{
        config::{Config, DriverConfig, Mode, Phase},
        *,
    },
    units::*,
};
use max31855::{Max31855, Unit};
use woodstove_lib::Temperature;

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Beginning test of temperature measurement...");

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
        peripherals.pins.gpio6,
        peripherals.pins.gpio10,
        Some(peripherals.pins.gpio2),
        Option::<AnyIOPin>::None,
        &bus_config,
        &config,
    )
    .unwrap();

    let mut cs = PinDriver::output(peripherals.pins.gpio7).unwrap();

    log::info!("SPI and CS configured successfully!");

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
