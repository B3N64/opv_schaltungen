// Einstiegspunkt der Anwendung
// Diese Datei lädt alle Module und mountet die Leptos-App zum DOM

mod errors;   // Modul für Fehlerbehandlung
mod signals;  // Modul für Eingangssignale
mod app;      // Hauptmodul für die UI und Routing
mod circuits; // Modul für OPV-Schaltungen
mod plot;     // Modul für Diagramme


use leptos::mount::mount_to_body;

// Hauptfunktion: Startet die Leptos-Anwendung
fn main() {
    mount_to_body(app::App)
}
