use eframe::egui;
use egui::{Align2, Color32, Window, FontId, Pos2};
use std::error::Error;
use egui_plot::{Line, Plot};

fn main() -> Result<(), Box<dyn Error>> {

    let app = App {
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
    time: Option<f64>,
    time2: Option<f64>,
}


impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Window::new("first window").pivot(Align2::LEFT_BOTTOM).fade_in(true).default_open(false).collapsible(true).scroll([true, true]).show(ctx, |ui| {

            let within = ui.rect_contains_pointer(ui.available_rect_before_wrap());
            Plot::new("first plot").show(ui, |plot_ui| {
                plot_ui.line(Line::new("", vec![[0.0, 9.0], [0.0, 999.0]]));
                self.time = if within { plot_ui.pointer_coordinate().map(|p| p.x) } else { None };
            });
        });

        Window::new("second window").pivot(Align2::RIGHT_BOTTOM).fade_in(true).collapsible(false).scroll([true, true]).show(ctx, |ui| {

            let within = ui.rect_contains_pointer(ui.available_rect_before_wrap());
            Plot::new("second plot").show(ui, |plot_ui| {
                plot_ui.line(Line::new("", vec![[0.0, 9.0], [0.0, 999.0]]));
                self.time2 = if within { plot_ui.pointer_coordinate().map(|p| p.x) } else { None };
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.painter().text(Pos2::ZERO, Align2::LEFT_TOP, format!("time1 {:?}\ntime2 {:?}", self.time, self.time2), FontId::default(), Color32::BLACK);
        });
    }
}
