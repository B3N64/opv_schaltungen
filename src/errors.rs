use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    NegativeFrequency,
    CircuitConstructError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::NegativeFrequency => write!(f, "Frequency cannot be negative"),
            Error::CircuitConstructError(var) => write!(f, "Missing required variable: {}", var),
        }
    }
}
