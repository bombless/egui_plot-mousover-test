use eframe::egui;
use egui_plot::{Plot, PlotPoint, Text as PlotText};

fn main() {
    let app = App;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_title("mouseover test"),
        ..Default::default()
    };

    eframe::run_native("test", native_options, Box::new(|_cc| Ok(Box::new(app)))).unwrap();
}

struct App;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let plot = Plot::new("test")
                .allow_scroll(true)
                .allow_zoom(true)
                .allow_boxed_zoom(true)
                .allow_drag(true)
                .include_x(0.0)
                .include_x(0.1);

            plot.show(ui, |plot_ui| {
                let bounds = plot_ui.plot_bounds();
                let val = bounds.min()[0];
                plot_ui.text(PlotText::new(
                    "",
                    PlotPoint { x: val, y: 0.0 },
                    val.to_string(),
                ));
            });
        });
    }
}
