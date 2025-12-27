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
    mqtt::client::*,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use max31855::{Max31855, Unit};
use woodstove_lib::{BurnState, StoveStateMachine, Temperature};

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");
const MQTT_ENDPOINT: &str = env!("MQTT_ENDPOINT");
const MQTT_USER: &str = env!("MQTT_USER");
const MQTT_PASS: &str = env!("MQTT_PASS");

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
    FreeRtos::delay_ms(10000); // sleep for 10 seconds to just let everybody chill
    log::info!("wifi connected");

    let mqtt_config = MqttClientConfiguration {
        client_id: Some("woodstove_monitor"),
        username: Some(MQTT_USER),
        password: Some(MQTT_PASS),
        ..Default::default()
    };

    let (mut mqtt_client, _) = EspMqttClient::new(MQTT_ENDPOINT, &mqtt_config)?;

    // setup the state machine
    let mut stove_state_machine = StoveStateMachine::new();

    loop {
        match Max31855::read_thermocouple(&mut spi, &mut cs, Unit::Celsius) {
            Ok(temp_c) => {
                let temp = Temperature::from_celsius(temp_c);

                // publish temperature
                mqtt_client
                    .publish(
                        "woodstove/temperature",
                        QoS::AtMostOnce,
                        false,
                        temp.fahrenheit().to_string().as_bytes(),
                    )
                    .ok();

                // update state machine
                let state_changed = stove_state_machine.update(temp);

                let state_string = match stove_state_machine.current_state() {
                    BurnState::Idle => "idle",
                    BurnState::Startup => "startup",
                    BurnState::ActiveBurn => "active_burn",
                    BurnState::Coaling => "coaling",
                    BurnState::Overheat => "overheat",
                };

                if state_changed {
                    log::info!("State changed to: {}", state_string);
                }

                mqtt_client
                    .publish(
                        "woodstove/state",
                        QoS::AtLeastOnce,
                        true,
                        state_string.as_bytes(),
                    )
                    .ok();

                log::info!("Published state as {}", state_string);

                // publish time in state every 6th loop
                mqtt_client
                    .publish(
                        "woodstove/time_in_state",
                        QoS::AtLeastOnce,
                        false,
                        stove_state_machine
                            .time_in_state()
                            .as_secs()
                            .to_string()
                            .as_bytes(),
                    )
                    .ok();

                log::info!("Logged time in state");

                // publish status
                mqtt_client
                    .publish("woodstove/status", QoS::AtMostOnce, true, b"online")
                    .ok();

                status_led.set_low().ok();

                log::info!("Published: {:.1}F", temp.fahrenheit());
            }
            Err(e) => {
                let error_msg = format!("Sensor error: {:?}", e);
                mqtt_client
                    .publish(
                        "woodstove/error",
                        QoS::AtLeastOnce,
                        false,
                        error_msg.as_bytes(),
                    )
                    .ok();

                status_led.set_high().ok();

                log::error!("Sensor error: {:?}", e);
            }
        }

        FreeRtos::delay_ms(10000);
    }
}
