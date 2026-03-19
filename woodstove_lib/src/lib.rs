pub mod sensor;
pub mod state_machine;
pub mod temperature;

pub use sensor::max31855_sensor;
pub use state_machine::{BurnState, StoveConfig, StoveStateMachine};
pub use temperature::{RateOfChange, Temperature, TemperatureDelta};
