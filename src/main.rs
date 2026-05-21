// Einstiegspunkt der Anwendung
// Diese Datei lädt alle Module und mountet die Leptos-App zum DOM

mod app;
mod circuit_data;
mod circuits;
mod errors;
mod plot;
mod signals;

use leptos::mount::mount_to_body;

// Hauptfunktion: Startet die Leptos-Anwendung
fn main() {
    mount_to_body(app::App)
}
