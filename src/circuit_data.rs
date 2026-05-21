// Datenverwaltung für OPV-Schaltungen
// Definiert die verfügbaren Schaltungen und deren Eigenschaften

// Struktur für Informationen über eine OPV-Schaltung
#[derive(Clone, Copy)]
pub struct CircuitInfo {
    pub id: &'static str,                      // Eindeutige Kennung der Schaltung
    pub name: &'static str,                    // Name der Schaltung (z.B. "Integrator")
    pub image: &'static str,                   // Pfad zum Schaltungsbild
    pub variables: &'static [&'static str],    // Liste der Eingabevariablen
}

// Alle verfügbaren OPV-Schaltungen
pub const CIRCUIT_INFOS: &[CircuitInfo] = &[
    // Integratorschaltung mit Widerstand, Kapazität, Eingangsspannung und Zeitintervall
    CircuitInfo {
        id: "integrator",
        name: "Integrator",
        image: "/images/integrator.png",
        variables: &["R", "C", "U_e", "dt"],
    },
    // Differentiatorschaltung mit ähnlichen Variablen
    CircuitInfo {
        id: "differentiator",
        name: "Differentiator",
        image: "/images/differentiator.png",
        variables: &["R", "C", "U_e", "dt"],
    },
];

// Hilfsfunktion: Findet eine Schaltung anhand ihrer ID
pub fn find_circuit_info(id: &str) -> Option<CircuitInfo> {
    CIRCUIT_INFOS.iter().copied().find(|c| c.id == id)
}