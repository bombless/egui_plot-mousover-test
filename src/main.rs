use eframe::egui;
use eframe::egui::{pos2, Window, FontId, Pos2};
use eframe::epaint::PathShape;
use egui::{Align2, Color32, Painter, Stroke, Vec2};
use std::collections::{HashMap};
use std::{error::Error, f32::consts::PI};
use eframe::egui::Shape::LineSegment;
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, PlotUi};

fn main() -> Result<(), Box<dyn Error>> {

    let data: Vec<(f64, Vec<(String, f64)>)> = serde_json::from_slice(include_bytes!("../tones_track.json")).unwrap();
    let app = App {
        data: prepare_data(&data),
        raw_data: data,
        time: None,
        time2: None,
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_title("hover test"),
        ..Default::default()
    };

    eframe::run_native(
        "主频轨迹浏览器",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )?;

    Ok(())
}

// ========================== GUI 应用 ==========================

struct App {
    data: Vec<(String, Color32, Vec<[PlotPoint; 2]>)>,
    raw_data: Vec<(f64, Vec<(String, f64)>)>,
    time: Option<f64>,
    time2: Option<f64>,
}

mod color_gemini;


fn draw_pie_chart(painter: &Painter, show_data: Option<&[(&str, f64)]>) {
    let radius = 200.0;
    let data = [
        ("C#5", 554.3652619537442, 77077.125),
        ("C5", 523.2511306011972, 34012.4),
        ("D5", 587.3295358348151, 13019.165),
        ("C#6", 1108.7305239074883, 1211.661),
        ("B4", 493.8833012561241, 833.9814),
        ("A#4", 466.1637615180899, 685.5318),
        ("G#6", 1661.2187903197805, 648.7387),
        ("C#7", 2217.4610478149766, 273.14865),
        ("D6", 1174.6590716696303, 136.46376),
        ("D#5", 622.2539674441618, 133.80542),
        ("C3", 130.8127826502993, 86.437386),
        ("A4", 440.0, 85.93467),
        ("D3", 146.8323839587038, 64.65748),
        ("A2", 110.0, 31.289621),
        ("E5", 659.2551138257398, 24.001055),
    ].iter().map(|&(x, _, y)| (x, y)).collect::<Vec<_>>();



    let data_source = if let Some(data) = show_data {
        data
    } else {
        &data[..]
    };

    let total: f64 = data_source.iter().map(|x| x.1).sum();
    let center = pos2(310.0, 220.0);

    let mut start_angle = -PI / 2.0;

    let mut color_iter = color_gemini::ColorIterator::default();

    let mut label_height_offset = 0.0;

    let momo_font = FontId::monospace(10.0);

    for (name, value) in data_source {
        let sweep = (value / total * 2.0 * PI as f64) as f32;

        let mut points = vec![center];
        for i in 0..=30 {
            let angle = start_angle + sweep * (i as f32 / 30.0);
            let point = center + radius * Vec2::new(angle.cos(), angle.sin());
            points.push(point);
        }
        points.push(center); // 闭合路径

        let color = color_iter.next().unwrap();

        painter.add(PathShape::convex_polygon(
            points,
            color,
            Stroke::new(1.0, Color32::BLACK),
        ));

        if value / total >= 0.05 {
            let label = format!("{name:<3} {value:>6.1}");
            let label_angle = start_angle + sweep / 2.0;
            let label_pos = center + (radius * 1.2) * Vec2::new(label_angle.cos(), label_angle.sin());
            painter.text(label_pos, Align2::CENTER_CENTER, label, momo_font.clone(), color);

        } else {
            let label = format!("{name:<3} {value:>6.1}");

            painter.add(LineSegment {points: [pos2(10.0, 480.0 + label_height_offset), pos2(100.0, 480.0 + label_height_offset)], stroke: Stroke::new(10.0, color)});
            painter.text(pos2(110.0, 480.0 + label_height_offset), Align2::LEFT_CENTER, label, momo_font.clone(), color);
            label_height_offset += 20.0;
        }


        start_angle += sweep;
    }
}

fn prepare_data(data: &[(f64, Vec<(String, f64)>)]) -> Vec<(String, Color32, Vec<[PlotPoint; 2]>)> {
    let mut container = HashMap::new();
    let mut iter_color = color_gemini::ColorIterator::default();
    let mut tone_to_color = HashMap::new();
    let order = {
        let mut container = HashMap::new();
        for (_, candidates) in data {
            for (tone, w) in candidates {
                *container.entry(tone).or_insert(0.0) += w;
            }
        }
        let mut data = container.drain().collect::<Vec<_>>();
        data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let mut order_container = HashMap::new();
        for (sort, (name, _)) in data.iter().enumerate() {
            order_container.insert(*name, sort);
        }
        order_container
    };
    for (t, candidates) in data {
        let mut acc = 0.0;
        let mut ordered_weights = candidates.iter().collect::<Vec<_>>();
        ordered_weights.sort_by(|(a, _), (b, _)| order[a].cmp(&order[b]));
        for (tone, weight) in ordered_weights {
            let color = tone_to_color.entry(tone).or_insert_with(|| iter_color.next().unwrap());

            container
                .entry(tone.clone())
                .or_insert_with(|| (*color, Vec::new()))
                .1.push([PlotPoint {x: *t, y: acc}, PlotPoint {x: *t, y: acc + weight}]);
            acc += weight;
        }
    }
    container.into_iter().map(|(note, (color, data))| (note, color, data)).collect()
}

fn draw_graph2<'a, 'b: 'a>(plot_ui: &mut PlotUi<'a>, data: &'b [(String, Color32, Vec<[PlotPoint; 2]>)]) {

    for (note, color, candidates) in data {
        for point_pair in candidates {
            let points_data = PlotPoints::from(&point_pair[..]);
            plot_ui.line(Line::new(note, points_data).color(*color));
        }
    }

}

fn draw_graph(plot_ui: &mut PlotUi) {

    let data: Vec<(f64, Vec<(String, f64)>)> = serde_json::from_slice(include_bytes!("../tones_track.json")).unwrap();
    let mut iter_color = color_gemini::ColorIterator::default();
    let mut tone_to_color = HashMap::new();
    for (t, candidates) in data {
        let mut acc = 0.0;
        for (tone, weight) in candidates {
            let color = tone_to_color.entry(tone).or_insert_with(|| iter_color.next().unwrap());
            let points_data = PlotPoints::new(vec![
                [t, acc], [t, acc + weight]
            ]);
            acc += weight;
            plot_ui.line(Line::new("", points_data).color(*color));
        }

    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            // draw_pie_chart(painter);
            if let Some(time) = self.time.or(self.time2) {
                if let Some((_, data)) = self.raw_data.iter().find(|(t, ..)| time < *t) {
                    ui.painter().text(Pos2::ZERO, Align2::LEFT_TOP, time.to_string(), FontId::default(), Color32::BLACK);
                    draw_pie_chart(painter, Some(&data.iter().map(|(n, w)| (&**n, *w)).collect::<Vec<_>>()));
                } else {
                    draw_pie_chart(painter, None);
                }
            } else {
                draw_pie_chart(painter, None);
            }

            // draw_pie_chart_opus(painter);
            // draw_pie_chart_gemini(painter);
            // exit(0);
        });
        Window::new("draw_graph").fade_in(true).default_open(false).collapsible(true).scroll([true, true]).show(ctx, |ui| {

            Plot::new("plot-draw_graph").show(ui, |plot_ui| {
                draw_graph(plot_ui);
                self.time = plot_ui.pointer_coordinate().map(|p| p.x);
            });
        });

        Window::new("draw_graph2").pivot(Align2::RIGHT_TOP).fade_in(true).collapsible(false).scroll([true, true]).show(ctx, |ui| {

            Plot::new("plot-draw_graph2").show(ui, |plot_ui| {
                draw_graph2(plot_ui, &self.data);
                self.time2 = plot_ui.pointer_coordinate().map(|p| p.x);
            });
        });
    }
}
