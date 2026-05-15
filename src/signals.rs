use std::f64::consts::PI;
use std::ops::Add;

pub trait Signal {
    fn value_at(&self, t: f64) -> f64;
    fn generate(&self, duration: f64, step: f64) -> Vec<(f64, f64)> {
        let mut results = vec![];
        let num_steps = (duration / step) as usize;

        for i in 0..num_steps {
            let t = i as f64 * step;
            results.push((t, self.value_at(t)));
        }

        results
    }
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
}

pub struct Sinus {
    param: SignalParams,
}

impl Signal for Sinus {
    fn value_at(&self, t: f64) -> f64 {
        self.param.amplitude * (2.0 * PI * self.param.frequency * t + self.param.phase).sin()
    }
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
}

impl<'a> Add for &'a dyn Signal {
    type Output = CombinedSignal<'a>;

    fn add(self, other: &'a dyn Signal) -> CombinedSignal<'a> {
        CombinedSignal::new(self, other)
    }
}
