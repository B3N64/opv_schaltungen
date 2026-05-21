// Hauptanwendung für OPV-Schaltungen
// Diese Datei enthält alle UI-Komponenten und das Routing

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::{use_params_map, use_navigate};
use leptos_router::path;

use crate::circuits::CircuitCalculation;
use crate::circuit_data::{
    find_circuit_info,
    CircuitInfo,
    CIRCUIT_INFOS,
};

// Hauptkomponente: Definiert die Routing-Struktur der Anwendung
//
// In Leptos wird hier keine eigene "goto_page"-Funktion gebraucht.
// Stattdessen übernimmt der Router die Navigation:
// - <Router> erzeugt den Client-Router und beobachtet die URL
// - <Routes> definiert, welche Seite bei welcher URL angezeigt wird
// - <A href=...> erzeugt einen internen Link, der die URL ändert
//   ohne die Seite neu zu laden
//
// Wenn ein Benutzer auf eine Schaltung klickt, wird die URL z.B. zu
// "circuit/integrator" geändert. Der Router erkennt die neue Route und
// rendert dann die entsprechende Komponente.
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main class="page">
                <Routes fallback=|| view! { <h1>"Seite nicht gefunden"</h1> }>
                    <Route path=path!("/") view=HomePage />              
                    <Route path=path!("/circuit/:id") view=CircuitPage /> 
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
//
// NEUE VERSION: Diese Komponente verwendet jetzt `use_navigate()` für
// PROGRAMMATISCHE NAVIGATION statt eines einfachen <A> Links.
//
// use_navigate() gibt eine Funktion zurück, die die App zu einer
// neuen Route navigiert. Damit können wir:
// 1. Auf Click reagieren
// 2. Vor der Navigation etwas überprüfen oder verarbeiten
// 3. Beliebig viel Code auslösen, bevor wir navigieren
//
// In diesem Beispiel ist der onClick-Handler die Navigations-Funktion.
#[component]
fn CircuitCard(circuit: CircuitInfo) -> impl IntoView {
    // use_navigate() gibt uns eine Funktion, mit der wir programmatisch
    // zu anderen Routen springen können
    let navigate = use_navigate();

    // onClick-Handler: Beim Klick auf die Karte wird diese Funktion aufgerufen
    let on_click = move |_| {
        // navigate() ändert die URL zur Detailseite dieser Schaltung
        // Die URL wird zu: "/circuit/integrator" oder "/circuit/differentiator" etc.
        navigate(&format!("/circuit/{}", circuit.id), Default::default());
    };

    view! {
        <div class="circuit-card" on:click=on_click>
            <img src=circuit.image alt=circuit.name />
            <h2>{circuit.name}</h2>
        </div>
    }
}

// Hilfskomponente für den Back-Link mit programmatischer Navigation
//
// Diese Komponente zeigt den "Zurück"-Button und navigiert zur Startseite.
// Sie benutzt `use_navigate()` um beim Klick die Route zu wechseln.
#[component]
fn BackLink() -> impl IntoView {
    let navigate = use_navigate();
    let on_click = move |_| {
        // Navigiere zur Startseite zurück
        navigate("/", Default::default());
    };

    view! {
        <div on:click=on_click class="back-link">
            "← Zurück zur Übersicht"
        </div>
    }
}

#[component]
fn CircuitDetail(circuit: CircuitInfo) -> impl IntoView {
    let input_values = RwSignal::new(vec![String::new(); circuit.variables.len()]);
    let calculation = RwSignal::new(None::<CircuitCalculation>);
    let error_message = RwSignal::new(None::<String>);

    let on_calculate = move |_| {
        let raw_values = input_values.get();

        match circuit.calculate(&raw_values) {
            Ok(result) => {
                calculation.set(Some(result));
                error_message.set(None);
            }
            Err(error) => {
                calculation.set(None);
                error_message.set(Some(error.to_string()));
            }
        }
    };

    view! {
        <div class="detail-layout">
            <div class="image-panel">
                <h1>{circuit.name}</h1>
                <img src=circuit.image alt=circuit.name />
            </div>

            <div class="input-panel">
                <h2>"Variablen"</h2>

                <form on:submit=|ev| ev.prevent_default()>
                    {circuit.variables
                        .iter()
                        .enumerate()
                        .map(|(index, variable)| view! {
                            <label class="input-row">
                                <span>{variable.label}</span>
                                <input
                                    type="number"
                                    step="any"
                                    placeholder=variable.placeholder
                                    prop:value=move || input_values.with(|values| values[index].clone())
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        input_values.update(|values| values[index] = value);
                                    }
                                />
                            </label>
                        })
                        .collect_view()}

                    <button type="button" on:click=on_calculate>
                        "Berechnen"
                    </button>
                </form>

                {move || error_message.get().map(|message| view! {
                    <p class="status-message error-message">{message}</p>
                })}

                {move || calculation.get().map(|result| view! {
                    <div class="result-panel">
                        <h3>"Ergebnis"</h3>
                        <p class="result-row">
                            <strong>"U_a"</strong>
                            <span>{format!("{:.6} V", result.output_voltage)}</span>
                        </p>
                        <p class="result-row">
                            <strong>"f_g"</strong>
                            <span>{format!("{:.6} Hz", result.cutoff_frequency)}</span>
                        </p>
                    </div>
                })}
            </div>
        </div>
    }
}

// Detailseite: Zeigt eine einzelne Schaltung mit Eingabeformular für Variablen
//
// Diese Seite wird durch die Route `circuit/:id` ausgewählt.
// Der Platzhalter `:id` in der Route wird zur Laufzeit aus der aktuellen
// URL extrahiert. Hier lesen wir diesen Wert mit `use_params_map()`.
#[component]
fn CircuitPage() -> impl IntoView {
    let params = use_params_map();

    // Schaltung anhand der URL-Parameter laden
    // Beispiel: Bei URL "circuit/integrator" ist id = "integrator".
    let circuit = move || {
        params
            .read()
            .get("id")
            .and_then(|id| find_circuit_info(&id))
    };

    view! {
        <section>
            <BackLink />

            {/* Bedingte Anzeige: Schaltungsinformationen oder Fehlermeldung */}
            {move || match circuit() {
                Some(circuit) => view! { <CircuitDetail circuit /> }.into_any(),

                None => view! {
                    <h1>"Schaltung nicht gefunden"</h1>
                }.into_any(),
            }}
        </section>
    }
}
