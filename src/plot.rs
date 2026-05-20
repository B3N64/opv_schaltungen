use std::panic::PanicHookInfo;
use std::vec;

use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters_canvas::CanvasBackend;

use std::f64::consts::PI;
use web_sys::HtmlCanvasElement;
use web_sys::console;

use crate::circuits::Circuit;
use crate::signals::Signal;

const BACKGROUND_COLOR: RGBColor = RGBColor(30, 33, 38);
const BORDER_COLOR: RGBColor = RGBColor(255, 255, 255);

fn wrap_phase_deg(mut deg: f64) -> f64 {
    if !deg.is_finite() {
        return deg;
    }

    // [0, 360)
    deg = deg.rem_euclid(360.0);
    // (-180, 180]
    if deg > 180.0 {
        deg -= 360.0;
    }
    deg
}

// TODO: parameter zu signal und circuit
pub fn draw_bode_diagram(canvas: HtmlCanvasElement, circuit: &dyn Circuit) {
    let backend = CanvasBackend::with_canvas_object(canvas).expect("canvas backend");
    let root = backend.into_drawing_area();
    root.fill(&BACKGROUND_COLOR).unwrap();

    let cutoff_freq = circuit.cutoff_frequency().max(f64::MIN_POSITIVE);

    let decades_each_side = 3.0_f64;
    let span = 10.0_f64.powf(decades_each_side);

    let min_freq = (cutoff_freq / span).max(f64::MIN_POSITIVE);
    let max_freq = cutoff_freq * span;

    let mut amplitude_response = vec![];
    let mut phase_response = vec![];

    let samples = 600usize;
    let ratio = max_freq / min_freq;

    for i in 0..=samples {
        let t = i as f64 / samples as f64; // 0..1
        let freq = min_freq * ratio.powf(t);

        let a = circuit.amplitude_at(freq).abs().max(1e-12);
        let amplitude = 20.0 * a.log10();

        let mut phase = circuit.phase_at(freq).to_degrees();
        phase = wrap_phase_deg(phase).abs();

        amplitude_response.push((freq, amplitude));
        phase_response.push((freq, phase));
    }

    let min_amplitude = amplitude_response
        .iter()
        .map(|(_, u)| u.abs())
        .fold(0.0_f64, |min, u| min.min(u));

    let max_amplitude = amplitude_response
        .iter()
        .map(|(_, u)| u.abs())
        .fold(0.0_f64, |max, u| max.max(u));

    let abs_max_amplitude = if max_amplitude.abs() > min_amplitude.abs() {
        max_amplitude.abs()
    } else {
        min_amplitude.abs()
    };

    let max_phase = phase_response
        .iter()
        .map(|(_, p)| *p)
        .fold(0.0_f64, |max, p| max.max(p));

    let pad = 10.0_f64;

    let phase_y_min = -pad;

    let phase_y_max = max_phase + pad;

    let pixel_range = root.get_pixel_range();
    root.draw(&Rectangle::new(
        [
            (pixel_range.0.start, pixel_range.1.start),
            (pixel_range.0.end, pixel_range.1.end),
        ], // Use corner points
        ShapeStyle::from(WHITE).stroke_width(4), // White border with 2px thickness
    ))
    .unwrap();

    // upper: Frequenzgang
    let mut chart = ChartBuilder::on(&root)
        .margin(40)
        .caption("Bode Diagram", ("sans-serif", 60).into_font().color(&WHITE))
        .x_label_area_size(100)
        .y_label_area_size(180)
        .right_y_label_area_size(200)
        .build_cartesian_2d(
            (min_freq..max_freq).log_scale(),
            -abs_max_amplitude..abs_max_amplitude,
        )
        .unwrap()
        .set_secondary_coord((min_freq..max_freq).log_scale(), phase_y_min..phase_y_max); // Rechte Y-Achse

    chart
        .configure_mesh()
        .axis_style(ShapeStyle::from(&WHITE).stroke_width(4))
        .label_style(("sans-serif", 40).into_font().color(&WHITE))
        .set_all_tick_mark_size(10)
        .x_desc("f [Hz]")
        .y_desc("Amplitude [dB]")
        .bold_line_style(&WHITE.mix(0.3))
        .light_line_style(&WHITE.mix(0.1))
        .draw()
        .unwrap();

    chart
        .configure_secondary_axes()
        .axis_style(ShapeStyle::from(&WHITE).stroke_width(4))
        .set_all_tick_mark_size(10)
        .y_labels(10)
        .y_desc("Phase [°]")
        .label_style(("sans-serif", 40).into_font().color(&WHITE))
        .draw()
        .unwrap();

    let y_min = -abs_max_amplitude;
    let y_max = abs_max_amplitude;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(cutoff_freq, y_min), (cutoff_freq, y_max)],
            ShapeStyle::from(&YELLOW.mix(0.3)).stroke_width(6),
        )))
        .unwrap();

    let cutoff_amp_db = 20.0 * circuit.amplitude_at(cutoff_freq).abs().max(1e-12).log10();

    chart
        .draw_series(std::iter::once(Circle::new(
            (cutoff_freq, cutoff_amp_db),
            10,
            ShapeStyle::from(&YELLOW).filled(),
        )))
        .unwrap();

    chart
        .draw_series(std::iter::once(Text::new(
            format!("f_c = {:.2} Hz", cutoff_freq),
            (cutoff_freq, y_max - 3.0),
            ("sans-serif", 32)
                .into_font()
                .color(&YELLOW)
                .pos(Pos::new(HPos::Center, VPos::Top)),
        )))
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            amplitude_response,
            ShapeStyle::from(&RED).stroke_width(8),
        ))
        .unwrap()
        .label("Input Signal")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 40, y)],
                ShapeStyle::from(&RED).stroke_width(4),
            )
        });

    chart
        .draw_secondary_series(LineSeries::new(
            phase_response,
            ShapeStyle::from(&BLUE).stroke_width(8),
        ))
        .unwrap()
        .label("Phase")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y - 5), (x + 40, y - 5)],
                ShapeStyle::from(&BLUE).stroke_width(4),
            )
        });

    root.present().unwrap();
}

// TODO: parameter zu signal und circuit
pub fn draw_time_response(
    canvas: HtmlCanvasElement,
    signal: Vec<(f64, f64)>,
    circuit: Vec<(f64, f64)>,
) {
    let backend = CanvasBackend::with_canvas_object(canvas).expect("canvas backend");
    let root = backend.into_drawing_area();
    root.fill(&BACKGROUND_COLOR).unwrap();

    let max_t = signal.last().map(|(t, _)| *t).unwrap_or(0.5);
    let max_u = circuit
        .iter()
        .map(|(_, u)| u.abs())
        .fold(0.0_f64, |max, u| max.max(u));

    let base = 1.0_f64;
    let factor = 4.0_f64;
    let max_y = if max_u <= base {
        base
    } else {
        let n = ((max_u / base).ln() / factor.ln()).ceil() as i32;
        base * factor.powi(n)
    };

    let pixel_range = root.get_pixel_range();
    root.draw(&Rectangle::new(
        [
            (pixel_range.0.start, pixel_range.1.start),
            (pixel_range.0.end, pixel_range.1.end),
        ], // Use corner points
        ShapeStyle::from(WHITE).stroke_width(4), // White border with 2px thickness
    ))
    .unwrap();

    let mut chart = ChartBuilder::on(&root)
        .margin(40)
        .caption(
            "Two-Scales Plot",
            ("sans-serif", 60).into_font().color(&WHITE),
        )
        /*         .top_x_label_area_size(40) */
        .x_label_area_size(100)
        .y_label_area_size(120)
        .right_y_label_area_size(160)
        .build_cartesian_2d(0f64..max_t, -1.5f64..1.5f64) // Linke Y-Achse
        .unwrap()
        .set_secondary_coord(0f64..max_t, -max_y..max_y); // Rechte Y-Achse

    // Zeichne das Gitter für die linke Y-Achse
    chart
        .configure_mesh()
        .axis_style(ShapeStyle::from(&WHITE).stroke_width(4))
        .label_style(("sans-serif", 40).into_font().color(&WHITE))
        .set_all_tick_mark_size(10)
        .y_desc("Input Signal")
        .x_desc("t [s]")
        .bold_line_style(&WHITE.mix(0.3))
        .light_line_style(&WHITE.mix(0.1))
        .draw()
        .unwrap();

    // Zeichne das Gitter für die rechte Y-Achse
    chart
        .configure_secondary_axes()
        .axis_style(ShapeStyle::from(&WHITE).stroke_width(4))
        .set_all_tick_mark_size(10)
        .y_labels(10)
        .y_desc("Output Signal")
        .label_style(("sans-serif", 40).into_font().color(&WHITE))
        .draw()
        .unwrap();

    // Zeichne das Input-Signal (linke Y-Achse)
    chart
        .draw_series(LineSeries::new(
            signal,
            ShapeStyle::from(&RED).stroke_width(8),
        ))
        .unwrap()
        .label("Input Signal")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y), (x + 40, y)],
                ShapeStyle::from(&RED).stroke_width(4),
            )
        });

    // Zeichne das Output-Signal (rechte Y-Achse)
    chart
        .draw_secondary_series(LineSeries::new(
            circuit,
            ShapeStyle::from(&BLUE).stroke_width(8),
        ))
        .unwrap()
        .label("Output Signal")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y - 5), (x + 40, y - 5)],
                ShapeStyle::from(&BLUE).stroke_width(4),
            )
        });

    // Zeichne die Legende
    chart
        .configure_series_labels()
        .background_style(&BACKGROUND_COLOR)
        .border_style(&WHITE)
        .legend_area_size(60)
        .label_font(("sans-serif", 40).into_font().color(&WHITE))
        .position(SeriesLabelPosition::UpperRight)
        .draw()
        .unwrap();

    root.present().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsCast;
    use web_sys::{Document, HtmlCanvasElement, window};

    #[test]
    fn test_canvas_backend() {}
}
