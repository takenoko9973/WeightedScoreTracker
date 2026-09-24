use crate::constants::BAR_BASE_COLOR;
use crate::domain::ScoreEntry;
use crate::logic::{StatsPoint, calculate_plot_params, calculate_stats, calculate_stats_series};
use crate::utils::comma_display::CommaDisplay;
use eframe::egui;
use egui_plot::{
    Bar, BarChart, Corner, Legend, Plot, PlotBounds, PlotMemory, PlotResponse, PlotTransform,
    PlotUi, Polygon,
};
use serde::{Deserialize, Serialize};
use std::iter::zip;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum WeightedAverageMode {
    Hidden,
    Current,
    History,
}

impl WeightedAverageMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Hidden => "非表示",
            Self::Current => "現在値",
            Self::History => "推移",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct ChartSettings {
    pub(crate) average_mode: WeightedAverageMode,
    pub(crate) show_std_band: bool,
}

impl Default for ChartSettings {
    fn default() -> Self {
        Self {
            average_mode: WeightedAverageMode::Current,
            show_std_band: false,
        }
    }
}

struct PlotOverlays<'a> {
    mean_points: &'a [[f64; 2]],
    band_polygons: &'a [Vec<[f64; 2]>],
    min_y: f64,
    max_y: f64,
}

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
        settings: ChartSettings,
        reset_bounds: bool,
        selected_index: &mut Option<usize>,
        scroll_req_index: &mut Option<usize>,
    ) {
        // 統計計算
        let (avg, std, _, weights) = calculate_stats(scores, decay_rate);
        let current_stats = StatsPoint { mean: avg, std };
        let history_stats = if settings.average_mode == WeightedAverageMode::History {
            calculate_stats_series(scores, decay_rate)
        } else {
            Vec::new()
        };
        let params = calculate_plot_params(scores, &weights);

        // バーとクリック判定境界の作成
        let (bars, boundaries, centers) = self.create_bars(scores, &weights, *selected_index);
        let mean_points = mean_line_points(
            settings.average_mode,
            &centers,
            &weights,
            current_stats,
            &history_stats,
        );
        let band_polygons = standard_deviation_band_polygons(
            settings.average_mode,
            &centers,
            &weights,
            current_stats,
            &history_stats,
            settings.show_std_band,
        );
        let overlays = PlotOverlays {
            mean_points: &mean_points,
            band_polygons: &band_polygons,
            min_y: params.min_y,
            max_y: params.max_y,
        };

        // プロット、クリック処理
        let clicked_idx = self.draw_plot(ui, bars, &boundaries, overlays, reset_bounds);

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
        selected_index: Option<usize>,
    ) -> (Vec<Bar>, Vec<f64>, Vec<f64>) {
        let mut boundaries = Vec::new(); // クリック判定用のバー範囲記録
        let mut current_x = 0.0; // 棒グラフの合計横幅記録用
        let centers = bar_centers(weights);

        let base_color = BAR_BASE_COLOR; // バーカラー

        // let bar_base = params.bar_base;
        let bars = zip(scores.iter(), weights.iter())
            .enumerate()
            .map(|(i, (entry, &weight))| {
                let width = weight; // 重みがそのまま横幅
                let height = entry.score as f64;

                let center_x = centers[i];

                let is_selected = selected_index == Some(i);
                let bar_color = if is_selected {
                    base_color // 選択時は濃く
                } else {
                    base_color.gamma_multiply(0.4) // 通常は薄く
                };

                let bar = Bar::new(center_x, height.max(0.0))
                    .width(width)
                    .name(format!("{}回目", i + 1))
                    .fill(bar_color)
                    .stroke(egui::Stroke::new(1.0_f32, base_color));

                boundaries.push(current_x + weight);
                current_x += width;

                bar
            })
            .collect::<Vec<Bar>>();

        (bars, boundaries, centers)
    }

    fn draw_plot(
        &self,
        ui: &mut egui::Ui,
        bars: Vec<Bar>,
        boundaries: &[f64],
        overlays: PlotOverlays<'_>,
        reset_bounds: bool,
    ) -> Option<usize> {
        let plot_height = ui.available_height() * 0.6; // 画面の縦幅6割を使用
        let plot_id = ui.make_persistent_id("score_plot");
        let uses_default_y_bounds = overlays.min_y < overlays.max_y;

        if !uses_default_y_bounds {
            restore_fallback_y_bounds(
                ui.ctx(),
                plot_id,
                overlays.min_y,
                overlays.max_y,
                reset_bounds,
            );
        }

        let mut plot = Plot::new("score_plot")
            .id(plot_id)
            .height(plot_height) // 固定の高さ
            .legend(Legend::default().position(Corner::RightBottom))
            .show_axes([false, true])
            .show_x(false)
            .allow_axis_zoom_drag([false, true])
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_double_click_reset(false)
            .auto_bounds(egui::Vec2b::new(true, false));

        if uses_default_y_bounds {
            // Y軸の復帰先はスコア由来の既存計算式だけに限定する。
            plot = plot.default_y_bounds(overlays.min_y, overlays.max_y);
        } else {
            // 空履歴・全同値では default_y_bounds が使えないため、従来の経路を保つ。
            plot = plot.include_y(overlays.max_y).include_y(overlays.min_y);
        }

        let total_width = bars.iter().map(|bar| bar.bar_width).sum();
        let plot_response = plot.show(ui, |plot_ui| {
            if reset_bounds && uses_default_y_bounds {
                plot_ui.set_auto_bounds(egui::Vec2b::new(true, true));
            }
            self.show_standard_deviation_band(plot_ui, overlays.band_polygons);
            self.show_bars(plot_ui, bars);
            self.show_average_line(plot_ui, overlays.mean_points);
        });

        let reset_button_response = PlotMemory::load(ui.ctx(), plot_id)
            .filter(|memory| {
                if uses_default_y_bounds {
                    !memory.auto_bounds.y
                } else {
                    fallback_y_bounds_are_manual(memory, overlays.min_y, overlays.max_y)
                }
            })
            .map(|_| {
                let button_position =
                    plot_response.transform.frame().left_top() + egui::vec2(4.0, 4.0);
                egui::Area::new(ui.make_persistent_id("score_plot_y_reset"))
                    .fixed_pos(button_position)
                    .order(egui::Order::Foreground)
                    .movable(false)
                    .enabled(ui.is_enabled())
                    .show(ui.ctx(), |ui| {
                        let response = ui
                            .add(egui::Button::new("↺"))
                            .on_hover_text("Y軸を自動調整に戻す");
                        response.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::WidgetType::Button,
                                ui.is_enabled(),
                                "Y軸を自動調整に戻す",
                            )
                        });
                        response
                    })
                    .inner
            });

        let reset_button_clicked = reset_button_response
            .as_ref()
            .is_some_and(egui::Response::clicked);
        let button_blocks_plot = reset_button_response
            .as_ref()
            .is_some_and(egui::Response::contains_pointer);

        if reset_button_clicked && let Some(mut memory) = PlotMemory::load(ui.ctx(), plot_id) {
            memory.auto_bounds.y = true;
            memory.store(ui.ctx(), plot_id);
            ui.ctx().request_repaint();
        }

        (!button_blocks_plot)
            .then(|| self.check_click(&plot_response, boundaries, total_width))
            .flatten()
    }

    /// 棒グラフ描画
    fn show_bars(&self, plot_ui: &mut PlotUi, bars: Vec<Bar>) {
        plot_ui.bar_chart(
            BarChart::new("スコア", bars)
                .highlight(false)
                .color(BAR_BASE_COLOR) // 凡例のカラー設定
                .element_formatter(Box::new(|bar, _| {
                    format!("{}\nスコア: {}", bar.name, bar.value.to_comma_fmt(0))
                })),
        );
    }

    /// 加重標準偏差帯描画
    fn show_standard_deviation_band(&self, plot_ui: &mut PlotUi, band_polygons: &[Vec<[f64; 2]>]) {
        let band_color = egui::Color32::ORANGE.linear_multiply(0.2);

        for (index, points) in band_polygons.iter().enumerate() {
            let name = if index == 0 { "加重標準偏差" } else { "" };
            plot_ui.polygon(
                Polygon::new(name, points.clone())
                    .id(egui::Id::new(("weighted_std_band", index)))
                    .fill_color(band_color)
                    .stroke(egui::Stroke::new(1.0, band_color))
                    .highlight(false)
                    .allow_hover(false),
            );
        }
    }

    /// 平均線描画
    fn show_average_line(&self, plot_ui: &mut PlotUi, mean_points: &[[f64; 2]]) {
        if mean_points.is_empty() {
            return;
        }

        plot_ui.line(
            egui_plot::Line::new("加重平均", egui_plot::PlotPoints::new(mean_points.to_vec()))
                .color(egui::Color32::ORANGE)
                .style(egui_plot::LineStyle::Dashed { length: 10.0 })
                .highlight(false)
                .allow_hover(false),
        );
    }

    /// クリック判定
    fn check_click(
        &self,
        plot_response: &PlotResponse<()>,
        boundaries: &[f64],
        width: f64,
    ) -> Option<usize> {
        if plot_response.response.clicked()
            && let Some(position) = plot_response.response.interact_pointer_pos()
        {
            let pos = plot_response.transform.value_from_position(position);
            // グラフ範囲内のクリックなら、どのバーかを探す
            if (0.0..=width).contains(&pos.x) {
                return find_clicked_bar(pos.x, boundaries);
            } else {
                // 範囲外クリックで選択解除
                return Some(usize::MAX);
            }
        }
        None
    }
}

fn restore_fallback_y_bounds(
    ctx: &egui::Context,
    plot_id: egui::Id,
    min_y: f64,
    max_y: f64,
    reset_bounds: bool,
) {
    let Some(mut memory) = PlotMemory::load(ctx, plot_id) else {
        return;
    };

    if !reset_bounds && !memory.auto_bounds.y {
        return;
    }

    // include_y と同じ無効なY範囲を渡し、egui_plotの従来の初期化処理で表示範囲を復帰する。
    let bounds = fallback_y_bounds(*memory.bounds(), *memory.transform().frame(), min_y, max_y);
    memory.set_bounds(bounds);
    memory.auto_bounds.x = reset_bounds || memory.auto_bounds.x;
    memory.auto_bounds.y = false;
    memory.store(ctx, plot_id);
}

fn fallback_y_transform(frame: egui::Rect, min_y: f64, max_y: f64) -> PlotTransform {
    let bounds = PlotBounds::from_min_max([f64::INFINITY, min_y], [f64::NEG_INFINITY, max_y]);
    PlotTransform::new(frame, bounds, false)
}

fn fallback_y_bounds(
    current_bounds: PlotBounds,
    frame: egui::Rect,
    min_y: f64,
    max_y: f64,
) -> PlotBounds {
    let mut bounds = current_bounds;
    bounds.set_y(fallback_y_transform(frame, min_y, max_y).bounds());
    bounds
}

fn fallback_y_bounds_are_manual(memory: &PlotMemory, min_y: f64, max_y: f64) -> bool {
    !memory.auto_bounds.y
        && memory.bounds().range_y()
            != fallback_y_transform(*memory.transform().frame(), min_y, max_y)
                .bounds()
                .range_y()
}

/// 重みを既存バーの中心座標へ変換する。
fn bar_centers(weights: &[f64]) -> Vec<f64> {
    let mut current_x = 0.0;

    weights
        .iter()
        .map(|&width| {
            let center_x = current_x + (width / 2.0);
            current_x += width;
            center_x
        })
        .collect()
}

fn mean_line_points(
    mode: WeightedAverageMode,
    centers: &[f64],
    widths: &[f64],
    current_stats: StatsPoint,
    history_stats: &[StatsPoint],
) -> Vec<[f64; 2]> {
    match mode {
        WeightedAverageMode::Hidden => Vec::new(),
        WeightedAverageMode::Current => {
            if widths.is_empty() {
                Vec::new()
            } else {
                let total_width = widths.iter().sum();
                vec![[0.0, current_stats.mean], [total_width, current_stats.mean]]
            }
        }
        WeightedAverageMode::History => centers
            .iter()
            .zip(history_stats)
            .map(|(&center, stats)| [center, stats.mean])
            .collect(),
    }
}

fn standard_deviation_band_polygons(
    mode: WeightedAverageMode,
    centers: &[f64],
    widths: &[f64],
    current_stats: StatsPoint,
    history_stats: &[StatsPoint],
    show_band: bool,
) -> Vec<Vec<[f64; 2]>> {
    if !show_band {
        return Vec::new();
    }

    match mode {
        WeightedAverageMode::Hidden => Vec::new(),
        WeightedAverageMode::Current => current_band_polygons(widths, current_stats),
        WeightedAverageMode::History => history_band_polygons(centers, widths, history_stats),
    }
}

fn current_band_polygons(widths: &[f64], stats: StatsPoint) -> Vec<Vec<[f64; 2]>> {
    if widths.is_empty() {
        return Vec::new();
    }

    let total_width = widths.iter().sum();
    let polygon = band_between_points(0.0, stats, total_width, stats);

    vec![polygon]
}

fn history_band_polygons(
    centers: &[f64],
    widths: &[f64],
    stats_series: &[StatsPoint],
) -> Vec<Vec<[f64; 2]>> {
    let points = centers
        .iter()
        .copied()
        .zip(stats_series.iter().copied())
        .collect::<Vec<_>>();

    match points.as_slice() {
        [] => Vec::new(),
        [(center, stats)] => {
            vec![band_around_point(
                *center,
                widths.first().copied().unwrap_or(0.0),
                *stats,
            )]
        }
        _ => points
            .windows(2)
            .map(|pair| {
                let (left_center, left_stats) = pair[0];
                let (right_center, right_stats) = pair[1];
                band_between_points(left_center, left_stats, right_center, right_stats)
            })
            .collect(),
    }
}

fn band_around_point(center: f64, width: f64, stats: StatsPoint) -> Vec<[f64; 2]> {
    let half_width = width / 2.0;
    band_between_points(center - half_width, stats, center + half_width, stats)
}

fn band_between_points(
    left_x: f64,
    left_stats: StatsPoint,
    right_x: f64,
    right_stats: StatsPoint,
) -> Vec<[f64; 2]> {
    vec![
        [left_x, left_stats.mean - left_stats.std],
        [right_x, right_stats.mean - right_stats.std],
        [right_x, right_stats.mean + right_stats.std],
        [left_x, left_stats.mean + left_stats.std],
    ]
}

/// x座標がどのバーに属するか判定
fn find_clicked_bar(x: f64, boundaries: &[f64]) -> Option<usize> {
    // クリック場所が負の場合は範囲外確定
    if x < 0.0 {
        return Some(usize::MAX); // 選択解除
    }

    boundaries.iter().position(|&end_x| x < end_x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_centers_follow_cumulative_bar_widths() {
        let centers = bar_centers(&[0.25, 0.5, 1.0]);

        assert_eq!(centers, vec![0.125, 0.5, 1.25]);
    }

    #[test]
    fn fallback_y_bounds_keep_the_existing_y_initialization_and_current_x_range() {
        let frame = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 100.0));
        let current_bounds = PlotBounds::from_min_max([10.0, 20.0], [30.0, 40.0]);

        let bounds = fallback_y_bounds(current_bounds, frame, 42.0, 42.0);

        assert_eq!(bounds.range_x(), 10.0..=30.0);
        assert_eq!(bounds.range_y(), 41.5..=42.5);
    }

    #[test]
    fn current_line_and_band_cover_the_full_bar_width() {
        let centers = bar_centers(&[0.25, 0.5, 1.0]);
        let widths = [0.25, 0.5, 1.0];
        let current_stats = StatsPoint {
            mean: 50.0,
            std: 5.0,
        };
        let line = mean_line_points(
            WeightedAverageMode::Current,
            &centers,
            &widths,
            current_stats,
            &[],
        );
        let polygons = standard_deviation_band_polygons(
            WeightedAverageMode::Current,
            &centers,
            &widths,
            current_stats,
            &[],
            true,
        );

        assert_eq!(line, vec![[0.0, 50.0], [1.75, 50.0]]);
        assert_eq!(
            polygons,
            vec![vec![[0.0, 45.0], [1.75, 45.0], [1.75, 55.0], [0.0, 55.0],]]
        );
    }

    #[test]
    fn history_band_uses_each_point_std() {
        let polygons = standard_deviation_band_polygons(
            WeightedAverageMode::History,
            &[0.5, 1.5, 2.5],
            &[1.0, 1.0, 1.0],
            StatsPoint {
                mean: 0.0,
                std: 0.0,
            },
            &[
                StatsPoint {
                    mean: 10.0,
                    std: 1.0,
                },
                StatsPoint {
                    mean: 20.0,
                    std: 2.0,
                },
                StatsPoint {
                    mean: 30.0,
                    std: 3.0,
                },
            ],
            true,
        );

        assert_eq!(polygons.len(), 2);
        assert_eq!(
            polygons[0],
            vec![[0.5, 9.0], [1.5, 18.0], [1.5, 22.0], [0.5, 11.0],]
        );
        assert_eq!(
            polygons[1],
            vec![[1.5, 18.0], [2.5, 27.0], [2.5, 33.0], [1.5, 22.0],]
        );
    }

    #[test]
    fn chart_helpers_are_safe_for_empty_and_single_point_inputs() {
        let empty_centers = bar_centers(&[]);
        let empty_line = mean_line_points(
            WeightedAverageMode::History,
            &empty_centers,
            &[],
            StatsPoint {
                mean: 0.0,
                std: 0.0,
            },
            &[],
        );
        let empty_band = standard_deviation_band_polygons(
            WeightedAverageMode::History,
            &empty_centers,
            &[],
            StatsPoint {
                mean: 0.0,
                std: 0.0,
            },
            &[],
            true,
        );
        let empty_current_line = mean_line_points(
            WeightedAverageMode::Current,
            &empty_centers,
            &[],
            StatsPoint {
                mean: 0.0,
                std: 0.0,
            },
            &[],
        );
        let empty_current_band = standard_deviation_band_polygons(
            WeightedAverageMode::Current,
            &empty_centers,
            &[],
            StatsPoint {
                mean: 0.0,
                std: 0.0,
            },
            &[],
            true,
        );

        assert!(empty_line.is_empty());
        assert!(empty_band.is_empty());
        assert!(empty_current_line.is_empty());
        assert!(empty_current_band.is_empty());

        let one_center = bar_centers(&[1.0]);
        let one_line = mean_line_points(
            WeightedAverageMode::History,
            &one_center,
            &[1.0],
            StatsPoint {
                mean: 42.0,
                std: 3.0,
            },
            &[StatsPoint {
                mean: 42.0,
                std: 3.0,
            }],
        );
        let one_band = standard_deviation_band_polygons(
            WeightedAverageMode::History,
            &one_center,
            &[1.0],
            StatsPoint {
                mean: 42.0,
                std: 3.0,
            },
            &[StatsPoint {
                mean: 42.0,
                std: 3.0,
            }],
            true,
        );
        let one_current_line = mean_line_points(
            WeightedAverageMode::Current,
            &one_center,
            &[1.0],
            StatsPoint {
                mean: 42.0,
                std: 3.0,
            },
            &[],
        );
        let one_current_band = standard_deviation_band_polygons(
            WeightedAverageMode::Current,
            &one_center,
            &[1.0],
            StatsPoint {
                mean: 42.0,
                std: 3.0,
            },
            &[],
            true,
        );

        assert_eq!(one_line, vec![[0.5, 42.0]]);
        assert_eq!(one_band.len(), 1);
        assert_eq!(
            one_band[0],
            vec![[0.0, 39.0], [1.0, 39.0], [1.0, 45.0], [0.0, 45.0]]
        );
        assert!(one_band[0].iter().flatten().all(|value| value.is_finite()));
        assert_eq!(one_current_line, vec![[0.0, 42.0], [1.0, 42.0]]);
        assert_eq!(one_current_band, one_band);
    }
}
