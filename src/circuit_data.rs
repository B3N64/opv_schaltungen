// Datenverwaltung für OPV-Schaltungen
// Definiert die verfügbaren Schaltungen und deren Eigenschaften

use crate::circuits::{CircuitCalculation, CircuitInput, calculate_circuit};
use crate::errors::{Error, Result};

#[derive(Clone, Copy)]
pub enum CircuitKind {
    Integrator,
    Differentiator,
}

#[derive(Clone, Copy)]
pub enum VariableKey {
    Resistance,
    Capacitance,
    InputVoltage,
    TimeStep,
}

#[derive(Clone, Copy)]
pub struct VariableInfo {
    pub key: VariableKey,
    pub label: &'static str,
    pub placeholder: &'static str,
}

// Struktur für Informationen über eine OPV-Schaltung
#[derive(Clone, Copy)]
pub struct CircuitInfo {
    pub id: &'static str,                      // Eindeutige Kennung der Schaltung
    pub name: &'static str,                    // Name der Schaltung (z.B. "Integrator")
    pub image: &'static str,                   // Pfad zum Schaltungsbild
    pub kind: CircuitKind,                     // Verknüpfung zur Berechnungslogik
    pub variables: &'static [VariableInfo],    // Liste der Eingabevariablen
}

impl CircuitInfo {
    pub fn parse_input_values(&self, raw_values: &[String]) -> Result<CircuitInput> {
        if raw_values.len() != self.variables.len() {
            return Err(Error::MissingInput("den benoetigten Eingaben"));
        }

        let mut r = None;
        let mut c = None;
        let mut ue = None;
        let mut dt = None;

        for (variable, raw_value) in self.variables.iter().zip(raw_values.iter()) {
            let trimmed = raw_value.trim();
            if trimmed.is_empty() {
                return Err(Error::MissingInput(variable.label));
            }

            let value = trimmed
                .parse::<f64>()
                .map_err(|_| Error::InvalidNumber(variable.label))?;

            match variable.key {
                VariableKey::Resistance => r = Some(value),
                VariableKey::Capacitance => c = Some(value),
                VariableKey::InputVoltage => ue = Some(value),
                VariableKey::TimeStep => dt = Some(value),
            }
        }

        Ok(CircuitInput {
            r: r.ok_or(Error::MissingInput("R"))?,
            c: c.ok_or(Error::MissingInput("C"))?,
            ue: ue.ok_or(Error::MissingInput("U_e"))?,
            dt: dt.ok_or(Error::MissingInput("dt"))?,
        })
    }

    pub fn calculate(&self, raw_values: &[String]) -> Result<CircuitCalculation> {
        let input = self.parse_input_values(raw_values)?;
        calculate_circuit(self.kind, &input)
    }
}

const COMMON_VARIABLES: &[VariableInfo] = &[
    VariableInfo {
        key: VariableKey::Resistance,
        label: "R",
        placeholder: "Widerstand eingeben",
    },
    VariableInfo {
        key: VariableKey::Capacitance,
        label: "C",
        placeholder: "Kapazitaet eingeben",
    },
    VariableInfo {
        key: VariableKey::InputVoltage,
        label: "U_e",
        placeholder: "Eingangsspannung eingeben",
    },
    VariableInfo {
        key: VariableKey::TimeStep,
        label: "dt",
        placeholder: "Zeitschritt eingeben",
    },
];

// Alle verfügbaren OPV-Schaltungen
pub const CIRCUIT_INFOS: &[CircuitInfo] = &[
    // Integratorschaltung mit Widerstand, Kapazität, Eingangsspannung und Zeitintervall
    CircuitInfo {
        id: "integrator",
        name: "Integrator",
        image: "/public/images/integrator.png",
        kind: CircuitKind::Integrator,
        variables: COMMON_VARIABLES,
    },
    // Differentiatorschaltung mit ähnlichen Variablen
    CircuitInfo {
        id: "differentiator",
        name: "Differentiator",
        image: "/public/images/differentiator.png",
        kind: CircuitKind::Differentiator,
        variables: COMMON_VARIABLES,
    },
];

// Hilfsfunktion: Findet eine Schaltung anhand ihrer ID
pub fn find_circuit_info(id: &str) -> Option<CircuitInfo> {
    CIRCUIT_INFOS.iter().copied().find(|c| c.id == id)
}
