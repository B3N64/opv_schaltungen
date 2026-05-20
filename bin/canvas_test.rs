mod circuits;
mod errors;
mod plot;
mod signals;

use crate::circuits::*;
use crate::plot::{draw_bode_diagram, draw_time_response};
use crate::signals::Signal; // Expliziter Import des Traits
use crate::signals::{CombinedSignal, Sinus}; // Import der benötigten Typen
use web_sys::console;

use crate::signals::*;
use leptos::ev::SubmitEvent;
use leptos::html::Canvas;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlInputElement};

#[component]
pub fn App() -> impl IntoView {
    let canvas_ref_time = NodeRef::<Canvas>::new();
    let canvas_ref_freq = NodeRef::<Canvas>::new();

    // Reaktive Signale für Frequenzen und Amplitude
    let (frequency_1, set_frequency_1) = signal(10.0);
    let (frequency_2, set_frequency_2) = signal(1400.0);
    let (amplitude, set_amplitude) = signal(1.0);

    // Effekt für das Neuzeichnen beider Canvas
    Effect::new(move |_| {
        // Damit der Effekt auch bei frequency_1 neu läuft (falls du es später brauchst)
        let _ = frequency_1.get();

        let freq2 = frequency_2.get();
        let amp = amplitude.get();

        let sinus = Sinus::new(amp, freq2, 0.0);

        let r = 1000.0;
        let c = 0.000001;

        // Separates Exemplar für die Zeitsimulation (mutiert internen State)
        let mut circuit_time = Differentiator::new(r, c);
        // Separates Exemplar für den Frequenzgang
        let circuit_freq = Differentiator::new(r, c);

        let duration = 2.0 * 1.0 / freq2;
        let step = duration / 1000.0;

        let input_values = sinus.generate(duration, step);
        let output_values = circuit_time.simulate(&sinus, duration, step);

        // Time-Canvas
        if let Some(canvas) = canvas_ref_time.get() {
            let canvas: HtmlCanvasElement = canvas.unchecked_into();
            draw_time_response(canvas, input_values, output_values);
        }

        // Frequency-Canvas (Bode)
        if let Some(canvas) = canvas_ref_freq.get() {
            let canvas: HtmlCanvasElement = canvas.unchecked_into();
            draw_bode_diagram(canvas, &circuit_freq);
        }
    });

    view! {
        <div>
            <p>
                "Frequenz 1: "
                {move || format!("{:.1}", frequency_1.get())}
            </p>
            <input
                type="range"
                min="1"
                max="100"
                step="1"
                on:input=move |ev| {
                    let value = event_target_value(&ev)
                        .parse::<f64>()
                        .unwrap();
                    set_frequency_1.set(value);
                }
            />

            <p>
                "Frequenz 2: "
                {move || format!("{:.1}", frequency_2.get())}
            </p>
            <input
                type="range"
                min="1"
                max="10000"
                step="1"
                on:input=move |ev| {
                    let value = event_target_value(&ev)
                        .parse::<f64>()
                        .unwrap();
                    set_frequency_2.set(value);
                }
            />

            <p>
                "Amplitude: "
                {move || format!("{:.1}", amplitude.get())}
            </p>
            <input
                type="range"
                min="0.1"
                max="10.0"
                step="0.1"
                on:input=move |ev| {
                    let value = event_target_value(&ev)
                        .parse::<f64>()
                        .unwrap();
                    set_amplitude.set(value);
                }
            />

            <canvas
                node_ref=canvas_ref_time
                width="1920"
                height="1080"
                style="width: 640px; height: 360px;">
            </canvas>
            <p>"Frequenzgang"</p>
            <canvas
                node_ref=canvas_ref_freq
                width="1920"
                height="1080"
                style="width: 640px; height: 360px;">
            </canvas>
        </div>
    }
}

fn main() {
    leptos::mount::mount_to_body(App)
}
