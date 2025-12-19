use crate::constants::BAR_BASE_COLOR;
use crate::logic::{PlotParams, calculate_plot_params, calculate_stats};
use crate::models::app::ScoreEntry;
use crate::utils::comma_display::CommaDisplay;
use eframe::egui;
use egui_plot::{Bar, BarChart, Corner, Legend, Plot};
use std::iter::zip;

pub struct WeightedScoreChart;

impl WeightedScoreChart {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        scores: &[ScoreEntry],
        decay_rate: f64,
        selected_index: &mut Option<usize>,
        scroll_req_index: &mut Option<usize>,
    ) {
        // 統計計算
        let (avg, _, _, weights) = calculate_stats(scores, decay_rate);
        let params = calculate_plot_params(scores, &weights);

        // バーとクリック判定境界の作成
        let (bars, boundaries, total_width) =
            self.create_bars(scores, &weights, &params, *selected_index);

        // プロット、クリック処理
        let clicked_idx = self.draw_plot(ui, bars, &boundaries, avg, total_width);

        // クリック結果
        if let Some(idx) = clicked_idx {
            *selected_index = (idx != usize::MAX).then_some(idx);
            *scroll_req_index = (idx != usize::MAX).then_some(idx);
        }
    }

    fn create_bars(
        &self,
        scores: &[ScoreEntry],
        weights: &[f64],
        params: &PlotParams,
        selected_index: Option<usize>,
    ) -> (Vec<Bar>, Vec<f64>, f64) {
        let mut boundaries = Vec::new(); // クリック判定用のバー範囲記録
        let mut current_x = 0.0; // 棒グラフの合計横幅記録用

        let base_color = BAR_BASE_COLOR; // バーカラー

        let bar_base = params.bar_base;
        let bars = zip(scores.iter(), weights.iter())
            .enumerate()
            .map(|(i, (entry, &weight))| {
                let width = weight; // 重みがそのまま横幅
                let height = entry.score as f64 - bar_base; // bar_baseからの高さ

                let center_x = current_x + (width / 2.0);

                let is_selected = selected_index == Some(i);
                let bar_color = if is_selected {
                    base_color // 選択時は濃く
                } else {
                    base_color.gamma_multiply(0.4) // 通常は薄く
                };

                let bar = Bar::new(center_x, height.max(0.0))
                    .base_offset(bar_base)
                    .width(width)
                    .name(format!("{}回目", i + 1))
                    .fill(bar_color)
                    .stroke(egui::Stroke::new(1.0, base_color));

                boundaries.push(current_x + weight);
                current_x += width;

                bar
            })
            .collect::<Vec<Bar>>();

        (bars, boundaries, current_x)
    }

    fn draw_plot(
        &self,
        ui: &mut egui::Ui,
        bars: Vec<Bar>,
        boundaries: &[f64],
        avg: f64,
        total_width: f64,
    ) -> Option<usize> {
        let mut clicked_bar_idx = None;

        let plot_height = ui.available_height() * 0.6; // 画面の縦幅6割を使用
        let plot = Plot::new("score_plot")
            .height(plot_height) // 固定の高さ
            .legend(Legend::default().position(Corner::RightBottom))
            .show_axes([false, true])
            .show_x(false)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false);

        plot.show(ui, |plot_ui| {
            // 棒グラフ
            plot_ui.bar_chart(
                BarChart::new("スコア", bars)
                    .highlight(false)
                    .color(BAR_BASE_COLOR) // 凡例のカラー設定
                    .element_formatter(Box::new(|bar, _| {
                        format!(
                            "{}\nスコア: {}",
                            bar.name,
                            (bar.value + bar.base_offset.unwrap_or(0.0)).to_comma_fmt(0)
                        )
                    })),
            );

            // 平均線
            let line_points = vec![[0.0, avg], [total_width, avg]];
            plot_ui.line(
                egui_plot::Line::new("荷重平均", egui_plot::PlotPoints::new(line_points))
                    .color(egui::Color32::ORANGE)
                    .style(egui_plot::LineStyle::Dashed { length: 10.0 })
                    .highlight(false)
                    .allow_hover(false),
            );

            // クリック検出
            let clicked = plot_ui.response().clicked();
            if clicked && let Some(pos) = plot_ui.pointer_coordinate() {
                // グラフ範囲内クリックなら、どのバーかを探す
                if (0.0..=total_width).contains(&pos.x) {
                    clicked_bar_idx = find_clicked_bar(pos.x, boundaries);
                } else {
                    // 範囲外クリックで選択解除
                    clicked_bar_idx = Some(usize::MAX);
                }
            }
        });

        clicked_bar_idx
    }
}

/// x座標がどのバーに属するか判定
fn find_clicked_bar(x: f64, boundaries: &[f64]) -> Option<usize> {
    // クリック場所が負の場合は範囲外確定
    if x < 0.0 {
        return Some(usize::MAX); // 選択解除
    }

    boundaries.iter().position(|&end_x| x < end_x)
}
