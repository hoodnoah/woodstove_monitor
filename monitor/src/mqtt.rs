use std::time::Duration;

use esp_idf_svc::{
    mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS},
    sys::EspError,
};
use woodstove_lib::Temperature;

const TEMP_TOPIC: &str = "woodstove/temperature";
const STATE_TOPIC: &str = "woodstove/state";
const TIME_IN_STATE_TOPIC: &str = "woodstove/time_in_state";
const STATUS_TOPIC: &str = "woodstove/status";
const ERROR_TOPIC: &str = "woodstove/error";

pub struct WoodstoveMQTT<'a> {
    client: EspMqttClient<'a>,
}

impl<'a> WoodstoveMQTT<'a> {
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
            ..Default::default()
        };

        let (client, _) = EspMqttClient::new(mqtt_endpoint, &config)?;

        Ok(Self { client })
    }

    pub fn publish_temperature(&mut self, temp: &Temperature) -> Result<u32, EspError> {
        self.client.publish(
            TEMP_TOPIC,
            QoS::AtMostOnce,
            false, // temp is constantly changing, so we *don't* want it retained
            temp.fahrenheit().to_string().as_bytes(),
        )
    }

    pub fn publish_state(&mut self, state: woodstove_lib::BurnState) -> Result<u32, EspError> {
        self.client.publish(
            STATE_TOPIC,
            QoS::AtLeastOnce,
            true, // state is set and kept, retain means we keep it
            state.to_string().as_bytes(),
        )
    }

    pub fn publish_time_in_state(&mut self, time_in_state: Duration) -> Result<u32, EspError> {
        self.client.publish(
            TIME_IN_STATE_TOPIC,
            QoS::AtLeastOnce,
            false,
            time_in_state.as_secs().to_string().as_bytes(),
        )
    }

    pub fn publish_status(&mut self) -> Result<u32, EspError> {
        self.client
            .publish(STATUS_TOPIC, QoS::AtLeastOnce, false, b"online")
    }

    pub fn publish_error(&mut self, error_msg: String) -> Result<u32, EspError> {
        self.client
            .publish(ERROR_TOPIC, QoS::AtLeastOnce, false, error_msg.as_bytes())
    }
}
