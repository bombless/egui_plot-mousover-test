use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use eframe::emath::Vec2b;

fn main() {

    let app = App;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("test"),
        ..Default::default()
    };

    eframe::run_native(
        "test",
        native_options,
        Box::new(|_cc| {
            Ok(Box::new(app))
        }),
    ).unwrap();
}

struct App;
const TIME_BOUNDS : (f64, f64) = (0.0, 100.0);
const FREQ_BOUNDS : (f64, f64) = (0.0, 100.0);

impl App {
    fn draw_plot(&self, ui: &mut egui::Ui) {
        let plot = Plot::new("dominant_freq_plot")
            .allow_scroll(false)
            .allow_zoom(true)
            .allow_boxed_zoom(true)
            .allow_drag(true)
            .default_x_bounds(TIME_BOUNDS.0, TIME_BOUNDS.1)
            .default_y_bounds(FREQ_BOUNDS.0, FREQ_BOUNDS.1)
            .auto_bounds(Vec2b::new(true, true));

        plot.show(ui, |plot_ui| {
            let beat_duration = 1f64;
            let bounds = plot_ui.plot_bounds();
            let y_span = FREQ_BOUNDS.1 - FREQ_BOUNDS.0;

            let start_beat = (bounds.min()[0] / beat_duration).floor() as i32;
            let end_beat = (bounds.max()[0] / beat_duration).ceil() as i32;

            for beat_num in start_beat..=end_beat {
                if beat_num < 0 {
                    continue;
                }

                let beat_time = beat_num as f64 * beat_duration;

                if beat_time > 100.0 {
                    break;
                }


                // 矩形位置：在图表顶部
                let rect_y_center = bounds.max()[1] - 0.05 * y_span;
                let rect_height = 0.8 * y_span;
                let rect_width = beat_duration * 0.8;

                // 矩形的四个角
                let rect_x_min = beat_time;
                let rect_x_max = beat_time + rect_width;
                let rect_y_max = rect_y_center + rect_height / 2.0;




                // 上边框
                plot_ui.line(Line::new("PlotPoints", PlotPoints::from_iter(vec![
                    [rect_x_min, rect_y_max],
                    [rect_x_max, rect_y_max],
                ])).width(10.0));


            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_plot(ui);
        });
    }
}