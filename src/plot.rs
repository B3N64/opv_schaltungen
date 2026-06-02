use std::vec;

use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters_canvas::CanvasBackend;

use web_sys::HtmlCanvasElement;

use crate::circuits::Circuit;
use crate::signals::Signal;

const BACKGROUND_COLOR: RGBColor = RGBColor(30, 33, 38);

const OUTLINE_LINE_WIDTH: u32 = 4;
const OUTLINE_MARGIN: u32 = 40;
const OUTLINE_COLOR: RGBColor = RGBColor(255, 255, 255);

const CAPTION_FONT_SIZE: u32 = 60;

const AXIS_FONT_SIZE: u32 = 40;
const AXIS_LINE_WIDTH: u32 = 4;
const AXIS_ARIA_SIZE_X: u32 = 120;
const AXIS_AREA_SIZE_Y_L: u32 = 180;
const AXIS_AREA_SIZE_Y_R: u32 = 200;

const FONT_COLOR: RGBColor = RGBColor(255, 255, 255);

const SIGNAL_1_COLOR: RGBColor = RGBColor(255, 0, 0);
const SIGNAL_2_COLOR: RGBColor = RGBColor(0, 0, 255);
const CUTOFF_COLOR: RGBColor = RGBColor(255, 255, 0);

const DATA_LINE_WIDTH: u32 = 8;

const LEGEND_FONT_SIZE: u32 = 32;
const LEGEND_AREA_SIZE: u32 = 60;
const LEGEND_LINE_WIDTH: u32 = 4;
const LEGEND_LINE_LENGTH: u32 = 50;
const LEGEND_MARGIN: u32 = 30;

const BOLD_LINE_MIX: f64 = 0.3;
const LIGHT_LINE_MIX: f64 = 0.1;

/// n, µ, m, k, M, G
fn format_si(v: f64) -> String {
    if !v.is_finite() {
        return format!("{v}");
    }
    if v == 0.0 {
        return "0".to_string();
    }

    fn trim_zeros(mut s: String) -> String {
        if s.contains('.') {
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
        }
        if s == "-0" { "0".to_string() } else { s }
    }

    fn format_sig(v: f64, sig: i32) -> String {
        if v == 0.0 {
            return "0".to_string();
        }
        let a = v.abs();
        let exp = a.log10().floor() as i32;
        let decimals = (sig - 1 - exp).max(0).min(6) as usize;
        trim_zeros(format!("{:.*}", decimals, v))
    }

    let a = v.abs();

    if a >= 0.1 && a < 1000.0 {
        let s = format_sig(v, 3);
        if s == "1000" {
            return "1k".to_string();
        }
        if s == "-1000" {
            return "-1k".to_string();
        }
        return s;
    }

    let mut factor: f64;
    let mut prefix: &str;

    if a >= 1000.0 {
        (factor, prefix) = (1e9, "G");
        for (f, p) in [(1e9, "G"), (1e6, "M"), (1e3, "k"), (1.0, "")] {
            let scaled = a / f;
            if scaled >= 1.0 && scaled < 1000.0 {
                factor = f;
                prefix = p;
                break;
            }
        }
    } else {
        (factor, prefix) = (1e-9, "n");
        for (f, p) in [(1e-3, "m"), (1e-6, "µ"), (1e-9, "n")] {
            let scaled = a / f;
            if scaled >= 1.0 && scaled < 1000.0 {
                factor = f;
                prefix = p;
                break;
            }
        }
    }

    let scaled_abs = (v / factor).abs();
    if scaled_abs >= 999.5 {
        match prefix {
            "n" => {
                factor = 1e-6;
                prefix = "µ";
            }
            "µ" => {
                factor = 1e-3;
                prefix = "m";
            }
            "m" => {
                factor = 1.0;
                prefix = "";
            }
            "" => {
                factor = 1e3;
                prefix = "k";
            }
            "k" => {
                factor = 1e6;
                prefix = "M";
            }
            "M" => {
                factor = 1e9;
                prefix = "G";
            }
            _ => {}
        }
    }

    let scaled = v / factor;
    format!("{}{}", format_sig(scaled, 3), prefix)
}

fn choose_si_axis_scale(max_abs: f64) -> (f64, &'static str) {
    if !max_abs.is_finite() || max_abs == 0.0 {
        return (1.0, "");
    }

    if max_abs >= 0.1 && max_abs < 1000.0 {
        return (1.0, "");
    }

    if max_abs >= 1e9 {
        (1e9, "G")
    } else if max_abs >= 1e6 {
        (1e6, "M")
    } else if max_abs >= 1e3 {
        (1e3, "k")
    } else if max_abs >= 1e-3 {
        (1e-3, "m")
    } else if max_abs >= 1e-6 {
        (1e-6, "µ")
    } else {
        (1e-9, "n")
    }
}

fn nice_step(range: f64, target: f64) -> f64 {
    if !range.is_finite() || range <= 0.0 || !target.is_finite() || target <= 0.0 {
        return 1.0;
    }

    let raw = range / target;
    let exp = 10.0_f64.powf(raw.log10().floor());
    let frac = raw / exp;

    let nice_frac = if frac < 1.5 {
        1.0
    } else if frac < 3.0 {
        2.0
    } else if frac < 7.0 {
        5.0
    } else {
        10.0
    };

    nice_frac * exp
}

fn decimals_for_step(step: f64) -> usize {
    if !step.is_finite() || step <= 0.0 {
        return 0;
    }

    let mut scale = 1.0_f64;
    for d in 0..=6 {
        let x = step * scale;
        if (x - x.round()).abs() < 1e-6 {
            return d;
        }
        scale *= 10.0;
    }
    6
}

fn format_si_fixed(v: f64, factor: f64, prefix: &str, decimals: usize) -> String {
    if !v.is_finite() {
        return format!("{v}");
    }

    if v == 0.0 {
        return if decimals == 0 {
            "0".to_string()
        } else {
            format!("{:.*}", decimals, 0.0_f64)
        };
    }

    let scaled = v / factor;

    let mut s = if decimals == 0 {
        format!("{:.0}", scaled)
    } else {
        format!("{:.*}", decimals, scaled)
    };

    // -0 / -0.0 vermeiden
    if s.starts_with("-0") {
        let rest = &s[2..];
        if rest.is_empty() || rest.chars().all(|c| c == '.' || c == '0') {
            s = if decimals == 0 {
                "0".to_string()
            } else {
                format!("{:.*}", decimals, 0.0_f64)
            };
        }
    }

    format!("{}{}", s, prefix)
}

pub fn draw_bode_diagram(canvas: HtmlCanvasElement, circuit: &dyn Circuit) {
    let backend = CanvasBackend::with_canvas_object(canvas).expect("canvas backend");
    let root = backend.into_drawing_area();
    root.fill(&BACKGROUND_COLOR).unwrap();

    let cutoff_freq = circuit.cutoff_frequency().max(f64::MIN_POSITIVE);

    let decades_each_side = 3.0;
    let span = 10.0_f64.powf(decades_each_side);

    let min_freq = (cutoff_freq / span).max(f64::MIN_POSITIVE);
    let max_freq = cutoff_freq * span;

    let samples: usize = 600;
    let ratio = max_freq / min_freq;

    let mut amplitude_response = Vec::with_capacity(samples + 1);
    let mut phase_response = Vec::with_capacity(samples + 1);

    let mut min_amplitude = f64::INFINITY;
    let mut max_amplitude = f64::NEG_INFINITY;
    let mut max_phase = f64::NEG_INFINITY;
    let mut min_phase = f64::INFINITY;

    for i in 0..=samples {
        let t = i as f64 / samples as f64; // 0..1
        let freq = min_freq * ratio.powf(t);

        let a = circuit.amplitude_at(freq).abs().max(f64::MIN_POSITIVE);
        let amplitude = 20.0 * a.log10();

        let phase = circuit.phase_at(freq).to_degrees();

        amplitude_response.push((freq, amplitude));
        phase_response.push((freq, phase));

        if amplitude < min_amplitude {
            min_amplitude = amplitude;
        }
        if amplitude > max_amplitude {
            max_amplitude = amplitude;
        }
        if phase > max_phase {
            max_phase = phase;
        }
        if phase < min_phase {
            min_phase = phase;
        }
    }

    if max_amplitude.abs() != min_amplitude.abs() {
        if max_amplitude.abs() < min_amplitude.abs() {
            max_amplitude += 10.1;
        } else {
            min_amplitude -= 10.1;
        }
    }

    if max_phase.abs() == min_phase.abs() {
        if max_phase > 0.0 {
            max_phase = 100.0;
            min_phase = -10.0;
        } else {
            max_phase = 10.0;
            min_phase = -100.0;
        }
    } else {
        min_phase -= 10.1;
        max_phase += 10.1;
    }

    // Chart Umrandung
    let pixel_range = root.get_pixel_range();
    root.draw(&Rectangle::new(
        [
            (pixel_range.0.start, pixel_range.1.start),
            (pixel_range.0.end, pixel_range.1.end),
        ],
        ShapeStyle::from(OUTLINE_COLOR).stroke_width(OUTLINE_LINE_WIDTH),
    ))
    .unwrap();

    // Hauptchart
    let mut chart = ChartBuilder::on(&root)
        .margin(OUTLINE_MARGIN)
        .caption(
            "Bode Diagram",
            ("sans-serif", CAPTION_FONT_SIZE)
                .into_font()
                .color(&FONT_COLOR),
        )
        .x_label_area_size(AXIS_ARIA_SIZE_X)
        .y_label_area_size(AXIS_AREA_SIZE_Y_L)
        .right_y_label_area_size(AXIS_AREA_SIZE_Y_R)
        .build_cartesian_2d(
            (min_freq..max_freq).log_scale(),
            min_amplitude..max_amplitude,
        )
        .unwrap()
        .set_secondary_coord((min_freq..max_freq).log_scale(), min_phase..max_phase);

    // Gitter (linke Y-Achse)
    chart
        .configure_mesh()
        .axis_style(ShapeStyle::from(&OUTLINE_COLOR).stroke_width(AXIS_LINE_WIDTH))
        .label_style(
            ("sans-serif", AXIS_FONT_SIZE)
                .into_font()
                .color(&FONT_COLOR),
        )
        .set_all_tick_mark_size(10)
        .x_label_formatter(&|v| format_si(*v))
        .x_desc("f [Hz]")
        .y_desc("Amplitude [dB]")
        .bold_line_style(&OUTLINE_COLOR.mix(BOLD_LINE_MIX))
        .light_line_style(&OUTLINE_COLOR.mix(LIGHT_LINE_MIX))
        .draw()
        .unwrap();

    // Gitter (rechte Y-Achse)
    chart
        .configure_secondary_axes()
        .axis_style(ShapeStyle::from(&OUTLINE_COLOR).stroke_width(AXIS_LINE_WIDTH))
        .set_all_tick_mark_size(10)
        .y_labels(12)
        .y_desc("Phase [°]")
        .label_style(
            ("sans-serif", AXIS_FONT_SIZE)
                .into_font()
                .color(&FONT_COLOR),
        )
        .draw()
        .unwrap();

    // Durchtrittsfrequenz vertikale Linie
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(cutoff_freq, min_amplitude), (cutoff_freq, max_amplitude)],
            ShapeStyle::from(&CUTOFF_COLOR.mix(BOLD_LINE_MIX)).stroke_width(DATA_LINE_WIDTH),
        )))
        .unwrap();

    // Durchtrittsfrequenz Beschriftung
    chart
        .draw_series(std::iter::once(Text::new(
            format!("fc = {:.2} Hz", cutoff_freq),
            (cutoff_freq, max_amplitude - 3.0),
            ("sans-serif", LEGEND_FONT_SIZE)
                .into_font()
                .color(&CUTOFF_COLOR)
                .pos(Pos::new(HPos::Center, VPos::Top)),
        )))
        .unwrap();

    // Amplitudengang
    chart
        .draw_series(LineSeries::new(
            amplitude_response,
            ShapeStyle::from(&SIGNAL_1_COLOR).stroke_width(DATA_LINE_WIDTH),
        ))
        .unwrap()
        .label("Amplitude")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y - 10), (x + LEGEND_LINE_LENGTH as i32, y - 10)],
                ShapeStyle::from(&SIGNAL_1_COLOR).stroke_width(LEGEND_LINE_WIDTH),
            )
        });

    // Phasegang
    chart
        .draw_secondary_series(LineSeries::new(
            phase_response,
            ShapeStyle::from(&SIGNAL_2_COLOR).stroke_width(DATA_LINE_WIDTH),
        ))
        .unwrap()
        .label("Phase")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y - 10), (x + LEGEND_LINE_LENGTH as i32, y - 10)],
                ShapeStyle::from(&SIGNAL_2_COLOR).stroke_width(LEGEND_LINE_WIDTH),
            )
        });

    // Legende
    chart
        .configure_series_labels()
        .legend_area_size(LEGEND_AREA_SIZE)
        .label_font(
            ("sans-serif", LEGEND_FONT_SIZE)
                .into_font()
                .color(&FONT_COLOR),
        )
        .margin(LEGEND_MARGIN)
        .position(SeriesLabelPosition::UpperRight)
        .draw()
        .unwrap();

    root.present().unwrap();
}

pub fn draw_time_response(
    canvas: HtmlCanvasElement,
    signal: &dyn Signal,
    circuit: &mut dyn Circuit,
) {
    let backend = CanvasBackend::with_canvas_object(canvas).expect("canvas backend");
    let root = backend.into_drawing_area();
    root.fill(&BACKGROUND_COLOR).unwrap();

    let duration = 2.0 / signal.frequency();
    let samples: usize = 1000;

    let mut signal_values = Vec::with_capacity(samples + 1);
    let mut circuit_values = Vec::with_capacity(samples + 1);

    let mut max_t = 0.0;
    let mut max_ue = 0.0;
    let mut max_ua = 0.0;

    for i in 0..=samples {
        let t = i as f64 / samples as f64 * duration;
        let ue = signal.value_at(t);
        signal_values.push((t, ue));
        let ua = circuit.output_voltage(ue, duration / samples as f64);
        circuit_values.push((t, ua));

        if max_t < t {
            max_t = t;
        }
        if max_ue < ue.abs() {
            max_ue = ue.abs();
        }
        if max_ua < ua.abs() {
            max_ua = ua.abs();
        }
    }

    // ue 1/2 vom max_ue
    max_ue *= 1.5;

    // ua zwischen 1/3 - 1 vom max_ua
    let base = 1.0_f64;
    let factor = 4.0_f64;
    let max_ua = if max_ua <= base {
        base
    } else {
        let n = ((max_ua / base).ln() / factor.ln()).ceil() as i32;
        base * factor.powi(n)
    };

    // Erster Punkt ist nicht immer Null.
    circuit_values[0] = circuit_values[1];

    // Chart Umrandung
    let pixel_range = root.get_pixel_range();
    root.draw(&Rectangle::new(
        [
            (pixel_range.0.start, pixel_range.1.start),
            (pixel_range.0.end, pixel_range.1.end),
        ],
        ShapeStyle::from(OUTLINE_COLOR).stroke_width(OUTLINE_LINE_WIDTH),
    ))
    .unwrap();

    // Hauptchart
    let mut chart = ChartBuilder::on(&root)
        .margin(OUTLINE_MARGIN)
        .caption(
            "Two-Scales Plot",
            ("sans-serif", CAPTION_FONT_SIZE)
                .into_font()
                .color(&FONT_COLOR),
        )
        .x_label_area_size(AXIS_ARIA_SIZE_X)
        .y_label_area_size(AXIS_AREA_SIZE_Y_L)
        .right_y_label_area_size(AXIS_AREA_SIZE_Y_R)
        .build_cartesian_2d(0f64..max_t, -max_ue..max_ue)
        .unwrap()
        .set_secondary_coord(0f64..max_t, -max_ua..max_ua);

    let target_ticks = 10.0_f64;

    let (x_factor, x_prefix) = choose_si_axis_scale(max_t.abs());
    let x_step = nice_step((max_t / x_factor).abs(), target_ticks);
    let x_decimals = decimals_for_step(x_step);
    let x_fmt = |v: &f64| format_si_fixed(*v, x_factor, x_prefix, x_decimals);

    let (y_factor, y_prefix) = choose_si_axis_scale(max_ue.abs());
    let y_step = nice_step(((2.0 * max_ue) / y_factor).abs(), target_ticks);
    let y_decimals = decimals_for_step(y_step);
    let y_fmt = |v: &f64| format_si_fixed(*v, y_factor, y_prefix, y_decimals);

    let (y2_factor, y2_prefix) = choose_si_axis_scale(max_ua.abs());
    let y2_step = nice_step(((2.0 * max_ua) / y2_factor).abs(), target_ticks);
    let y2_decimals = decimals_for_step(y2_step);
    let y2_fmt = |v: &f64| format_si_fixed(*v, y2_factor, y2_prefix, y2_decimals);

    // Gitter (linke Y-Achse)
    chart
        .configure_mesh()
        .axis_style(ShapeStyle::from(&OUTLINE_COLOR).stroke_width(AXIS_LINE_WIDTH))
        .label_style(
            ("sans-serif", AXIS_FONT_SIZE)
                .into_font()
                .color(&FONT_COLOR),
        )
        .set_all_tick_mark_size(10)
        .x_label_formatter(&x_fmt)
        .y_label_formatter(&y_fmt)
        .y_desc("Input Signal [V]")
        .x_desc("t [s]")
        .bold_line_style(&OUTLINE_COLOR.mix(BOLD_LINE_MIX))
        .light_line_style(&OUTLINE_COLOR.mix(LIGHT_LINE_MIX))
        .draw()
        .unwrap();

    // Gitter (rechte Y-Achse)
    chart
        .configure_secondary_axes()
        .axis_style(ShapeStyle::from(&OUTLINE_COLOR).stroke_width(AXIS_LINE_WIDTH))
        .label_style(
            ("sans-serif", AXIS_FONT_SIZE)
                .into_font()
                .color(&FONT_COLOR),
        )
        .set_all_tick_mark_size(10)
        .y_labels(10)
        .y_label_formatter(&y2_fmt)
        .y_desc("Output Signal [V]")
        .draw()
        .unwrap();

    // Input-Signal (linke Y-Achse)
    chart
        .draw_series(LineSeries::new(
            signal_values,
            ShapeStyle::from(&SIGNAL_1_COLOR).stroke_width(DATA_LINE_WIDTH),
        ))
        .unwrap()
        .label("Input Signal")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y - 10), (x + LEGEND_LINE_LENGTH as i32, y - 10)],
                ShapeStyle::from(&SIGNAL_1_COLOR).stroke_width(LEGEND_LINE_WIDTH),
            )
        });

    // Output-Signal (rechte Y-Achse)
    chart
        .draw_secondary_series(LineSeries::new(
            circuit_values,
            ShapeStyle::from(&SIGNAL_2_COLOR).stroke_width(DATA_LINE_WIDTH),
        ))
        .unwrap()
        .label("Output Signal")
        .legend(|(x, y)| {
            PathElement::new(
                vec![(x, y - 10), (x + LEGEND_LINE_LENGTH as i32, y - 10)],
                ShapeStyle::from(&SIGNAL_2_COLOR).stroke_width(LEGEND_LINE_WIDTH),
            )
        });

    // Legende
    chart
        .configure_series_labels()
        .legend_area_size(LEGEND_AREA_SIZE)
        .label_font(
            ("sans-serif", LEGEND_FONT_SIZE)
                .into_font()
                .color(&FONT_COLOR),
        )
        .margin(LEGEND_MARGIN)
        .position(SeriesLabelPosition::UpperRight)
        .draw()
        .unwrap();

    root.present().unwrap();
}
