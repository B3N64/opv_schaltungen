use crate::errors::{Error, Result};
use crate::signals::Signal;
use std::f64::consts::PI;
use std::ops::Add;

enum CircuitType {
    Integrator,
    Differentiator,
    Tiefpass,
}

impl CircuitType {
    fn id(&self) -> &'static str {
        match self {
            CircuitType::Integrator => "Integrator",
            CircuitType::Differentiator => "Differentiator",
            CircuitType::Tiefpass => "Tiefpass",
        }
    }
    fn name(&self, name: &str) -> Option<CircuitType> {
        match name.to_lowercase().as_str() {
            "integrator" => Some(CircuitType::Integrator),
            "differentiator" => Some(CircuitType::Differentiator),
            "tiefpass" => Some(CircuitType::Tiefpass),
            _ => None,
        }
    }
    fn image(&self) -> &'static str {
        match self {
            CircuitType::Integrator => "/images/integrator.png",
            CircuitType::Differentiator => "/images/differentiator.png",
            CircuitType::Tiefpass => "/images/tiefpass.png",
        }
    }
    fn variables(&self) -> &'static [&'static str] {
        match self {
            CircuitType::Integrator => &["R", "C"],
            CircuitType::Differentiator => &["R", "C"],
            CircuitType::Tiefpass => &["R1", "CK", "RK"],
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
        }
    }
}

pub trait Circuit {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64;
    fn cutoff_frequency(&self) -> f64;
    fn amplitude_at(&self, frequenz: f64) -> f64;
    fn phase_at(&self, frequenz: f64) -> f64;
    fn simulate(&mut self, signal: &dyn Signal, duration: f64, step: f64) -> Vec<(f64, f64)> {
        let num_steps = (duration / step) as usize;
        let mut results = Vec::with_capacity(num_steps);

        for i in 0..num_steps {
            let t = i as f64 * step;
            let ue = signal.value_at(t);
            let ua = self.output_voltage(ue, step);
            results.push((t, ua));
        }

        let test = results[1];
        results[0] = test;

        results
    }
}

pub struct Tiefpass {
    r1: f64,
    ck: f64,
    rk: f64,
    last_ue: f64,
}

impl Tiefpass {
    pub fn new(r1: f64, ck: f64, rk: f64) -> Self {
        Self {
            r1,
            ck,
            rk,
            last_ue: 0.0,
        }
    }
}

impl Circuit for Tiefpass {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        // U_out(t) = (1/(R1*CK)) * ∫ (U_in(t) - U_out(t)) dt
        // ua[n] = ua[n−1] + (Δt/(R1*CK)) * (ue[n] - ua[n−1])
        let ua = self.last_ue + (dt / (self.r1 * self.ck)) * (ue - self.last_ue);
        self.last_ue = ua;
        ua
    }

    fn cutoff_frequency(&self) -> f64 {
        // |H(jw)| = 1/√(1+(wR1CK)^2) = 1/√2  =>  wR1CK = 1  =>  f = 1/(2πR1CK)
        1.0 / (2.0 * PI * self.r1 * self.ck)
    }

    fn amplitude_at(&self, frequenz: f64) -> f64 {
        // |H(jw)| = 1/√(1+(wR1CK)^2)
        let w = 2.0 * PI * frequenz;
        1.0 / (1.0 + (w * self.r1 * self.ck).powi(2)).sqrt()
    }

    fn phase_at(&self, frequenz: f64) -> f64 {
        // H(jw) = 1/(1+jwR1CK)  => Phase = -arctan(wR1CK)
        let w = 2.0 * PI * frequenz;
        -(w * self.r1 * self.ck).atan()
    }
}

pub struct Integrator {
    r: f64,
    c: f64,
    last_ua: f64,
}

impl Integrator {
    pub fn new(r: f64, c: f64) -> Self {
        Self { r, c, last_ua: 0.0 }
    }
}

impl Circuit for Integrator {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        // U_out(t) = -1/(R*C) * ∫ U_in(t) dt
        // ua[n] = ua[n−1] − 1/(R*C) * Δt * ue[n]
        let ua = self.last_ua - 1.0 / (self.r * self.c) * dt * ue;
        self.last_ua = ua;
        ua
    }

    fn cutoff_frequency(&self) -> f64 {
        // |H(jw)| = 1/(wRC) = 1  =>  w = 1/(RC)  =>  f = 1/(2πRC)
        1.0 / (2.0 * PI * self.r * self.c)
    }

    fn amplitude_at(&self, frequenz: f64) -> f64 {
        // |H(jw)| = 1/(ωRC)
        let f = frequenz.max(f64::MIN_POSITIVE);
        1.0 / (2.0 * PI * f * self.r * self.c)
    }

    fn phase_at(&self, _frequenz: f64) -> f64 {
        // H(jw) = -1/(jωRC) = +j/(ωRC)  => Phase = +90°
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
        Self { r, c, last_ue: 0.0 }
    }
}

impl Circuit for Differentiator {
    fn output_voltage(&mut self, ue: f64, dt: f64) -> f64 {
        // U_out(t) = -R*C * dU_in(t)/dt
        // ua[n] = -R*C * (ue[n] - ue[n−1]) / Δt
        let ua = -self.r * self.c * (ue - self.last_ue) / dt;
        self.last_ue = ue;
        ua
    }

    fn cutoff_frequency(&self) -> f64 {
        // |H(jw)| = ωRC = 1  =>  f = 1/(2πRC)
        1.0 / (2.0 * PI * self.r * self.c)
    }

    fn amplitude_at(&self, frequenz: f64) -> f64 {
        // |H(jw)| = ωRC
        2.0 * PI * frequenz * self.r * self.c
    }

    fn phase_at(&self, _frequenz: f64) -> f64 {
        // H(jw) = -jωRC  => Phase = -90°
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
