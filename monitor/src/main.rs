mod config_store;
mod mqtt;
mod wifi;

use esp_idf_svc::{
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
const WIFI_CHECK_INTERVAL: u32 = 5; // every 5 loops, 50 seconds

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

    let freq: Hertz = 4.MHz().into();
    let mode = Mode {
        polarity: config::Polarity::IdleLow,
        phase: Phase::CaptureOnFirstTransition,
    };

    let bus_config = DriverConfig::new();

    let config = Config::new().baudrate(freq).data_mode(mode);

    let mut spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio1,       // CLK/SCK (GPIO1)
        peripherals.pins.gpio21,      // (dummy, just put one randomly)
        Some(peripherals.pins.gpio4), // DO (GPIO4)
        Option::<AnyIOPin>::None,     // handled elsewhere, CS
        &bus_config,
        &config,
    )
    .unwrap();

    let mut cs = PinDriver::output(peripherals.pins.gpio3).unwrap(); // CS (GPIO3)

    log::info!("SPI and CS configured successfully!");

    // Take the NVS partition once. It's Arc-backed so we can clone it for WiFi.
    let nvs = EspDefaultNvsPartition::take()?;

    // Load persisted config (falls back to defaults on first boot or parse error)
    let initial_config = config_store::load_from_nvs(&nvs);

    // Setup wifi (needs the NVS partition for calibration data)
    let mut wifi_handler =
        wifi::WifiHandler::new(peripherals.modem, WIFI_SSID, WIFI_PASSWORD, nvs.clone())?;
    wifi_handler.connect()?;
    log::info!("wifi connected");

    // Setup mqtt
    let mut mqtt_handler =
        WoodstoveMQTT::new("woodstove_monitor", MQTT_ENDPOINT, MQTT_USER, MQTT_PASS)?;

    // Publish the active config on boot so the broker reflects current thresholds
    log_publish_result("config", mqtt_handler.publish_config(&initial_config));

    // Setup the state machine with the loaded (or default) config
    let mut stove_state_machine = StoveStateMachine::with_config(initial_config);

    let mut wifi_connected = true;
    let mut loops_since_wifi_check = 0u32;

    loop {
        if loops_since_wifi_check >= WIFI_CHECK_INTERVAL {
            match wifi_handler.ensure_connected() {
                Ok(_) => {
                    if !wifi_connected {
                        log::info!("WiFi connection restored");
                    }
                    wifi_connected = true;
                }
                Err(e) => {
                    log::error!("WiFi check failed: {:?}", e);
                    wifi_connected = false;
                }
            }
            loops_since_wifi_check = 0;
        }

        // Subscribe after MQTT connects or reconnects (safe to call every iteration)
        mqtt_handler.subscribe_if_needed();

        // Check for a pending config update received via MQTT
        if let Some(json) = mqtt_handler.take_config_update() {
            match config_store::apply_update(stove_state_machine.config(), &json) {
                Ok(new_config) => {
                    config_store::save_to_nvs(&nvs, &new_config);
                    log_publish_result("config", mqtt_handler.publish_config(&new_config));
                    stove_state_machine.update_config(new_config);
                    log::info!("Config updated and persisted");
                }
                Err(e) => log::warn!("Invalid config update payload: {e:?}"),
            }
        }

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
            }
            Err(e) => {
                let error_msg = format!("Sensor error: {:?}", e);

                if let Err(mqtt_err) = mqtt_handler.publish_error(error_msg.clone()) {
                    log::warn!("Failed to publish sensor error: {:?}", mqtt_err);
                }

                log::error!("Sensor error: {:?}", e);
            }
        }

        loops_since_wifi_check += 1;
        FreeRtos::delay_ms(LOOP_DELAY_MS);
    }
}
