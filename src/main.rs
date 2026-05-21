// Einstiegspunkt der Anwendung
// Diese Datei lädt alle Module und mountet die Leptos-App zum DOM

mod circuit_data;  // Modul für die Verwaltung von OPV-Schaltungsdaten
mod errors;        // Modul für Fehlerbehandlung
mod signals;       // Modul für reaktive Signale
mod app;           // Hauptmodul für die UI und Routing

use leptos::mount::mount_to_body;

// Hauptfunktion: Startet die Leptos-Anwendung
fn main() {
    mount_to_body(app::App)
}
