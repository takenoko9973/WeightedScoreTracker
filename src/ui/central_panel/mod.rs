mod chart;
mod history;
mod input;

use eframe::egui::{self};

use crate::logic::calculate_stats;
use crate::models::app::{AppData, ItemData};
use crate::ui::Action;
use crate::ui::central_panel::chart::WeightedScoreChart;
use crate::ui::central_panel::history::HistoryList;
use crate::ui::central_panel::input::ScoreInput;
use crate::ui::state::UiState;
use crate::utils::comma_display::CommaDisplay;

pub struct CentralPanel {
    selected_index: Option<usize>,
    scroll_req_index: Option<usize>,
}

impl CentralPanel {
    pub fn new() -> Self {
        Self {
            selected_index: None,   // 選択中インデックス
            scroll_req_index: None, // リストに対するスクロール処理用インデックス
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        data: &AppData,
        state: &mut UiState,
    ) -> Option<Action> {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                // カテゴリ未選択
                let (Some(cat_name), Some(item_name)) = (
                    &state.selection.current_category,
                    &state.selection.current_item,
                ) else {
                    ui.centered_and_justified(|ui| {
                        ui.label("左のリストから項目を選択するか、追加してください");
                    });
                    return None;
                };

                // データ取得
                let Ok(item_data) = data.get_item(cat_name, item_name) else {
                    ui.label("項目データ読み込みエラー");
                    return None;
                };

                // ===========================================

                // ヘッダー
                let header_action = self.draw_header(ui, item_data);
                ui.separator();

                // グラフ
                WeightedScoreChart::new().show(
                    ui,
                    &item_data.scores,
                    item_data.decay_rate,
                    &mut self.selected_index,
                    &mut self.scroll_req_index,
                );

                ui.add_space(10.0);

                // 入力と履歴
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(20, 0))
                    .show(ui, |ui| {
                        let input_action = egui::Grid::new("input_history_col")
                            .min_row_height(ui.available_height())
                            .num_columns(2)
                            .spacing([20.0, 0.0])
                            .show(ui, |ui| {
                                // 左カラム: 入力
                                let input_action = ScoreInput::new().show(ui);
                                // 右カラム: 履歴
                                let history_action = HistoryList::new(&item_data.scores).show(
                                    ui,
                                    &mut self.selected_index,
                                    &mut self.scroll_req_index,
                                );
                                input_action.or(history_action)
                            })
                            .inner;

                        header_action.or(input_action)
                    })
                    .inner
            })
            .inner
    }

    /// ヘッダー（統計情報と設定ボタン）の描画
    fn draw_header(&self, ui: &mut egui::Ui, item_data: &ItemData) -> Option<Action> {
        let (avg, std, count, _) = calculate_stats(&item_data.scores, item_data.decay_rate);
        let mut action = None;

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("現在の加重平均: {}", avg.to_comma_fmt(2)))
                    .size(16.0)
                    .strong(),
            );
            ui.label(format!("(加重標準偏差: {})", std.to_comma_fmt(2)));
            ui.label(format!("(データ数: {})", count));

            // 右寄せ配置 (右から左に順番に設置)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("設定変更").clicked() {
                    action = Some(Action::ShowEditDecayModal(item_data.decay_rate));
                }
                ui.label(format!("減衰率: {}", item_data.decay_rate.to_comma_fmt(2)));
            });
        });

        action
    }
}
