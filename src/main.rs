use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use eframe::emath::Vec2b;

fn main() {
    // Create the application instance
    let app = App;

    // Configure native window options
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0]) // Set window size
            .with_title("test"),               // Set window title
        ..Default::default()
    };

    // Run the native eframe application
    eframe::run_native(
        "test",
        native_options,
        Box::new(|_cc| {
            Ok(Box::new(app))
        }),
    ).unwrap();
}

struct App;

// Constants for time and frequency axis bounds
const TIME_BOUNDS: (f64, f64) = (0.0, 100.0);
const FREQ_BOUNDS: (f64, f64) = (0.0, 100.0);

impl App {
    fn draw_plot(&self, ui: &mut egui::Ui) {
        // Create a plot with specified interaction settings
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

            // Rectangle position: at the top of the plot
            let rect_y_center = bounds.max()[1] - 0.05 * y_span;
            let rect_height = 0.8 * y_span;
            let rect_width = beat_duration * 0.8;

            // Rectangle corners
            let rect_x_min = 0.0;
            let rect_x_max = 0.0 + rect_width;
            let rect_y_max = rect_y_center + rect_height / 2.0;

            // Draw top edge of the rectangle
            plot_ui.line(
                Line::new(
                    "PlotPoints",
                    PlotPoints::from_iter(vec![
                        [rect_x_min, rect_y_max],
                        [rect_x_max, rect_y_max],
                    ]),
                )
                    .width(10.0),
            );
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show the plot in the central panel
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_plot(ui);
        });
    }
}
