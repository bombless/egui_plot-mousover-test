use eframe::egui;
use eframe::egui::{pos2, vec2, Rect, CornerRadius, StrokeKind};
use eframe::epaint::{PathShape, RectShape};
use egui::{Align2, Color32, Painter, Shape, Stroke, Vec2};
use std::collections::HashSet;
use std::{error::Error, f32::consts::PI};
use eframe::egui::Shape::LineSegment;

fn main() -> Result<(), Box<dyn Error>> {
    let app = App;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_title("主频轨迹浏览器（支持滚轮缩放/平移）"),
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

struct App;

fn softmax(values: impl Iterator<Item = f64> + Clone) -> Vec<f64> {
    let max = values.clone().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = values.map(|v| ((v - max) / 3.0).exp()).collect();
    let sum: f64 = exps.iter().sum();
    exps.iter().map(|&v| v / sum).collect()
}

mod color_gemini;

fn draw_pie_chart(painter: &Painter) {
    let radius = 20.0;

    let data = [
        ("C5", 523.2511306011972, 639.4707),
        ("C#5", 554.3652619537442, 1.5174463),
        ("B4", 493.8833012561241, 1.4178972),
        ("A#4", 466.1637615180899, 0.18944788),
        ("D5", 587.3295358348151, 0.13700715),
        ("A4", 440.0, 0.08572553),
        ("D#5", 622.2539674441618, 0.05192451),
        ("G#4", 415.3046975799451, 0.050112166),
        ("G4", 391.99543598174927, 0.033461154),
    ];

    let data = data.iter().map(|x| x.2).collect::<Vec<_>>();

    let total: f64 = data.iter().sum();
    // println!("total {total}");
    let mut start_angle = -PI / 2.0; // 从12点开始
    let center = pos2(50.0, 50.0);

    let soften = softmax(data.iter().cloned());

    println!("soften {soften:?}");

    let mut first = true;

    let mut global_min = f32::INFINITY;
    let mut global_max = -f32::INFINITY;
    let mut set = HashSet::new();
    let mut iter_color = color_gemini::ColorIterator::default();
    for offset in 0..2 {
        let sweep = soften[offset] as f32 * 2.0 * PI;
        // println!("sweep {sweep}");

        // 绘制扇形
        let mut points = vec![center];
        for i in 0..=20 {
            let angle = start_angle + sweep * (i as f32 / 20.0);
            let point = center + radius * Vec2::new(angle.cos(), angle.sin());
            set.insert(format!("{point}"));
            if first {
                println!(
                    "#up angle {angle} start_angle {start_angle} sweep {sweep} point {point:?} center {center:?} radius {radius:?}"
                );
            }
            points.push(point);
        }
        if first {
            first = false;
            println!(
                "#up set len {} {points:?}， {:?}",
                set.len(),
                points.iter().fold(f32::INFINITY, |acc, p| (acc.min(p.y)))
            );
        }

        global_max = global_max.max(points.iter().fold(-f32::INFINITY, |acc, p| acc.max(p.y)));
        global_min = global_min.min(points.iter().fold(f32::INFINITY, |acc, p| acc.min(p.y)));

        painter.add(Shape::convex_polygon(
            points,
            iter_color.next().unwrap(),
            Stroke::new(0.0, Color32::GOLD),
        ));

        start_angle += sweep;
        // break
    }
    println!(
        "#up(gold) global min {global_min} global max {global_max} diff {}",
        global_max - global_min
    );

    let mut start_angle = -PI / 2.0;

    let center = pos2(150.0, 150.0);

    let mut first = true;
    let mut global_min = f32::INFINITY;
    let mut global_max = -f32::INFINITY;
    let mut iter_color = color_gemini::ColorIterator::default();
    for (offset, value) in data.iter().enumerate() {
        let sweep = (value / total * 2.0 * (PI as f64)) as f32;

        // 绘制扇形
        let mut points = vec![center];
        for i in 0..=20 {
            let angle = start_angle + sweep * (i as f32 / 20.0);
            let point = center + radius * Vec2::new(angle.cos(), angle.sin());
            if first {
                println!("#down angle {angle} start_angle {start_angle} sweep {sweep} point {point:?} center {center:?} radius {radius:?}");
            }
            points.push(point);
        }
        if first {
            first = false;
            let min = points.iter().fold(f32::INFINITY, |acc, p| acc.min(p.y));
            let max = points.iter().fold(-f32::INFINITY, |acc, p| acc.max(p.y));
            let diff = max - min;
            // println!("#down {points:?} min {min} max {max} diff {diff:?}", );
        }

        global_max = global_max.max(points.iter().fold(-f32::INFINITY, |acc, p| acc.max(p.y)));
        global_min = global_min.min(points.iter().fold(f32::INFINITY, |acc, p| acc.min(p.y)));

        painter.add(Shape::convex_polygon(
            points,
            iter_color.next().unwrap(),
            Stroke::new(3.0, Color32::BLACK),
        ));

        start_angle += sweep;
    }
    // println!("#down(black) global min {global_min} global max {global_max} diff {}", global_max - global_min);
}

fn draw_pie_chart2(painter: &Painter) {
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
    ];

    let total: f64 = data.iter().map(|x| x.2).sum();
    let center = pos2(300.0, 220.0);

    let mut start_angle = -PI / 2.0;

    let mut color_iter = color_gemini::ColorIterator::default();

    let mut label_height_offset = 0.0;

    for (name, f, value) in data.iter() {
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

        if value / total >= 0.1 {
            let label = format!("{name:<5}{f:.1}Hz");
            let label_angle = start_angle + sweep / 2.0;
            let label_pos = center + (radius * 1.2) * Vec2::new(label_angle.cos(), label_angle.sin());
            painter.text(label_pos, Align2::CENTER_CENTER, label,
                         egui::FontId::default(), Color32::BLACK);
            println!("label_pos {label_pos} {name}");

        } else if value / total >= 0.005 {
            let label = format!("{name:<5}{f:.1}Hz");
            // let rect = Rect { min: pos2(10.0, 450.0 + label_height_offset), max: pos2(100.0, 450.0 + label_height_offset + 20.0) };
            // painter.add(RectShape::new(rect, CornerRadius::ZERO, color, Stroke::new(10.0, color), StrokeKind::Middle));
            painter.add(LineSegment {points: [pos2(10.0, 450.0 + label_height_offset), pos2(100.0, 450.0 + label_height_offset)], stroke: Stroke::new(10.0, color)});
            painter.text(pos2(110.0, 450.0 + label_height_offset), Align2::LEFT_CENTER, label, egui::FontId::default(), Color32::BLACK);
            label_height_offset += 20.0;
        }


        start_angle += sweep;
    }
}


fn draw_pie_chart_opus(painter: &Painter) {
    let radius = 150.0; // 减小半径，为图例留出空间
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
    ];

    let total: f64 = data.iter().map(|x| x.2).sum();
    let center = pos2(300.0, 250.0); // 上移中心点，为底部图例留出空间

    let mut start_angle = -PI / 2.0;
    let mut color_iter = color_gemini::ColorIterator::default();

    // 存储图例项（占比1%-10%的项目）
    let mut legend_items = Vec::new();

    for (name, f, value) in data.iter() {
        let percentage = (value / total * 100.0) as f32;

        // 跳过占比低于1%的项目
        if percentage < 1.0 {
            continue;
        }

        let sweep = (value / total * 2.0 * PI as f64) as f32;
        let color = color_iter.next().unwrap();

        // 绘制饼图扇形
        let mut points = vec![center];
        for i in 0..=20 {
            let angle = start_angle + sweep * (i as f32 / 20.0);
            let point = center + radius * Vec2::new(angle.cos(), angle.sin());
            points.push(point);
        }
        points.push(center);

        painter.add(PathShape::convex_polygon(
            points,
            color,
            Stroke::new(1.0, Color32::BLACK),
        ));

        // 根据占比决定显示方式
        if percentage >= 10.0 {
            // 占比>=10%：在饼图旁显示标签
            let label = format!("{name:<5}{f:.1} ({percentage:.1}%)", );
            let label_angle = start_angle + sweep / 2.0;
            let label_radius = radius * 1.3;
            let label_pos = center + label_radius * Vec2::new(label_angle.cos(), label_angle.sin());

            // 智能调整标签位置，避免重叠
            let align = if label_angle.cos() > 0.0 {
                Align2::LEFT_CENTER
            } else {
                Align2::RIGHT_CENTER
            };

            painter.text(
                label_pos,
                align,
                label,
                egui::FontId::default(),
                Color32::BLACK
            );

            // 添加引导线
            let line_start = center + radius * 0.9 * Vec2::new(label_angle.cos(), label_angle.sin());
            let line_mid = center + radius * 1.1 * Vec2::new(label_angle.cos(), label_angle.sin());
            painter.line_segment(
                [line_start, line_mid],
                Stroke::new(1.0, Color32::GRAY)
            );
        } else {
            // 占比1%-10%：添加到图例
            legend_items.push((name.to_string(), *f, percentage, color));
        }

        start_angle += sweep;
    }

    // 绘制底部图例
    if !legend_items.is_empty() {
        let legend_start_y = center.y + radius + 50.0;
        let legend_start_x = center.x - 150.0;
        let items_per_row = 3; // 每行显示3个图例项

        for (i, (name, f, percentage, color)) in legend_items.iter().enumerate() {
            let row = i / items_per_row;
            let col = i % items_per_row;

            let x = legend_start_x + (col as f32) * 120.0;
            let y = legend_start_y + (row as f32) * 25.0;

            // 绘制彩色方块
            let rect = egui::Rect::from_min_size(
                pos2(x, y - 5.0),
                vec2(10.0, 10.0)
            );
            painter.rect_filled(rect, 0.0, *color);

            // 绘制文字标签
            let label = format!("{name} {f:.1} ({percentage:.1}%)");
            painter.text(
                pos2(x + 15.0, y),
                Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(10.0),
                Color32::BLACK
            );
        }

        // 添加图例标题
        painter.text(
            pos2(center.x, legend_start_y - 20.0),
            Align2::CENTER_CENTER,
            "其他项目 (1%-10%)",
            egui::FontId::proportional(12.0),
            Color32::DARK_GRAY
        );
    }

    // 显示被过滤项目的信息
    let filtered_count = data.iter()
        .filter(|(_, _, value)| (value / total * 100.0) < 1.0)
        .count();

    if filtered_count > 0 {
        let info_text = format!("({filtered_count} 个项目占比低于1%，已隐藏)");
        painter.text(
            pos2(center.x, center.y + radius + 150.0),
            Align2::CENTER_CENTER,
            info_text,
            egui::FontId::proportional(10.0),
            Color32::GRAY
        );
    }
}


fn draw_pie_chart_gemini(painter: &Painter) {
    let radius = 150.0; // 缩小半径以便为下方的图例留出空间
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
    ];

    let total: f64 = data.iter().map(|x| x.2).sum();
    let center = pos2(300.0, 200.0); // 向上移动中心点，为图例留出空间

    let mut start_angle = -PI / 2.0;

    let mut color_iter = color_gemini::ColorIterator::default();

    // 用于存储需要显示在图例中的项目
    let mut legend_items: Vec<(&str, f64, Color32)> = Vec::new();

    for (name, _f, value) in data.iter() {
        let percentage = *value / total;
        let sweep = (percentage * 2.0 * PI as f64) as f32;
        let color = color_iter.next().unwrap_or(Color32::GRAY);

        // --- 绘制饼图扇区 (对所有项目都绘制) ---
        let mut points = vec![center];
        for i in 0..=20 {
            let angle = start_angle + sweep * (i as f32 / 20.0);
            let point = center + radius * Vec2::new(angle.cos(), angle.sin());
            points.push(point);
        }
        painter.add(PathShape::convex_polygon(
            points,
            color,
            Stroke::new(1.0, Color32::BLACK),
        ));

        // --- 根据百分比决定显示标签还是加入图例 ---
        if percentage >= 0.10 {
            // 占比 >= 10%，在饼图旁显示标签
            let label = format!("{} ({:.1}%)", name, percentage * 100.0);
            let label_angle = start_angle + sweep / 2.0;
            let label_pos =
                center + (radius * 1.25) * Vec2::new(label_angle.cos(), label_angle.sin());
            painter.text(
                label_pos,
                Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(14.0),
                Color32::BLACK,
            );
        } else if percentage >= 0.01 {
            // 占比 < 10% 但 >= 1%，加入图例列表
            legend_items.push((name, percentage, color));
        }
        // 占比 < 1% 的项目，自动忽略，不显示标签也不加入图例

        start_angle += sweep;
    }

    // --- 在图表下方绘制图例 ---
    if !legend_items.is_empty() {
        let mut legend_pos = pos2(center.x - radius, center.y + radius + 40.0);
        let swatch_size = vec2(12.0, 12.0);
        let row_height = 18.0;

        for (name, percentage, color) in legend_items {
            // 绘制颜色方块
            let swatch_rect = Rect::from_min_size(legend_pos, swatch_size);
            painter.rect_filled(swatch_rect, 0.0, color);

            // 绘制文字
            let text_pos = legend_pos + vec2(swatch_size.x + 5.0, swatch_size.y / 2.0);
            let label = format!("{} ({:.2}%)", name, percentage * 100.0);
            painter.text(
                text_pos,
                Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(12.0),
                Color32::BLACK,
            );

            // 移动到下一行
            legend_pos.y += row_height;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();

            // draw_pie_chart(painter);
            draw_pie_chart2(painter);
            // draw_pie_chart_opus(painter);
            // draw_pie_chart_gemini(painter);
            // exit(0);
        });
        // egui::SidePanel::left("left").resizable(true).min_width(70.0).max_width(300.0).default_width(100.0).show(ctx, |ui| {
        //     ScrollArea::horizontal().max_width(600.0).show(ui, |ui| {
        //         let painter = ui.painter();
        //         draw_pie_chart(painter, Pos2::new(35.0, 100.0), 30.0, &[
        //             ("A".to_string(), 1.0, Color32::RED),
        //             ("B".to_string(), 3.0, Color32::BROWN),
        //         ]);
        //
        //     });
        // });
        //
        // egui::SidePanel::right("right").resizable(true).min_width(70.0).max_width(300.0).default_width(100.0).show(ctx, |ui| {
        //     ScrollArea::horizontal().max_width(600.0).show(ui, |ui| {
        //         let painter = ui.painter();
        //         draw_pie_chart(painter, Pos2::new(570.0, 100.0), 30.0, &[
        //             ("A".to_string(), 1.0, Color32::RED),
        //             ("B".to_string(), 2.0, Color32::BROWN),
        //         ]);
        //
        //     });
        // });

        // Window::new("My Window").constrain(false).fade_in(true).collapsible(false).scroll([true, true]).show(ctx, |ui| {
        //     ui.label("Hello World!");
        //     let painter = ui.painter();
        //     let rect = ui.cursor().left_top();
        //     draw_pie_chart(painter);
        // });
    }
}
