use std::f64::consts::PI;

pub trait Signal {
    fn value_at(&self, t: f64) -> f64;
}

pub struct SignalParams {
    pub amplitude: f64,
    pub frequency: f64,
    pub phase: f64,
}

pub struct Sinus {
    param: SignalParams,
}

impl Signal for Sinus {
    fn value_at(&self, t: f64) -> f64 {
        self.param.amplitude * (2.0 * PI * self.param.frequency * t + self.param.phase).sin()
    }
}

pub struct Cosinus {
    param: SignalParams,
}

impl Signal for Cosinus {
    fn value_at(&self, t: f64) -> f64 {
        self.param.amplitude * (2.0 * PI * self.param.frequency * t + self.param.phase).cos()
    }
}

pub struct Rectangular {
    param: SignalParams,
}

impl Signal for Rectangular {
    fn value_at(&self, t: f64) -> f64 {
        if (2.0 * PI * self.param.frequency * t + self.param.phase).sin() >= 0.0 {
            self.param.amplitude
        } else {
            -self.param.amplitude
        }
    }
}

pub struct Triangle {
    param: SignalParams,
}

impl Signal for Triangle {
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
}
