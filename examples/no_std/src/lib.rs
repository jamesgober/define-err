//! `define_errors!` and the verbs in a `#![no_std]` library. On a target without
//! compare-and-swap (`thumbv6m-none-eabi`) this still builds: `set_reporter`
//! does not exist, and every verb compiles to nothing.

#![no_std]

use define_err::define_errors;

define_errors! {
    namespace = "FW";

    pub enum SensorError {
        #[code(1, help = "Check the I2C wiring.")]
        #[category(Unavailable)]
        NotResponding { address: u8 } => "sensor at {address:#x} is not responding",

        #[code(2, help = "Recalibrate the sensor.")]
        #[severity(WARNING)]
        OutOfRange { reading: i32 } => "reading {reading} is out of range",
    }
}

/// Reads a sensor. Library code reports the same way on every target.
pub fn read(address: u8, reading: Option<i32>) -> Result<i32, SensorError> {
    let Some(reading) = reading else {
        let error = SensorError::NotResponding { address };
        define_err::report(&error);
        return Err(error);
    };
    if reading.abs() > 1_000 {
        define_err::warn!("clamping reading {reading}");
        define_err::soft(SensorError::OutOfRange { reading })?;
        return Ok(reading.clamp(-1_000, 1_000));
    }
    define_err::notice!("read {reading}");
    Ok(reading)
}
