use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints, VLine};
use egui::Color32;
use std::cell::Cell;
use eframe::emath::Vec2b;

fn main() {

    let app = App::new(
        300.0,
        100.0,
        300.0,
    );

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

struct App {
    duration: f64,
    time_bounds: (Cell<f64>, Cell<f64>),
    freq_bounds: (Cell<f64>, Cell<f64>),

    bpm: f64,
    show_beat_lines: bool,
    beats_per_bar: usize,
    show_beat_notes: bool,  // 新增：是否显示节拍音符标注

}

impl App {
    fn new(
        duration: f64,
        fmax: f64,
        _total_duration: f64,
    ) -> Self {
        Self {
            duration,
            time_bounds: (Cell::new(0.0), Cell::new(duration.max(1e-6))),
            freq_bounds: (Cell::new(0.0), Cell::new(fmax.max(1.0))),
            bpm: 120.0,
            show_beat_lines: true,
            beats_per_bar: 4,
            show_beat_notes: true,  // 默认显示节拍音符
        }
    }

    fn draw_plot(&self, ui: &mut egui::Ui) {
        let plot = Plot::new("dominant_freq_plot")
            .legend(Legend::default())
            .allow_scroll(false)
            .allow_zoom(true)
            .allow_boxed_zoom(true)
            .allow_drag(true)
            .default_x_bounds(self.time_bounds.0.get(), self.time_bounds.1.get())
            .default_y_bounds(self.freq_bounds.0.get(), self.freq_bounds.1.get())
            .auto_bounds(Vec2b::new(true, true));

        plot.show(ui, |plot_ui| {
            // 节拍线和音符标注
            if self.show_beat_lines && self.bpm > 0.0 {
                let beat_duration = 60.0 / self.bpm;
                let bounds = plot_ui.plot_bounds();
                let y_span = self.freq_bounds.1.get() - self.freq_bounds.0.get();

                let start_beat = (bounds.min()[0] / beat_duration).floor() as i32;
                let end_beat = (bounds.max()[0] / beat_duration).ceil() as i32;

                for beat_num in start_beat..=end_beat {
                    if beat_num < 0 {
                        continue;
                    }

                    let beat_time = beat_num as f64 * beat_duration;

                    if beat_time > self.duration {
                        break;
                    }

                    let is_bar_start = beat_num as usize % self.beats_per_bar == 0;

                    // 绘制节拍线
                    let (color, width) = if is_bar_start {
                        (Color32::from_rgb(0, 100, 200), 2.0)
                    } else {
                        (Color32::from_rgba_unmultiplied(100, 150, 255, 150), 1.0)
                    };

                    let beat_line = VLine::new("beat_time", beat_time)
                        .color(color)
                        .width(width);
                    plot_ui.vline(beat_line);

                    // 绘制节拍音符标注框
                    if self.show_beat_notes {
                        // if let Some((_, note_name, note_freq, is_strong)) = beat_notes.iter().take(1)
                        //     .find(|(t, _, _, _)| (*t - beat_time).abs() < beat_duration * 0.1) {

                            // 矩形位置：在图表顶部
                            let rect_y_center = bounds.max()[1] - 0.05 * y_span;
                            let rect_height = 0.08 * y_span;
                            let rect_width = beat_duration * 0.8;

                            // 矩形的四个角
                            let rect_x_min = beat_time;
                            let rect_x_max = beat_time + rect_width;
                            let rect_y_max = rect_y_center + rect_height / 2.0;

                            let border_width = 1f32;

                            let rect_x_min = rect_x_min.min(self.time_bounds.1.get());
                            let rect_x_max = rect_x_max.max(self.time_bounds.0.get());
                            let rect_y_max = rect_y_max.max(self.freq_bounds.0.get()) - border_width as f64;


                            // 上边框
                            plot_ui.line(Line::new("PlotPoints", PlotPoints::from_iter(vec![
                                [rect_x_min, rect_y_max],
                                [rect_x_max, rect_y_max],
                            ])).width(border_width));


                        // }
                    }

                }
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