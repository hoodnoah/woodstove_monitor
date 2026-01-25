mod mqtt;
mod wifi;

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
    sys::EspError,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use max31855::{Max31855, Unit};
use mqtt::WoodstoveMQTT;
use woodstove_lib::{StoveStateMachine, Temperature};

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
const MQTT_ENDPOINT: &str = env!("MQTT_ENDPOINT");
const MQTT_USER: &str = env!("MQTT_USER");
const MQTT_PASS: &str = env!("MQTT_PASS");

const LOOP_DELAY_MS: u32 = 10_000;

fn log_publish_result(name: &str, result: Result<u32, EspError>) {
    match result {
        Ok(_) => log::info!("Published {}", name),
        Err(e) => log::warn!("Failed to publish {}: {:?}", name, e),
    }
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // setup SPI for thermocouple reads
    let peripherals = Peripherals::take().unwrap();

    // setup error led
    let mut status_led = PinDriver::output(peripherals.pins.gpio8)?;

    let freq: Hertz = 4.MHz().into();
    let mode = Mode {
        polarity: config::Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let bus_config = DriverConfig::new();

    let config = Config::new().baudrate(freq).data_mode(mode);

    let mut spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio48,       // CLK (D13/GPIO48)
        peripherals.pins.gpio38,       // (dummy, just put one randomly)
        Some(peripherals.pins.gpio47), // DO (D12/GPIO47)
        Option::<AnyIOPin>::None,      // handled elsewhere, CS
        &bus_config,
        &config,
    )
    .unwrap();

    let mut cs = PinDriver::output(peripherals.pins.gpio21).unwrap(); // CS (D10/GPIO21)

    log::info!("SPI and CS configured successfully!");

    // Setup wifi
    let mut wifi_handler = wifi::WifiHandler::new(peripherals.modem, WIFI_SSID, WIFI_PASSWORD)?;
    wifi_handler.connect()?;
    log::info!("wifi connected");

    // Setup mqtt
    let mut mqtt_handler =
        WoodstoveMQTT::new("woodstove_monitor", MQTT_ENDPOINT, MQTT_USER, MQTT_PASS)?;

    // setup the state machine
    let mut stove_state_machine = StoveStateMachine::new();

    loop {
        match Max31855::read_thermocouple(&mut spi, &mut cs, Unit::Celsius) {
            Ok(temp_c) => {
                let temp = Temperature::from_celsius(temp_c);

                // publish temperature
                log_publish_result("temperature", mqtt_handler.publish_temperature(&temp));

                // update state machine
                let state_changed = stove_state_machine.update(temp);

                let state_string = stove_state_machine.current_state().to_string();

                if state_changed {
                    log::info!("State changed to: {}", state_string);
                }

                log_publish_result(
                    "state",
                    mqtt_handler.publish_state(stove_state_machine.current_state()),
                );

                // publish time in state every 6th loop
                log_publish_result(
                    "time in state",
                    mqtt_handler.publish_time_in_state(stove_state_machine.time_in_state()),
                );

                // publish status
                log_publish_result("status", mqtt_handler.publish_status());

                status_led.set_low().ok();
            }
            Err(e) => {
                let error_msg = format!("Sensor error: {:?}", e);

                if let Err(mqtt_err) = mqtt_handler.publish_error(error_msg.clone()) {
                    log::warn!("Failed to publish sensor error: {:?}", mqtt_err);
                }

                status_led.set_high().ok();

                log::error!("Sensor error: {:?}", e);
            }
        }

        FreeRtos::delay_ms(LOOP_DELAY_MS);
    }
}
