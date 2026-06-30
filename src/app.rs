// Hauptanwendung für OPV-Schaltungen
// Diese Datei enthält alle UI-Komponenten und das Routing

use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_params_map;
use leptos_router::path;
use std::collections::HashMap;

use crate::circuits::CircuitType;
use crate::plot::{draw_bode_diagram, draw_time_response};
use crate::signals::SignalType;

// Hauptkomponente: Definiert die Routing-Struktur der Anwendung
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router base="/opv_schaltungen">
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
                {CircuitType::all()
                .iter()
                .copied()
                .map(|circuit| view! {
                    <CircuitCard circuit=circuit />
                })
                .collect_view()}
            </div>
        </section>
    }
}

// Einzelne Schaltungskarte als Link zur Detailseite
#[component]
fn CircuitCard(circuit: CircuitType) -> impl IntoView {
    view! {
        <A href=format!("circuit/{}", circuit.id())>
            <div class="circuit-card">
                <img src=circuit.image() alt=circuit.name() />
                <h2>{circuit.name()}</h2>
            </div>
        </A>
    }
}

// Detailseite: Zeigt eine einzelne Schaltung mit Eingabeformular für Variablen
#[derive(Clone)]
struct SimulationPreview {
    cutoff_frequency: f64,
    input_amplitude: f64,
    output_amplitude: f64,
}

fn parse_values_for_variables(
    values: &HashMap<String, String>,
    variables: &[&str],
) -> std::result::Result<Vec<f64>, String> {
    variables
        .iter()
        .map(|variable| {
            let raw_value = values
                .get(*variable)
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .or_else(|| default_value_for_variable(*variable))
                .ok_or_else(|| format!("Fehlender Wert für {}", variable))?;

            raw_value
                .replace(',', ".")
                .parse::<f64>()
                .map_err(|_| format!("Ungültiger Zahlenwert für {}", variable))
        })
        .collect()
}

fn default_value_for_variable(variable: &str) -> Option<&'static str> {
    match variable {
        // Schaltungsparameter
        "R" | "R1" | "RK" => Some("1000"),
        "C" | "C1" | "CK" => Some("0,000001"),

        // Signalparameter
        "Value" => Some("1"),
        "Amplitude" => Some("1"),
        "Frequency" => Some("100"),
        "Phase" => Some("0"),

        _ => None,
    }
}

fn calculate_voltage_amplitudes(
    signal: &dyn crate::signals::Signal,
    circuit: &mut dyn crate::circuits::Circuit,
) -> (f64, f64) {
    let frequency = signal.frequency().abs();

    let duration = if frequency > 0.0 {
        2.0 / frequency
    } else {
        1.0
    };

    let samples = 1000;
    let dt = duration / samples as f64;

    let mut input_amplitude = 0.0;
    let mut output_amplitude = 0.0;

    for i in 0..=samples {
        let t = i as f64 * dt;
        let ue = signal.value_at(t);
        let ua = circuit.output_voltage(ue, dt);

        if ue.abs() > input_amplitude {
            input_amplitude = ue.abs();
        }

        if ua.abs() > output_amplitude {
            output_amplitude = ua.abs();
        }
    }

    (input_amplitude, output_amplitude)
}

#[component]
fn CircuitPage() -> impl IntoView {
    let params = use_params_map();

    // Schaltung anhand der URL-Parameter laden
    let circuit = move || {
        params
            .read()
            .get("id")
            .and_then(|id| CircuitType::from_id(&id))
    };

    let (selected_signal_id, set_selected_signal_id) =
        signal(SignalType::Constant.id().to_string());

    let selected_signal =
        move || SignalType::from_id(&selected_signal_id.get()).unwrap_or(SignalType::Constant);

    let (circuit_values, set_circuit_values) = signal(HashMap::<String, String>::new());

    let (signal_values, set_signal_values) = signal(HashMap::<String, String>::new());

    let (simulation_preview, set_simulation_preview) = signal(Option::<SimulationPreview>::None);

    let (calculation_error, set_calculation_error) = signal(Option::<String>::None);

    let time_canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let bode_canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    view! {
        <section>
            <A href="/opv_schaltungen/" attr:class="back-link">"← Zurück zur Übersicht"</A>

                {/* Bedingte Anzeige: Schaltungsinformationen oder Fehlermeldung */}
                {move || match circuit() {
                    Some(circuit) => view! {
                        <div class="simulation-page">
                                <div class="detail-layout">
                                    {/* Bereich A: Schaltungsbild */}
                                    <div class="image-panel">
                                        <h1>{circuit.name()}</h1>
                                        <img src=circuit.image() alt=circuit.name() />
                                    </div>

                                    {/* Bereiche B, C und D: Eingaben, Signal und berechnete Werte */}
                                    <div class="input-panel">
                                <h2>"Variablen"</h2>

                                <form>
                                    {/* Eingabefelder für alle Variablen generieren */}
                                    {circuit.variables()
                                    .iter()
                                    .map(|variable| {
                                        let variable_name = variable.to_string();
                                        let variable_name_for_value = variable_name.clone();
                                        let variable_name_for_input = variable_name.clone();

                                        view! {
                                            <label class="input-row">
                                                <span>{*variable}</span>
                                                <input
                                                    type="text"
                                                    inputmode="decimal"
                                                    placeholder=format!("{} eingeben", variable)
                                                    prop:value=move || {
                                                        circuit_values
                                                            .get()
                                                            .get(&variable_name_for_value)
                                                            .cloned()
                                                            .unwrap_or_else(|| {
                                                                default_value_for_variable(&variable_name_for_value)
                                                                    .unwrap_or("")
                                                                    .to_string()
                                                            })
                                                    }
                                                    on:input=move |ev| {
                                                        let value = event_target_value(&ev);

                                                        set_circuit_values.update(|values| {
                                                            values.insert(variable_name_for_input.clone(), value);
                                                        });
                                                    }
                                                />
                                            </label>
                                        }
                                    })
                                    .collect_view()}


                                    <h2>"Eingangssignal"</h2>

                                    <label class="input-row">
                                        <span>"Signaltyp"</span>
                                        <select
                                            prop:value=selected_signal_id
                                            on:change=move |ev| {
                                            set_selected_signal_id.set(event_target_value(&ev));
                                            set_signal_values.set(HashMap::new());
                                            }
                                        >
                                            {SignalType::all()
                                                .iter()
                                                .copied()
                                                .map(|signal_type| view! {
                                                    <option value=signal_type.id()>
                                                        {signal_type.name()}
                                                    </option>
                                                })
                                                .collect_view()}
                                        </select>
                                    </label>

    {move || {
        selected_signal()
            .variables()
            .iter()
            .map(|variable| {
                let variable_name = variable.to_string();
                let variable_name_for_value = variable_name.clone();
                let variable_name_for_input = variable_name.clone();

                view! {
                    <label class="input-row">
                        <span>{*variable}</span>
                        <input
                            type="text"
                            inputmode="decimal"
                            placeholder=format!("{} eingeben", variable)
                            prop:value=move || {
                                signal_values
                                    .get()
                                    .get(&variable_name_for_value)
                                    .cloned()
                                    .unwrap_or_else(|| {
                                        default_value_for_variable(&variable_name_for_value)
                                            .unwrap_or("")
                                            .to_string()
                                    })
                            }
                            on:input=move |ev| {
                                let value = event_target_value(&ev);

                                set_signal_values.update(|values| {
                                    values.insert(variable_name_for_input.clone(), value);
                                });
                            }
                        />
                    </label>
                }
            })
            .collect_view()
    }}

                                    {/* Berechnen-Button */}
                                    <button
                                        type="button"
                                        on:click=move |_| {
                                            let circuit_result = parse_values_for_variables(
                                                &circuit_values.get(),
                                                circuit.variables(),
                                            );

                                            let signal_type = selected_signal();

                                            let signal_result = parse_values_for_variables(
                                                &signal_values.get(),
                                                signal_type.variables(),
                                            );

                                            match (circuit_result, signal_result) {
                                                (Ok(circuit_numbers), Ok(signal_numbers)) => {
                                                    let circuit_instance_result = circuit.construct(&circuit_numbers);
                                                    let signal_instance_result = signal_type.construct(&signal_numbers);

                                                    match (circuit_instance_result, signal_instance_result) {
                                                        (Ok(mut circuit_instance), Ok(signal_instance)) => {
                                                            let cutoff_frequency = circuit_instance.cutoff_frequency();
                                                            let (input_amplitude, output_amplitude) =
                                                                calculate_voltage_amplitudes(signal_instance.as_ref(), circuit_instance.as_mut());

                                                            set_simulation_preview.set(Some(SimulationPreview {
                                                                cutoff_frequency,
                                                                input_amplitude,
                                                                output_amplitude,
                                                            }));

                                                            set_calculation_error.set(None);

                                                            if let (Some(time_canvas), Some(bode_canvas)) =
                                                                (time_canvas_ref.get(), bode_canvas_ref.get())
                                                            {
                                                                draw_time_response(
                                                                    time_canvas,
                                                                    signal_instance.as_ref(),
                                                                    circuit_instance.as_mut(),
                                                                );

                                                                draw_bode_diagram(
                                                                    bode_canvas,
                                                                    circuit_instance.as_ref(),
                                                                );
                                                            }
                                                        }
                                                        (Err(error), _) | (_, Err(error)) => {
                                                            set_simulation_preview.set(None);
                                                            set_calculation_error.set(Some(error.to_string()));
                                                        }
                                                    }
                                                }
                                                (Err(error), _) | (_, Err(error)) => {
                                                    set_simulation_preview.set(None);
                                                    set_calculation_error.set(Some(error));
                                                }
                                            }
                                        }
                                    >
                                        "Berechnen"
                                    </button>


                                </form>

                                <div class="result-panel">
                                <h2>"Berechnete Werte"</h2>

                                {move || {
                                    calculation_error
                                        .get()
                                        .map(|error| view! {
                                            <p class="calculation-error">
                                                {error}
                                            </p>
                                        })
                                }}

                                {move || {
                                    simulation_preview
                                        .get()
                                        .map(|preview| view! {
                                            <div>
                                                <p>
                                                    <strong>"Amplitude Ue: "</strong>
                                                    {format!("{:.3} V", preview.input_amplitude)}
                                                </p>

                                                <p>
                                                    <strong>"Amplitude Ua: "</strong>
                                                    {format!("{:.3} V", preview.output_amplitude)}
                                                </p>

                                                <p>
                                                    <strong>"Grenzfrequenz: "</strong>
                                                    {format!("{:.3} Hz", preview.cutoff_frequency)}
                                                </p>
                                            </div>
                                        })
                                }}
                            </div>
                            </div>
                            </div>


                                {/* Bereich E: Zeitdiagramm */}
                                <div class="diagram-panel">
                                    <h2>"Zeitdiagramm"</h2>
                                    <canvas
                                        id="time-response-canvas"
                                        node_ref=time_canvas_ref
                                        width="1920"
                                        height="1080"
                                        style="width: 640px; height: 360px;"
                                    ></canvas>
                                </div>

                                {/* Bereich F: Bode-Diagramm */}
                                <div class="diagram-panel">
                                    <h2>"Bode-Diagramm"</h2>
                                    <canvas
                                        id="bode-canvas"
                                        node_ref=bode_canvas_ref
                                        width="1920"
                                        height="1080"
                                        style="width: 640px; height: 360px;"
                                    ></canvas>
                                </div>

                            </div>


                            {/* Rechtes Panel: Eingabeformular für Variablen */}

                    }.into_any(),

                    None => view! {
                        <h1>"Schaltung nicht gefunden"</h1>
                    }.into_any(),
                }}
            </section>
        }
}
