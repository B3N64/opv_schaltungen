// Hauptanwendung für OPV-Schaltungen
// Diese Datei enthält alle UI-Komponenten und das Routing

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_params_map;
use leptos_router::path;

use crate::circuit_data::{
    find_circuit_info,
    CircuitInfo,
    CIRCUIT_INFOS,
};

// Hauptkomponente: Definiert die Routing-Struktur der Anwendung
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main class="page">
                <Routes fallback=|| view! { <h1>"Seite nicht gefunden"</h1> }>
                    <Route path=path!("/") view=HomePage />              {/* Startseite mit Schaltungsübersicht */}
                    <Route path=path!("/circuit/:id") view=CircuitPage /> {/* Detailseite für einzelne Schaltung */}
                </Routes>
            </main>
        </Router>
    }
}

// Startseite: Zeigt alle verfügbaren OPV-Schaltungen als Karten an
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <section>
            <h1>"OPV-Schaltungen"</h1>
            <p class="subtitle">
                "Wähle eine Schaltung aus, um die zugehörigen Variablen einzutragen."
            </p>

            {/* Grid mit Schaltungskarten */}
            <div class="circuit-grid">
                {CIRCUIT_INFOS
                    .iter()
                    .map(|circuit| view! {
                        <CircuitCard circuit=*circuit />
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

// Einzelne Schaltungskarte als Link zur Detailseite
#[component]
fn CircuitCard(circuit: CircuitInfo) -> impl IntoView {
    view! {
        <A href=format!("/circuit/{}", circuit.id) attr:class="circuit-card">
            <img src=circuit.image alt=circuit.name />
            <h2>{circuit.name}</h2>
        </A>
    }
}

// Detailseite: Zeigt eine einzelne Schaltung mit Eingabeformular für Variablen
#[component]
fn CircuitPage() -> impl IntoView {
    let params = use_params_map();

    // Schaltung anhand der URL-Parameter laden
    let circuit = move || {
        params
            .read()
            .get("id")
            .and_then(|id| find_circuit_info(&id))
    };

    view! {
        <section>
            <A href="/" attr:class="back-link">"← Zurück zur Übersicht"</A>

            {/* Bedingte Anzeige: Schaltungsinformationen oder Fehlermeldung */}
            {move || match circuit() {
                Some(circuit) => view! {
                    <div class="detail-layout">
                        {/* Linkes Panel: Schaltungsbild */}
                        <div class="image-panel">
                            <h1>{circuit.name}</h1>
                            <img src=circuit.image alt=circuit.name />
                        </div>

                        {/* Rechtes Panel: Eingabeformular für Variablen */}
                        <div class="input-panel">
                            <h2>"Variablen"</h2>

                            <form>
                                {/* Eingabefelder für alle Variablen generieren */}
                                {circuit.variables
                                    .iter()
                                    .map(|variable| view! {
                                        <label class="input-row">
                                            <span>{*variable}</span>
                                            <input
                                                type="number"
                                                placeholder=format!("{} eingeben", variable)
                                            />
                                        </label>
                                    })
                                    .collect_view()}

                                {/* Berechnen-Button */}
                                <button type="button">
                                    "Berechnen"
                                </button>
                            </form>
                        </div>
                    </div>
                }.into_any(),

                None => view! {
                    <h1>"Schaltung nicht gefunden"</h1>
                }.into_any(),
            }}
        </section>
    }
}