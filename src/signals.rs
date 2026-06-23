use crate::errors::{Error, Result};
use std::f64::consts::PI;
use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignalType {
    Constant,
    Sinus,
    Cosinus,
    Rectangular,
    Triangular,
}

impl SignalType {
    pub fn all() -> &'static [SignalType] {
        &[
            SignalType::Constant,
            SignalType::Sinus,
            SignalType::Cosinus,
            SignalType::Rectangular,
            SignalType::Triangular,
        ]
    }

    pub fn from_id(name: &str) -> Option<SignalType> {
        match name.to_lowercase().as_str() {
            "constant" => Some(SignalType::Constant),
            "sinus" => Some(SignalType::Sinus),
            "cosinus" => Some(SignalType::Cosinus),
            "rectangular" => Some(SignalType::Rectangular),
            "triangular" => Some(SignalType::Triangular),
            _ => None,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            SignalType::Constant => "constant",
            SignalType::Sinus => "sinus",
            SignalType::Cosinus => "cosinus",
            SignalType::Rectangular => "rectangular",
            SignalType::Triangular => "triangular",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SignalType::Constant => "Konstant",
            SignalType::Sinus => "Sinus",
            SignalType::Cosinus => "Cosinus",
            SignalType::Rectangular => "Rechteck",
            SignalType::Triangular => "Dreieck",
        }
    }

    pub fn variables(&self) -> &'static [&'static str] {
        match self {
            SignalType::Constant => &["Value"],
            SignalType::Sinus
            | SignalType::Cosinus
            | SignalType::Rectangular
            | SignalType::Triangular => &["Amplitude", "Frequency", "Phase"],
        }
    }

    pub fn construct(&self, values: &[f64]) -> Result<Box<dyn Signal>> {
        let require = |idx: usize, key: &'static str| {
            values
                .get(idx)
                .copied()
                .ok_or(Error::CircuitConstructError(key.to_string()))
        };

        match self {
            Self::Constant => Ok(Box::new(Constant::new(require(0, "Value")?))),

            Self::Sinus => {
                let amplitude = require(0, "Amplitude")?;
                let frequency = require(1, "Frequency")?;
                let phase = require(2, "Phase")?;

                if frequency < 0.0 {
                    return Err(Error::NegativeFrequency);
                }

                Ok(Box::new(Sinus::new(amplitude, frequency, phase)))
            }

            Self::Cosinus => {
                let amplitude = require(0, "Amplitude")?;
                let frequency = require(1, "Frequency")?;
                let phase = require(2, "Phase")?;

                if frequency < 0.0 {
                    return Err(Error::NegativeFrequency);
                }

                Ok(Box::new(Cosinus::new(amplitude, frequency, phase)))
            }

            Self::Rectangular => {
                let amplitude = require(0, "Amplitude")?;
                let frequency = require(1, "Frequency")?;
                let phase = require(2, "Phase")?;

                if frequency < 0.0 {
                    return Err(Error::NegativeFrequency);
                }

                Ok(Box::new(Rectangular::new(amplitude, frequency, phase)))
            }

            Self::Triangular => {
                let amplitude = require(0, "Amplitude")?;
                let frequency = require(1, "Frequency")?;
                let phase = require(2, "Phase")?;

                if frequency < 0.0 {
                    return Err(Error::NegativeFrequency);
                }

                Ok(Box::new(Triangular::new(amplitude, frequency, phase)))
            }
        }
    }
}

pub trait Signal {
    fn value_at(&self, t: f64) -> f64;
    fn frequency(&self) -> f64;
}

pub struct SignalParams {
    amplitude: f64,
    frequency: f64,
    phase: f64,
}

pub struct Constant {
    value: f64,
}

impl Constant {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl Signal for Constant {
    fn value_at(&self, _t: f64) -> f64 {
        self.value
    }

    fn frequency(&self) -> f64 {
        0.0
    }
}

pub struct Sinus {
    param: SignalParams,
}

impl Sinus {
    pub fn new(amplitude: f64, frequency: f64, phase: f64) -> Self {
        Self {
            param: SignalParams {
                amplitude,
                frequency,
                phase,
            },
        }
    }
}

impl Signal for Sinus {
    fn value_at(&self, t: f64) -> f64 {
        self.param.amplitude * (2.0 * PI * self.param.frequency * t + self.param.phase).sin()
    }

    fn frequency(&self) -> f64 {
        self.param.frequency
    }
}

pub struct Cosinus {
    param: SignalParams,
}

impl Cosinus {
    pub fn new(amplitude: f64, frequency: f64, phase: f64) -> Self {
        Self {
            param: SignalParams {
                amplitude,
                frequency,
                phase,
            },
        }
    }
}

impl Signal for Cosinus {
    fn value_at(&self, t: f64) -> f64 {
        self.param.amplitude * (2.0 * PI * self.param.frequency * t + self.param.phase).cos()
    }

    fn frequency(&self) -> f64 {
        self.param.frequency
    }
}

pub struct Rectangular {
    param: SignalParams,
}

impl Rectangular {
    pub fn new(amplitude: f64, frequency: f64, phase: f64) -> Self {
        Self {
            param: SignalParams {
                amplitude,
                frequency,
                phase,
            },
        }
    }
}

impl Signal for Rectangular {
    fn value_at(&self, t: f64) -> f64 {
        if (2.0 * PI * self.param.frequency * t + self.param.phase).sin() >= 0.0 {
            self.param.amplitude
        } else {
            -self.param.amplitude
        }
    }

    fn frequency(&self) -> f64 {
        self.param.frequency
    }
}

pub struct Triangular {
    param: SignalParams,
}

impl Triangular {
    pub fn new(amplitude: f64, frequency: f64, phase: f64) -> Self {
        Self {
            param: SignalParams {
                amplitude,
                frequency,
                phase,
            },
        }
    }
}

impl Signal for Triangular {
    fn value_at(&self, t: f64) -> f64 {
        let period = 1.0 / self.param.frequency;
        let x = ((t + self.param.phase / (2.0 * PI)) / period) % 1.0;

        if x < 0.25 {
            4.0 * self.param.amplitude * x
        } else if x < 0.75 {
            2.0 * self.param.amplitude - 4.0 * self.param.amplitude * x
        } else {
            -4.0 * self.param.amplitude + 4.0 * self.param.amplitude * x
        }
    }

    fn frequency(&self) -> f64 {
        self.param.frequency
    }
}

pub struct CombinedSignal<'a> {
    signal1: &'a dyn Signal,
    signal2: &'a dyn Signal,
}

impl<'a> CombinedSignal<'a> {
    pub fn new(signal1: &'a dyn Signal, signal2: &'a dyn Signal) -> Self {
        Self { signal1, signal2 }
    }
}

impl<'a> Signal for CombinedSignal<'a> {
    fn value_at(&self, t: f64) -> f64 {
        self.signal1.value_at(t) + self.signal2.value_at(t)
    }

    fn frequency(&self) -> f64 {
        self.signal1.frequency() + self.signal2.frequency()
    }
}

impl<'a> Add for &'a dyn Signal {
    type Output = CombinedSignal<'a>;

    fn add(self, other: &'a dyn Signal) -> CombinedSignal<'a> {
        CombinedSignal::new(self, other)
    }
}