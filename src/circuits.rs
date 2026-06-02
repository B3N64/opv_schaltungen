use crate::errors::{Error, Result};
use std::f64::consts::PI;
use std::ops::Add;

enum CircuitType {
    Integrator,
    Differentiator,
    Tiefpass,
    Hochpass,
}

impl CircuitType {
    fn all() -> &'static [CircuitType] {
        &[
            CircuitType::Integrator,
            CircuitType::Differentiator,
            CircuitType::Tiefpass,
            CircuitType::Hochpass,
        ]
    }

    fn from_id(&self, name: &str) -> Option<CircuitType> {
        match name.to_lowercase().as_str() {
            "integrator" => Some(CircuitType::Integrator),
            "differentiator" => Some(CircuitType::Differentiator),
            "tiefpass" => Some(CircuitType::Tiefpass),
            "hochpass" => Some(CircuitType::Hochpass),
            _ => None,
        }
    }

    fn id(&self) -> &'static str {
        match self {
            CircuitType::Integrator => "integrator",
            CircuitType::Differentiator => "differentiator",
            CircuitType::Tiefpass => "tiefpass",
            CircuitType::Hochpass => "hochpass",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            CircuitType::Integrator => "Integrator",
            CircuitType::Differentiator => "Differentiator",
            CircuitType::Tiefpass => "Tiefpass",
            CircuitType::Hochpass => "Hochpass",
        }
    }

    fn image(&self) -> &'static str {
        match self {
            CircuitType::Integrator => "/images/integrator.png",
            CircuitType::Differentiator => "/images/differentiator.png",
            CircuitType::Tiefpass => "/images/tiefpass.png",
            CircuitType::Hochpass => "/images/hochpass.png",
        }
    }

    fn variables(&self) -> &'static [&'static str] {
        match self {
            CircuitType::Integrator => &["R", "C"],
            CircuitType::Differentiator => &["R", "C"],
            CircuitType::Tiefpass => &["R1", "CK", "RK"],
            CircuitType::Hochpass => &["R1", "C1", "RK"],
        }
    }

    pub fn construct(&self, values: &[f64]) -> Result<Box<dyn Circuit>> {
        let require = |idx: usize, key: &'static str| {
            values
                .get(idx)
                .copied()
                .ok_or(Error::CircuitConstructError(key.to_string()))
        };

        match self {
            Self::Integrator => Ok(Box::new(Integrator::new(
                require(0, "R")?,
                require(1, "C")?,
            ))),
            Self::Differentiator => Ok(Box::new(Differentiator::new(
                require(0, "R")?,
                require(1, "C")?,
            ))),
            Self::Tiefpass => Ok(Box::new(Tiefpass::new(
                require(0, "R1")?,
                require(1, "CK")?,
                require(2, "RK")?,
            ))),
            Self::Hochpass => Ok(Box::new(Hochpass::new(
                require(0, "R1")?,
                require(1, "C1")?,
                require(2, "RK")?,
            ))),
        }
    }
}

pub trait Circuit {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64;
    fn cutoff_frequency(&self) -> f64;
    fn amplitude_at(&self, frequenz: f64) -> f64;
    fn phase_at(&self, frequenz: f64) -> f64;
}

pub struct Hochpass {
    r1: f64,
    rk: f64,
    c1: f64,
    uc: f64,
}

impl Hochpass {
    pub fn new(r1: f64, rk: f64, c1: f64) -> Self {
        let r1 = r1.max(f64::MIN_POSITIVE);
        let c1 = c1.max(f64::MIN_POSITIVE);
        let rk = rk.max(f64::MIN_POSITIVE);

        Self {
            r1,
            rk,
            c1,
            uc: 0.0,
        }
    }
}

impl Circuit for Hochpass {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        let i_in = (ue - self.uc) / self.r1;
        let duc = (i_in / self.c1) * dt;

        self.uc += duc;
        -(self.rk / self.r1) * (ue - self.uc)
    }

    fn cutoff_frequency(&self) -> f64 {
        1.0 / (2.0 * PI * self.r1 * self.c1)
    }

    fn amplitude_at(&self, frequenz: f64) -> f64 {
        let f = frequenz.max(f64::MIN_POSITIVE);
        let omega = 2.0 * PI * f;

        (omega * self.c1 * self.rk) / (1.0 + (omega * self.r1 * self.c1).powi(2)).sqrt()
    }

    fn phase_at(&self, frequenz: f64) -> f64 {
        let f = frequenz.max(f64::MIN_POSITIVE);
        let omega = 2.0 * PI * f;

        -PI / 2.0 - (omega * self.r1 * self.c1).atan()
    }
}

pub struct PDGlied {
    r1: f64,
    rk: f64,
    c1: f64,
    last_ue: f64, // Zustandsspeicher: Vorherige Eingangsspannung für die Ableitung
}

impl PDGlied {
    pub fn new(r1: f64, rk: f64, c1: f64) -> Self {
        Self {
            r1,
            rk,
            c1,
            last_ue: 0.0,
        }
    }
}

impl Circuit for PDGlied {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        let due_dt = if dt > 0.0 {
            (ue - self.last_ue) / dt
        } else {
            0.0
        };

        self.last_ue = ue;

        let proportional_part = (self.rk / self.r1) * ue;
        let derivative_part = self.rk * self.c1 * due_dt;

        -(proportional_part + derivative_part)
    }

    fn cutoff_frequency(&self) -> f64 {
        1.0 / (2.0 * PI * self.r1 * self.c1)
    }

    fn amplitude_at(&self, frequenz: f64) -> f64 {
        let f = frequenz.max(f64::MIN_POSITIVE);
        let omega = 2.0 * PI * f;
        let p_gain = self.rk / self.r1;

        p_gain * (1.0 + (omega * self.r1 * self.c1).powi(2)).sqrt()
    }

    fn phase_at(&self, frequenz: f64) -> f64 {
        let f = frequenz.max(f64::MIN_POSITIVE);
        let omega = 2.0 * PI * f;

        PI + (omega * self.r1 * self.c1).atan()
    }
}

pub struct Tiefpass {
    r1: f64,
    ck: f64,
    rk: f64,
    last_ua: f64,
}

impl Tiefpass {
    pub fn new(r1: f64, ck: f64, rk: f64) -> Self {
        let r1 = r1.max(f64::MIN_POSITIVE);
        let ck = ck.max(f64::MIN_POSITIVE);
        let rk = rk.max(f64::MIN_POSITIVE);

        Self {
            r1,
            ck,
            rk,
            last_ua: 0.0,
        }
    }
}

impl Circuit for Tiefpass {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        let derivative = -self.last_ua / (self.rk * self.ck) - ue / (self.r1 * self.ck);
        let ua = self.last_ua + derivative * dt;

        self.last_ua = ua;
        ua
    }

    fn cutoff_frequency(&self) -> f64 {
        1.0 / (2.0 * PI * self.rk * self.ck)
    }

    fn amplitude_at(&self, frequenz: f64) -> f64 {
        let f = frequenz.max(f64::MIN_POSITIVE);
        let omega = 2.0 * PI * f;
        let dc_gain = self.rk / self.r1;

        dc_gain / (1.0 + (omega * self.rk * self.ck).powi(2)).sqrt()
    }

    fn phase_at(&self, frequenz: f64) -> f64 {
        let f = frequenz.max(f64::MIN_POSITIVE);
        let omega = 2.0 * PI * f;

        PI - (omega * self.rk * self.ck).atan()
    }
}

pub struct Integrator {
    r: f64,
    c: f64,
    last_ua: f64,
}

impl Integrator {
    pub fn new(r: f64, c: f64) -> Self {
        let r = r.max(f64::MIN_POSITIVE);
        let c = c.max(f64::MIN_POSITIVE);

        Self { r, c, last_ua: 0.0 }
    }
}

impl Circuit for Integrator {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        let ua = self.last_ua - 1.0 / (self.r * self.c) * dt * ue;
        self.last_ua = ua;
        ua
    }

    fn cutoff_frequency(&self) -> f64 {
        1.0 / (2.0 * PI * self.r * self.c)
    }

    fn amplitude_at(&self, frequenz: f64) -> f64 {
        let f = frequenz.max(f64::MIN_POSITIVE);
        1.0 / (2.0 * PI * f * self.r * self.c)
    }

    fn phase_at(&self, _frequenz: f64) -> f64 {
        PI / 2.0
    }
}

pub struct Differentiator {
    r: f64,
    c: f64,
    last_ue: f64,
}

impl Differentiator {
    pub fn new(r: f64, c: f64) -> Self {
        let r = r.max(f64::MIN_POSITIVE);
        let c = c.max(f64::MIN_POSITIVE);

        Self { r, c, last_ue: 0.0 }
    }
}

impl Circuit for Differentiator {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        let ua = -self.r * self.c * (ue - self.last_ue) / dt;
        self.last_ue = ue;
        ua
    }

    fn cutoff_frequency(&self) -> f64 {
        1.0 / (2.0 * PI * self.r * self.c)
    }

    fn amplitude_at(&self, frequenz: f64) -> f64 {
        let frequenz = frequenz.max(f64::MIN_POSITIVE);

        2.0 * PI * frequenz * self.r * self.c
    }

    fn phase_at(&self, _frequenz: f64) -> f64 {
        -PI / 2.0
    }
}

pub struct CombinedCircuit<'a> {
    circuit1: &'a mut dyn Circuit,
    circuit2: &'a mut dyn Circuit,
}

impl<'a> CombinedCircuit<'a> {
    pub fn new(circuit1: &'a mut dyn Circuit, circuit2: &'a mut dyn Circuit) -> Self {
        Self { circuit1, circuit2 }
    }
}

impl<'a> Circuit for CombinedCircuit<'a> {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        let u1 = self.circuit1.output_voltage(ue, dt);
        self.circuit2.output_voltage(u1, dt)
    }
    fn cutoff_frequency(&self) -> f64 {
        self.circuit1.cutoff_frequency() + self.circuit2.cutoff_frequency()
    }
    fn amplitude_at(&self, frequenz: f64) -> f64 {
        self.circuit1.amplitude_at(frequenz) * self.circuit2.amplitude_at(frequenz)
    }
    fn phase_at(&self, frequenz: f64) -> f64 {
        self.circuit1.phase_at(frequenz) + self.circuit2.phase_at(frequenz)
    }
}

impl<'a> Add for &'a mut dyn Circuit {
    type Output = CombinedCircuit<'a>;

    fn add(self, other: &'a mut dyn Circuit) -> CombinedCircuit<'a> {
        CombinedCircuit::new(self, other)
    }
}
