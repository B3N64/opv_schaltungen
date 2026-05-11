use plotters::prelude::*;

fn triangle_wave(t: f64, frequency: f64, amplitude: f64) -> f64 {
    let period = 1.0 / frequency;
    let x = (t / period) % 1.0;

    if x < 0.25 {
        4.0 * amplitude * x
    } else if x < 0.75 {
        2.0 * amplitude - 4.0 * amplitude * x
    } else {
        -4.0 * amplitude + 4.0 * amplitude * x
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let amplitude = 1.0;
    let frequency = 5.0;

    // Integrator-Faktor:
    // z.B. beim invertierenden OPV-Integrator:
    // K = -1 / (R * C)
    let r = 10_000.0;
    let c = 10e-6;
    let k = -1.0 / (r * c);

    let t_max = 1.0;
    let samples = 5000;
    let dt = t_max / samples as f64;

    let mut input_signal: Vec<(f64, f64)> = Vec::new();
    let mut output_signal: Vec<(f64, f64)> = Vec::new();

    let mut y = 0.0;

    for i in 0..samples {
        let t = i as f64 * dt;

        let u = triangle_wave(t, frequency, amplitude);

        // numerische Integration
        y += k * u * dt;

        input_signal.push((t, u));
        output_signal.push((t, y));
    }

    let root =
        BitMapBackend::new("integrator_triangle.png", (1200, 700))
            .into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Integrator mit Dreiecksignal", ("sans-serif", 35))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..t_max, -2.0..2.0)?;

    chart
        .configure_mesh()
        .x_desc("Zeit [s]")
        .y_desc("Spannung [V]")
        .draw()?;

    chart
        .draw_series(LineSeries::new(input_signal, &BLUE))?
        .label("Eingang Dreieck u(t)")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], &BLUE)
        });

    chart
        .draw_series(LineSeries::new(output_signal, &RED))?
        .label("Ausgang Integrator y(t)")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], &RED)
        });

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    println!("Plot gespeichert als integrator_triangle.png");

    Ok(())
}