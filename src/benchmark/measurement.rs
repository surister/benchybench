use std::time::SystemTime;

/// Represents a `value` measured at a given `time`
#[derive(Debug)]
pub(crate) struct Measurement {
    /// The time the measured value was taken
    pub(crate) time: SystemTime,
    /// The measured value
    pub(crate) value: i32
}

impl Measurement {
    pub fn new(time: SystemTime, value: i32) -> Measurement {
        Measurement {
            time,
            value,
        }
    }
}