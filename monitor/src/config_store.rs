use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};
use serde::{Deserialize, Serialize};
use woodstove_lib::{RateOfChange, StoveConfig, Temperature, TemperatureDelta};

const NVS_NAMESPACE: &str = "woodstove";
const NVS_CONFIG_KEY: &str = "config";
const CONFIG_BUF_LEN: usize = 512;

/// Human-readable config snapshot for serialization and NVS storage.
/// Temperatures are in °F; rates are in °F/min.
///
/// MQTT config update format (partial JSON — only include fields to change):
/// ```json
/// {
///   "idle_threshold": 150.0,
///   "active_threshold": 400.0,
///   "active_exit_threshold": 350.0,
///   "overheat_threshold": 700.0,
///   "overheat_exit_threshold": 600.0,
///   "rising_fast_rate": 5.0,
///   "falling_rate": -3.0,
///   "stable_rate": 1.2
/// }
/// ```
/// Publish to topic: `woodstove/config_update`
/// Current config is published (retained) to: `woodstove/config`
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoveConfigSnapshot {
    pub idle_threshold: f32,
    pub active_threshold: f32,
    pub active_exit_threshold: f32,
    pub overheat_threshold: f32,
    pub overheat_exit_threshold: f32,
    pub rising_fast_rate: f32,
    pub falling_rate: f32,
    pub stable_rate: f32,
}

/// Partial update received over MQTT. Omitted fields retain their current values.
#[derive(Deserialize, Debug, Default)]
struct StoveConfigUpdate {
    idle_threshold: Option<f32>,
    active_threshold: Option<f32>,
    active_exit_threshold: Option<f32>,
    overheat_threshold: Option<f32>,
    overheat_exit_threshold: Option<f32>,
    rising_fast_rate: Option<f32>,
    falling_rate: Option<f32>,
    stable_rate: Option<f32>,
}

impl From<&StoveConfig> for StoveConfigSnapshot {
    fn from(c: &StoveConfig) -> Self {
        Self {
            idle_threshold: c.idle_threshold.fahrenheit(),
            active_threshold: c.active_threshold.fahrenheit(),
            active_exit_threshold: c.active_exit_threshold.fahrenheit(),
            overheat_threshold: c.overheat_threshold.fahrenheit(),
            overheat_exit_threshold: c.overheat_exit_threshold.fahrenheit(),
            rising_fast_rate: c.rising_fast_rate.fahrenheit_per_minute(),
            falling_rate: c.falling_rate.fahrenheit_per_minute(),
            stable_rate: c.stable_rate.fahrenheit_per_minute(),
        }
    }
}

impl From<&StoveConfigSnapshot> for StoveConfig {
    fn from(s: &StoveConfigSnapshot) -> Self {
        Self {
            idle_threshold: Temperature::from_fahrenheit(s.idle_threshold),
            active_threshold: Temperature::from_fahrenheit(s.active_threshold),
            active_exit_threshold: Temperature::from_fahrenheit(s.active_exit_threshold),
            overheat_threshold: Temperature::from_fahrenheit(s.overheat_threshold),
            overheat_exit_threshold: Temperature::from_fahrenheit(s.overheat_exit_threshold),
            rising_fast_rate: RateOfChange::new_per_minute(
                TemperatureDelta::from_fahrenheit(s.rising_fast_rate),
                1.0,
            ),
            falling_rate: RateOfChange::new_per_minute(
                TemperatureDelta::from_fahrenheit(s.falling_rate),
                1.0,
            ),
            stable_rate: RateOfChange::new_per_minute(
                TemperatureDelta::from_fahrenheit(s.stable_rate),
                1.0,
            ),
        }
    }
}

/// Serialize a config to JSON. Used for publishing and NVS storage.
pub fn config_to_json(config: &StoveConfig) -> String {
    serde_json::to_string(&StoveConfigSnapshot::from(config)).unwrap_or_default()
}

/// Apply a partial JSON update to a config, returning the updated config.
/// Fields omitted from the JSON retain their current values.
pub fn apply_update(current: &StoveConfig, json: &str) -> Result<StoveConfig, serde_json::Error> {
    let update: StoveConfigUpdate = serde_json::from_str(json)?;
    let mut s = StoveConfigSnapshot::from(current);

    if let Some(v) = update.idle_threshold {
        s.idle_threshold = v;
    }
    if let Some(v) = update.active_threshold {
        s.active_threshold = v;
    }
    if let Some(v) = update.active_exit_threshold {
        s.active_exit_threshold = v;
    }
    if let Some(v) = update.overheat_threshold {
        s.overheat_threshold = v;
    }
    if let Some(v) = update.overheat_exit_threshold {
        s.overheat_exit_threshold = v;
    }
    if let Some(v) = update.rising_fast_rate {
        s.rising_fast_rate = v;
    }
    if let Some(v) = update.falling_rate {
        s.falling_rate = v;
    }
    if let Some(v) = update.stable_rate {
        s.stable_rate = v;
    }

    Ok(StoveConfig::from(&s))
}

/// Load config from NVS, falling back to defaults if not present or on error.
pub fn load_from_nvs(partition: &EspDefaultNvsPartition) -> StoveConfig {
    match load_inner(partition) {
        Some(config) => {
            log::info!("Loaded stove config from NVS");
            config
        }
        None => {
            log::info!("No stored config found, using defaults");
            StoveConfig::default()
        }
    }
}

fn load_inner(partition: &EspDefaultNvsPartition) -> Option<StoveConfig> {
    let nvs = EspNvs::new(partition.clone(), NVS_NAMESPACE, true)
        .map_err(|e| log::warn!("NVS open failed: {e:?}"))
        .ok()?;

    let mut buf = [0u8; CONFIG_BUF_LEN];
    let json = nvs
        .get_str(NVS_CONFIG_KEY, &mut buf)
        .map_err(|e| log::warn!("NVS read failed: {e:?}"))
        .ok()??; // second ? handles None (key not present)

    serde_json::from_str::<StoveConfigSnapshot>(json)
        .map_err(|e| log::warn!("NVS config parse failed: {e:?}"))
        .ok()
        .map(|s| StoveConfig::from(&s))
}

/// Persist the current config to NVS. Logs a warning on failure.
pub fn save_to_nvs(partition: &EspDefaultNvsPartition, config: &StoveConfig) {
    let json = config_to_json(config);
    match EspNvs::new(partition.clone(), NVS_NAMESPACE, true) {
        Ok(mut nvs) => match nvs.set_str(NVS_CONFIG_KEY, &json) {
            Ok(_) => log::info!("Config saved to NVS"),
            Err(e) => log::warn!("NVS write failed: {e:?}"),
        },
        Err(e) => log::warn!("NVS open for write failed: {e:?}"),
    }
}
