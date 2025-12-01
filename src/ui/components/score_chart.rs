use crate::logic::{calculate_plot_params, calculate_stats};
use crate::models::ItemData;
use eframe::egui;
use egui_plot::{Bar, BarChart, Corner, Legend, Plot};
use std::iter::zip;

/// グラフの描画
pub fn show(ui: &mut egui::Ui, item_data: &ItemData, selected_index: &mut Option<usize>) {
    let (avg, _, weights) = calculate_stats(&item_data.scores, item_data.decay_rate);
    let base_color = egui::Color32::from_rgb(65, 105, 225);

    let params = calculate_plot_params(&item_data.scores, &weights);
    let bar_base = params.bar_base;

    let mut boundaries = Vec::new(); // クリック判定用のバー範囲記録
    let mut current_x = 0.0; // 棒グラフの合計横幅記録用

    let bars = zip(item_data.scores.iter(), weights.iter())
        .enumerate()
        .map(|(i, (entry, &weight))| {
            let width = weight; // 重みがそのまま横幅となる

            let center_x = current_x + (width / 2.0);
            let bar = Bar::new(center_x, (entry.score as f64 - bar_base).max(0.0))
                .base_offset(bar_base)
                .width(width)
                .name(format!("{}回目", i + 1));

            let is_selected = *selected_index == Some(i);
            let fill_color = if is_selected {
                base_color // 選択時は濃く
            } else {
                base_color.gamma_multiply(0.4) // 通常は薄く
            };

            boundaries.push(current_x + weight);
            current_x += width;

            bar.fill(fill_color)
                .stroke(egui::Stroke::new(1.0, base_color))
        })
        .collect();

    let total_width = current_x;

    // 棒グラフデータ作成
    let plot_height = ui.available_height() * 0.6; // 画面の6割を使う
    let plot = Plot::new("score_plot")
        .height(plot_height)
        .legend(Legend::default().position(Corner::RightBottom))
        .x_axis_formatter(|_, _| String::new())
        .show_x(false)
        .allow_axis_zoom_drag(false)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false);

    let plot_response = plot.show(ui, |plot_ui| {
        // 棒グラフの描画
        plot_ui.bar_chart(
            BarChart::new("スコア", bars)
                .color(base_color)
                .highlight(false)
                .element_formatter(Box::new(|bar, _chart| {
                    format!("{}\nスコア: {:.1}", bar.name, bar.value)
                })),
        );

        // 平均線の描画
        let avg_line_data = vec![[0.0, avg], [total_width, avg]];
        plot_ui.line(
            egui_plot::Line::new("荷重平均", egui_plot::PlotPoints::new(avg_line_data))
                .color(egui::Color32::ORANGE)
                .style(egui_plot::LineStyle::Dashed { length: 10.0 })
                .highlight(false)
                .allow_hover(false),
        );

        // --- クリック検出ロジック ---
        // グラフがクリックされたか？
        if plot_ui.response().clicked() {
            // マウスカーソルの座標（Plot空間）を取得
            if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                // X座標がどのバーの範囲内にあるか探す
                // boundaries[i] は i番目のバーの「右端」の座標
                if pointer_pos.x >= 0.0 && pointer_pos.x <= total_width {
                    let clicked_index = boundaries.iter().position(|&end_x| end_x > pointer_pos.x);
                    if let Some(idx) = clicked_index {
                        return Some(idx); // クリックされたインデックスを返す
                    }
                }
            }
            // 範囲外クリックなら選択解除（Noneを返す）
            return Some(usize::MAX); // 特殊値: 解除用
        }
        None
    });

    // InnerResponse経由でクリック結果を受け取る
    if let Some(idx) = plot_response.inner {
        *selected_index = if idx == usize::MAX { None } else { Some(idx) };
    }
}
