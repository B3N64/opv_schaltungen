use plotters::prelude::*;
use std::f64::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // =========================
    // SYSTEMPARAMETER
    // =========================

    // Eingangssignal
    let amplitude = 1.0;      // 1 V
    let frequency = 50.0;     // 50 Hz
    let omega = 2.0 * PI * frequency;

    // Tiefpass:
    // G(s) = 1 / (1 + sRC)
    let r = 1000.0;           // 1 kOhm
    let c = 100e-6;           // 100 µF

    // =========================
    // ÜBERTRAGUNGSFUNKTION
    // =========================

    // Betrag von G(jw)
    let gain =
        1.0 / (1.0 + (omega * r * c).powi(2)).sqrt();

    // Phase von G(jw)
    let phase =
        -(omega * r * c).atan();

    println!("Verstärkung: {}", gain);
    println!("Phase [rad]: {}", phase);

    // =========================
    // ZEITBEREICH
    // =========================

    let t_max = 0.1;      // 100 ms
    let samples = 2000;

    // Eingangssignal
    let input_signal: Vec<(f64, f64)> = (0..samples)
        .map(|i| {

            let t =
                i as f64 * t_max / samples as f64;

            let u =
                amplitude * (omega * t).sin();

            (t, u)
        })
        .collect();

    // Ausgangssignal
    let output_signal: Vec<(f64, f64)> = (0..samples)
        .map(|i| {

            let t =
                i as f64 * t_max / samples as f64;

            let y =
                amplitude
                * gain
                * (omega * t + phase).sin();

            (t, y)
        })
        .collect();

    // =========================
    // PLOTTERS
    // =========================

    let root =
        BitMapBackend::new("plot.png", (1200, 700))
        .into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Eingangs- und Ausgangssignal",
            ("sans-serif", 35),
        )
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(
            0.0..t_max,
            -1.5..1.5,
        )?;

    chart
        .configure_mesh()
        .x_desc("Zeit [s]")
        .y_desc("Spannung [V]")
        .draw()?;

    // Eingang
    chart
        .draw_series(
            LineSeries::new(input_signal, &BLUE),
        )?
        .label("Eingang u(t)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                &BLUE,
            )
        });

    // Ausgang
    chart
        .draw_series(
            LineSeries::new(output_signal, &RED),
        )?
        .label("Ausgang y(t)")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 20, y)],
                &RED,
            )
        });

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    println!("Plot gespeichert als plot.png");

    Ok(())
}