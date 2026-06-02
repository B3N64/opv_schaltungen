use std::f64::consts::PI;
use std::ops::Add;

enum SignalType {
    Constant,
    Sinus,
    Cosinus,
    Rectangular,
    Triangular,
}

impl SignalType {
    fn all() -> &'static [SignalType] {
        &[
            SignalType::Constant,
            SignalType::Sinus,
            SignalType::Cosinus,
            SignalType::Rectangular,
            SignalType::Triangular,
        ]
    }

    fn from_id(&self, name: &str) -> Option<SignalType> {
        match name.to_lowercase().as_str() {
            "constant" => Some(SignalType::Constant),
            "sinus" => Some(SignalType::Sinus),
            "cosinus" => Some(SignalType::Cosinus),
            "rectangular" => Some(SignalType::Rectangular),
            "triangular" => Some(SignalType::Triangular),
            _ => None,
        }
    }

    fn id(&self) -> &'static str {
        match self {
            SignalType::Constant => "constant",
            SignalType::Sinus => "sinus",
            SignalType::Cosinus => "cosinus",
            SignalType::Rectangular => "rectangular",
            SignalType::Triangular => "triangular",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            SignalType::Constant => "Konstant",
            SignalType::Sinus => "Sinus",
            SignalType::Cosinus => "Cosinus",
            SignalType::Rectangular => "Rechteck",
            SignalType::Triangular => "Dreieck",
        }
    }

    fn variables(&self) -> &'static [&'static str] {
        match self {
            SignalType::Constant => &["Value"],
            SignalType::Sinus
            | SignalType::Cosinus
            | SignalType::Rectangular
            | SignalType::Triangular => &["Amplitude", "Frequency", "Phase"],
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
