use eframe::egui;
use egui_plot::{Plot, PlotPoint, Text as PlotText};
use eframe::egui::Align;
use eframe::emath::{Align2, Vec2b};

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

impl App {


    fn draw_plot(ui: &mut egui::Ui) {
        let plot = Plot::new("dominant_freq_plot")
            .allow_scroll(false)
            .allow_zoom(true)
            .allow_boxed_zoom(true)
            .allow_drag(true)
            .default_x_bounds(0.0, 1.0)
            .default_y_bounds(-100.0, 100.0)
            .auto_bounds(Vec2b::new(true, true));

        plot.show(ui, |plot_ui| {
            // 节拍线和音符标注
                let beat_duration = 1.0;
                let bounds = plot_ui.plot_bounds();
                let y_span = bounds.max()[1] - bounds.min()[1];

                let start_beat = (bounds.min()[0] / beat_duration).floor() as i32;
                let end_beat = (bounds.max()[0] / beat_duration).ceil() as i32;

                // println!("start_beat {start_beat}\nend_beat {end_beat}\nbeat_duration {beat_duration}\nduration {}", self.duration);

                for beat_num in start_beat..=end_beat {
                    if beat_num < 0 {
                        continue;
                    }

                    let beat_time = beat_num as f64 * beat_duration;

                    if beat_time > 60.0 {
                        break;
                    }



                    // 小节编号标签（在底部）
                        let label_y = bounds.min()[1] + 0.02 * y_span;
                        plot_ui.text(
                            PlotText::new("beats", PlotPoint { x: beat_time, y: label_y.max(0.0) }, beat_num.to_string())
                                .anchor(Align2([Align::Center, Align::Max])),
                        );
                }


            // 鼠标坐标提示
            if let Some(pointer) = plot_ui.pointer_coordinate() {
                let txt = format!("{pointer:?}");
                plot_ui.text(
                    PlotText::new("鼠标坐标提示", PlotPoint {x: pointer.x, y: pointer.y}, txt)
                        .anchor(Align2([Align::Min, Align::Center]))
                );
            }
        });
    }
}


impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {
            App::draw_plot(ui);
        });
    }
}