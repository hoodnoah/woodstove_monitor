// Trait for abstracting temperature reading
pub trait TemperatureSensor {
    type Error;

    fn read_temperature_f(&mut self) -> Result<f32, Self::Error>;
}

// Mock implementation for testing
pub struct MockSensor {
    // TODO: How do you want to control mock readings?
    // - Fixed value?
    // - Sequence of values?
    // - Function that returns temp based on time?
}

impl TemperatureSensor for MockSensor {
    type Error = ();

    fn read_temperature_f(&mut self) -> Result<f32, Self::Error> {
        todo!()
    }
}
