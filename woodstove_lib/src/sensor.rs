use crate::temperature::Temperature;

#[cfg(feature = "max31855")]
pub mod max31855_sensor {
    use super::*;
    use embedded_hal::digital::OutputPin;
    use max31855::{Max31855, Unit};

    pub fn read_max31855<SPI, CS, SpiE, CsE>(
        spi: &mut SPI,
        cs: &mut CS,
    ) -> Result<Temperature, max31855::Error<SpiE, CsE>>
    where
        SPI: Max31855<SpiE, CsE, CS>,
        CS: OutputPin<Error = CsE>,
    {
        let temp_c = spi.read_thermocouple(cs, Unit::Celsius)?;
        Ok(Temperature::from_celsius(temp_c))
    }
}
