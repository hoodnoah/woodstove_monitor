use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature {
    celsius: f32,
}

impl Temperature {
    pub fn from_celsius(c: f32) -> Self {
        Self { celsius: c }
    }

    pub fn from_fahrenheit(f: f32) -> Self {
        Self {
            celsius: (f - 32.0) * 5.0 / 9.0,
        }
    }

    pub fn celsius(&self) -> f32 {
        self.celsius
    }

    pub fn fahrenheit(&self) -> f32 {
        self.celsius * 9.0 / 5.0 + 32.0
    }
}

// Temperature - Temperature = TemperatureDelta
impl Sub for Temperature {
    type Output = TemperatureDelta;

    fn sub(self, other: Temperature) -> TemperatureDelta {
        TemperatureDelta {
            delta_celsius: self.celsius - other.celsius,
        }
    }
}

// Represents a temperature *difference*
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TemperatureDelta {
    delta_celsius: f32,
}

impl TemperatureDelta {
    pub fn from_celsius(c: f32) -> TemperatureDelta {
        TemperatureDelta { delta_celsius: c }
    }

    pub fn from_fahrenheit(f: f32) -> TemperatureDelta {
        TemperatureDelta {
            delta_celsius: f * 5.0 / 9.0,
        }
    }

    pub fn celsius(&self) -> f32 {
        self.delta_celsius
    }

    pub fn fahrenheit(&self) -> f32 {
        self.delta_celsius * 9.0 / 5.0 // no +-32 for deltas
    }
}

impl Div<f32> for TemperatureDelta {
    type Output = RateOfChange;

    fn div(self, seconds: f32) -> RateOfChange {
        RateOfChange {
            celsius_per_second: self.delta_celsius / seconds,
        }
    }
}

// Temperature rate of change
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct RateOfChange {
    celsius_per_second: f32,
}

impl RateOfChange {
    pub fn new_per_second(delta: TemperatureDelta, seconds: f32) -> Self {
        Self {
            celsius_per_second: delta.celsius() / seconds,
        }
    }

    pub fn new_per_minute(delta: TemperatureDelta, minutes: f32) -> Self {
        Self {
            celsius_per_second: delta.celsius() / (minutes * 60.0),
        }
    }

    pub fn celsius_per_second(&self) -> f32 {
        self.celsius_per_second
    }

    pub fn fahrenheit_per_second(&self) -> f32 {
        self.celsius_per_second * 9.0 / 5.0
    }

    pub fn fahrenheit_per_minute(&self) -> f32 {
        self.fahrenheit_per_second() * 60.0
    }
}

// f32 * RateOfChange = RateOfChange
impl Mul<RateOfChange> for f32 {
    type Output = RateOfChange;

    fn mul(self, roc: RateOfChange) -> RateOfChange {
        RateOfChange {
            celsius_per_second: self * roc.celsius_per_second,
        }
    }
}

impl Add for RateOfChange {
    type Output = RateOfChange;

    fn add(self, other: RateOfChange) -> RateOfChange {
        RateOfChange {
            celsius_per_second: self.celsius_per_second + other.celsius_per_second,
        }
    }
}

impl Sub for RateOfChange {
    type Output = RateOfChange;

    fn sub(self, other: RateOfChange) -> RateOfChange {
        RateOfChange {
            celsius_per_second: self.celsius_per_second - other.celsius_per_second,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_f_to_c() {
        let temp = Temperature::from_fahrenheit(32.0);

        assert!((temp.celsius() - 0.0).abs() < 0.01)
    }

    #[test]
    fn converts_c_to_f() {
        let temp = Temperature::from_celsius(100.0);

        assert!((temp.fahrenheit() - 212.0).abs() < 0.01)
    }

    #[test]
    fn roundtrip() {
        let original = 98.6;
        let temp = Temperature::from_fahrenheit(original);

        assert!((temp.fahrenheit() - original).abs() < 0.01);
    }
}
