pub mod sensor;
pub mod state_machine;

pub use sensor::TemperatureSensor;
pub use state_machine::{BurnState, StoveStateMachine};
