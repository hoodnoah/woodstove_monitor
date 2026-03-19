use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use esp_idf_svc::{
    mqtt::client::{EspMqttClient, EspMqttConnection, EventPayload, MqttClientConfiguration, QoS},
    sys::EspError,
};
use woodstove_lib::{BurnState, StoveConfig, Temperature};

use crate::config_store;

const TEMP_TOPIC: &str = "woodstove/temperature";
const STATE_TOPIC: &str = "woodstove/state";
const TIME_IN_STATE_TOPIC: &str = "woodstove/time_in_state";
const STATUS_TOPIC: &str = "woodstove/status";
const ERROR_TOPIC: &str = "woodstove/error";
const CONFIG_TOPIC: &str = "woodstove/config";
const CONFIG_UPDATE_TOPIC: &str = "woodstove/config_update";

pub struct WoodstoveMQTT {
    client: EspMqttClient<'static>,
    pending_config: Arc<Mutex<Option<String>>>,
    // Set by the event thread on Connected; main thread reads and calls subscribe().
    // The event thread must NOT call subscribe() itself: the ESP-IDF MQTT task blocks
    // inside the event callback (share()), so calling any MQTT function from the event
    // thread deadlocks on the client's internal semaphore.
    needs_subscribe: Arc<AtomicBool>,
}

impl WoodstoveMQTT {
    pub fn new(
        client_id: &str,
        mqtt_endpoint: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, EspError> {
        let config = MqttClientConfiguration {
            client_id: Some(client_id),
            username: Some(username),
            password: Some(password),
            keep_alive_interval: Some(Duration::from_secs(30)),
            reconnect_timeout: Some(Duration::from_secs(5)),
            // Persist subscriptions so the broker restores them automatically on reconnect
            disable_clean_session: true,
            ..Default::default()
        };

        let (client, connection) = EspMqttClient::new(mqtt_endpoint, &config)?;

        let pending = Arc::new(Mutex::new(None::<String>));
        let needs_subscribe = Arc::new(AtomicBool::new(false));

        spawn_event_thread(connection, pending.clone(), needs_subscribe.clone());

        Ok(Self {
            client,
            pending_config: pending,
            needs_subscribe,
        })
    }

    /// Subscribe to config updates if the MQTT connection (re)established since the last call.
    /// Must be called from the main thread, not from within the MQTT event thread.
    pub fn subscribe_if_needed(&mut self) {
        if self.needs_subscribe.swap(false, Ordering::Relaxed) {
            log::info!("MQTT connected, subscribing to config updates");
            if let Err(e) = self.client.subscribe(CONFIG_UPDATE_TOPIC, QoS::AtLeastOnce) {
                log::warn!("Failed to subscribe to {CONFIG_UPDATE_TOPIC}: {e:?}");
            }
        }
    }

    /// Take any config update JSON payload received since the last call.
    pub fn take_config_update(&self) -> Option<String> {
        self.pending_config.lock().unwrap().take()
    }

    /// Publish current config as retained JSON to `woodstove/config`.
    pub fn publish_config(&mut self, config: &StoveConfig) -> Result<u32, EspError> {
        let json = config_store::config_to_json(config);
        self.client
            .publish(CONFIG_TOPIC, QoS::AtLeastOnce, true, json.as_bytes())
    }

    pub fn publish_temperature(&mut self, temp: &Temperature) -> Result<u32, EspError> {
        self.client.publish(
            TEMP_TOPIC,
            QoS::AtMostOnce,
            false,
            temp.fahrenheit().to_string().as_bytes(),
        )
    }

    pub fn publish_state(&mut self, state: BurnState) -> Result<u32, EspError> {
        self.client.publish(
            STATE_TOPIC,
            QoS::AtLeastOnce,
            true,
            state.to_string().as_bytes(),
        )
    }

    pub fn publish_time_in_state(&mut self, time_in_state: Duration) -> Result<u32, EspError> {
        self.client.publish(
            TIME_IN_STATE_TOPIC,
            QoS::AtMostOnce,
            false,
            time_in_state.as_secs().to_string().as_bytes(),
        )
    }

    pub fn publish_status(&mut self) -> Result<u32, EspError> {
        self.client
            .publish(STATUS_TOPIC, QoS::AtMostOnce, false, b"online")
    }

    pub fn publish_error(&mut self, error_msg: String) -> Result<u32, EspError> {
        self.client
            .publish(ERROR_TOPIC, QoS::AtMostOnce, false, error_msg.as_bytes())
    }
}

fn spawn_event_thread(
    mut connection: EspMqttConnection,
    pending: Arc<Mutex<Option<String>>>,
    needs_subscribe: Arc<AtomicBool>,
) {
    std::thread::Builder::new()
        .stack_size(6144)
        .spawn(move || loop {
            match connection.next() {
                Ok(event) => match event.payload() {
                    EventPayload::Connected(_) => {
                        // Signal the main thread to subscribe. Do NOT call subscribe() here:
                        // the MQTT task is blocked inside the event callback at this point,
                        // so any re-entrant MQTT call would deadlock on its internal semaphore.
                        needs_subscribe.store(true, Ordering::Relaxed);
                    }
                    EventPayload::Received { topic, data, .. } => {
                        if topic == Some(CONFIG_UPDATE_TOPIC) {
                            match core::str::from_utf8(data) {
                                Ok(json) => {
                                    log::info!("Received config update: {json}");
                                    *pending.lock().unwrap() = Some(json.to_string());
                                }
                                Err(e) => log::warn!("Config update: invalid UTF-8: {e:?}"),
                            }
                        }
                    }
                    _ => {}
                },
                Err(e) => {
                    log::warn!("MQTT connection closed: {e:?}");
                    break;
                }
            }
        })
        .expect("MQTT event thread spawn failed");
}
