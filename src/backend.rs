use plotters;

use crate::errors::{Error, Result};
use std::f64::consts::PI;

pub enum SignalType {
    Rectangular,
    Triangle,
    Cosinus,
    Sinus,
}

impl SignalType {
    pub fn from_str(type_name: &str) -> Result<Self> {
        match type_name {
            "rectangular" => Ok(Self::Rectangular),
            "triangle" => Ok(Self::Triangle),
            "cosinus" => Ok(Self::Cosinus),
            "sinus" => Ok(Self::Sinus),
            _ => Err(Error::syntax(format!("Invalid signal type: {}", type_name))),
        }
    }

    fn function(&self) -> fn(f64, f64, f64) -> f64 {
        match self {
            Self::Rectangular => Self::rectangular,
            Self::Triangle => Self::triangle,
            Self::Cosinus => Self::cosinus,
            Self::Sinus => Self::sinus,
        }
    }

    fn sinus(t: f64, amplitude: f64, pulsatance: f64) -> f64 {
        amplitude * (pulsatance * t).sin()
    }

    fn cosinus(t: f64, amplitude: f64, pulsatance: f64) -> f64 {
        amplitude * (pulsatance * t).cos()
    }

    fn rectangular(t: f64, amplitude: f64, pulsatance: f64) -> f64 {
        if (pulsatance * t).sin() >= 0.0 {
            amplitude
        } else {
            -amplitude
        }
    }

    fn triangle(t: f64, amplitude: f64, pulsatance: f64) -> f64 {
        let period = 2.0 * PI / pulsatance;
        let x = (t / period) % 1.0;

        if x < 0.25 {
            4.0 * amplitude * x
        } else if x < 0.75 {
            2.0 * amplitude - 4.0 * amplitude * x
        } else {
            -4.0 * amplitude + 4.0 * amplitude * x
        }
    }
}

pub struct Signal {
    amplitude: f64,
    pulsatance: f64,
    function: fn(f64, f64, f64) -> f64,
}

impl Signal {
    pub fn new(signal_type: SignalType, amplitude: f64, frequency: f64, phase_shift: f64) -> Self {
        let function = signal_type.function();
        let pulsatance = 2.0 * PI * frequency + phase_shift;

        Self {
            amplitude,
            pulsatance,
            function,
        }
    }

    pub fn voltage(&self, t: f64) -> f64 {
        (self.function)(t, self.amplitude, self.pulsatance)
    }
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            amplitude: 1.0,
            pulsatance: 2.0 * PI,
            function: SignalType::Sinus.function(),
        }
    }
}

enum SchaltungType {
    Integrierer,
    Differenzierer,
}

struct Schaltung {
    input_signal: Signal,
    output_signal: Signal,
}
