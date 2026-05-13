use std::f64::consts::PI;

pub trait Signal {
    fn value_at(param: &SignalParams, t: f64) -> f64;
}
pub struct SignalParams {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
}

pub struct Sinus;

impl Signal for Sinus {
    fn value_at(param: &SignalParams, t: f64) -> f64 {
        param.amplitude * (2.0 * PI * param.frequency * t + param.phase).sin()
    }
}

pub struct Cosinus;

impl Signal for Cosinus {
    fn value_at(param: &SignalParams, t: f64) -> f64 {
        param.amplitude * (2.0 * PI * param.frequency * t + param.phase).cos()
    }
}

pub struct Rectangular;

impl Signal for Rectangular {
    fn value_at(param: &SignalParams, t: f64) -> f64 {
        if (2.0 * PI * param.frequency * t + param.phase).sin() >= 0.0 {
            param.amplitude
        } else {
            -param.amplitude
        }
    }
}

pub struct Triangle;

impl Signal for Triangle {
    fn value_at(param: &SignalParams, t: f64) -> f64 {
        let period = 1.0 / param.frequency;
        let x = ((t + param.phase) / period) % 1.0;

        if x < 0.25 {
            4.0 * param.amplitude * x
        } else if x < 0.75 {
            2.0 * param.amplitude - 4.0 * param.amplitude * x
        } else {
            -4.0 * param.amplitude + 4.0 * param.amplitude * x
        }
    }
}
