use crate::signals::Signal;
use std::f64::consts::PI;
use std::ops::Add;

enum CircuitType {
    Integrator,
    Differentiator,
}

pub trait Circuit {
    fn response(&mut self, ue: f64, dt: f64) -> f64;
    fn cutoff_frequency(&self) -> f64;
    fn amplitudengang(&self, frequenz: f64) -> f64;
    fn generate_time_response(
        &mut self,
        signal: &dyn Signal,
        duration: f64,
        step: f64,
    ) -> Vec<(f64, f64)> {
        let mut results = vec![];
        let num_steps = (duration / step) as usize;

        for i in 0..num_steps {
            let t = i as f64 * step;
            let ue = signal.value_at(t);
            let ua = self.response(ue, step);
            results.push((t, ua));
        }

        results
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
    fn response(&mut self, ue: f64, dt: f64) -> f64 {
        // U_out(t) = -1/(R*C) * ∫ U_in(t) dt
        // ua[n] = ua[n−1] − 1/(R*C) * Δt * ue[n]
        let ua = self.last_ua - 1.0 / (self.r * self.c) * dt * ue;
        self.last_ua = ua;
        ua
    }
    fn cutoff_frequency(&self) -> f64 {
        1.0 / (2.0 * PI * self.r * self.c)
    }
    fn amplitudengang(&self, frequenz: f64) -> f64 {
        1.0 / (1.0 + (2.0 * PI * frequenz * self.r * self.c).powi(2)).sqrt()
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
    fn response(&mut self, ue: f64, dt: f64) -> f64 {
        // U_out(t) = -R*C * dU_in(t)/dt
        // ua[n] = -R*C * (ue[n] - ue[n−1]) / Δt
        let ua = -self.r * self.c * (ue - self.last_ue) / dt;
        self.last_ue = ue;
        ua
    }
    fn cutoff_frequency(&self) -> f64 {
        1.0 / (2.0 * PI * self.r * self.c)
    }
    fn amplitudengang(&self, frequenz: f64) -> f64 {
        2.0 * PI * frequenz * self.r * self.c
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
    fn response(&mut self, ue: f64, dt: f64) -> f64 {
        self.circuit2.response(ue, dt) + self.circuit2.response(ue, dt)
    }
    fn cutoff_frequency(&self) -> f64 {
        self.circuit1.cutoff_frequency() + self.circuit2.cutoff_frequency()
    }
    fn amplitudengang(&self, frequenz: f64) -> f64 {
        self.circuit1.amplitudengang(frequenz) * self.circuit2.amplitudengang(frequenz)
    }
}

impl<'a> Add for &'a mut dyn Circuit {
    type Output = CombinedCircuit<'a>;

    fn add(self, other: &'a mut dyn Circuit) -> CombinedCircuit<'a> {
        CombinedCircuit::new(self, other)
    }
}
