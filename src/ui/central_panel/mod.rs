mod chart;
mod history;
mod score_input;

use crate::action::Action;
use crate::domain::{ItemData, TrackerModel};
use crate::logic::calculate_stats;
use crate::ui::central_panel::chart::{ChartSettings, WeightedAverageMode, WeightedScoreChart};
use crate::ui::central_panel::history::HistoryList;
use crate::ui::central_panel::score_input::ScoreInput;
use crate::utils::comma_display::CommaDisplay;
use eframe::egui::{self};

const INPUT_TO_ITEM_SETTINGS_GAP: f32 = 8.0;
const ITEM_TO_CHART_SETTINGS_GAP: f32 = 8.0;

fn item_settings_action(cat_name: &str, item_name: &str, clicked: bool) -> Option<Action> {
    clicked.then(|| Action::ShowEditItemModal(cat_name.to_string(), item_name.to_string()))
}

pub struct CentralPanel {
    score_input_text: String,
    weighted_average_mode: WeightedAverageMode,
    show_weighted_std_band: bool,

    selected_index: Option<usize>,
    scroll_req_index: Option<usize>,
}

impl CentralPanel {
    pub fn new() -> Self {
        Self {
            score_input_text: String::new(),
            weighted_average_mode: WeightedAverageMode::Current,
            show_weighted_std_band: false,

            selected_index: None,   // 選択中インデックス
            scroll_req_index: None, // リストに対するスクロール処理用インデックス
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        model: &TrackerModel,
        enabled: bool,
    ) -> Option<Action> {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                if !enabled {
                    // UIの無効化
                    ui.disable();
                }

                // カテゴリ未選択
                let (Some(cat_name), Some(item_name)) =
                    (&model.selection.category, &model.selection.item)
                else {
                    ui.centered_and_justified(|ui| {
                        ui.label("左のリストから項目を選択するか、追加してください");
                    });
                    return None;
                };

                // データ取得
                let Ok(item_data) = model.data.get_item(cat_name, item_name) else {
                    ui.label("項目データ読み込みエラー");
                    return None;
                };

                // ===========================================

                // ヘッダー
                self.draw_header(ui, item_data);
                ui.separator();

                // グラフ
                WeightedScoreChart::new().show(
                    ui,
                    &item_data.scores,
                    item_data.decay_rate,
                    ChartSettings {
                        average_mode: self.weighted_average_mode,
                        show_std_band: self.show_weighted_std_band,
                    },
                    &mut self.selected_index,
                    &mut self.scroll_req_index,
                );

                ui.add_space(10.0);

                // 入力と履歴
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(20, 0))
                    .show(ui, |ui| {
                        egui::Grid::new("input_history_col")
                            .min_row_height(ui.available_height())
                            .num_columns(2)
                            .spacing([20.0, 0.0])
                            .show(ui, |ui| {
                                // 左カラム: 入力
                                let input_action = ui
                                    .vertical(|ui| {
                                        let input_action =
                                            ScoreInput::new().show(ui, &mut self.score_input_text);
                                        ui.add_space(INPUT_TO_ITEM_SETTINGS_GAP);
                                        let settings_action = self
                                            .draw_item_settings(ui, cat_name, item_name, item_data);
                                        ui.add_space(ITEM_TO_CHART_SETTINGS_GAP);
                                        ui.separator();
                                        self.draw_chart_settings(ui);
                                        input_action.or(settings_action)
                                    })
                                    .inner;

                                // 右カラム: 履歴
                                let history_action = HistoryList::new(&item_data.scores).show(
                                    ui,
                                    &mut self.selected_index,
                                    &mut self.scroll_req_index,
                                );
                                input_action.or(history_action)
                            })
                            .inner
                    })
                    .inner
            })
            .inner
    }

    /// ヘッダー（統計情報）の描画
    fn draw_header(&self, ui: &mut egui::Ui, item_data: &ItemData) {
        let (avg, std, count, _) = calculate_stats(&item_data.scores, item_data.decay_rate);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("現在の加重平均: {}", avg.to_comma_fmt(2)))
                    .size(16.0)
                    .strong(),
            );
            ui.label(format!("加重標準偏差: {}", std.to_comma_fmt(2)));
            ui.label(format!("データ数: {}", count));
        });
    }

    fn draw_item_settings(
        &self,
        ui: &mut egui::Ui,
        cat_name: &str,
        item_name: &str,
        item_data: &ItemData,
    ) -> Option<Action> {
        let mut action = None;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("項目設定").strong());
            ui.label(format!("減衰率 {}", item_data.decay_rate.to_comma_fmt(2)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let clicked = ui.button("設定…").clicked();
                action = item_settings_action(cat_name, item_name, clicked);
            });
        });

        action
    }

    fn draw_chart_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("グラフ設定").strong());
        ui.add_space(4.0);

        egui::Grid::new("chart_settings_summary")
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.label("加重平均");
                egui::ComboBox::from_id_salt("weighted_average_mode")
                    .selected_text(self.weighted_average_mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.weighted_average_mode,
                            WeightedAverageMode::Hidden,
                            WeightedAverageMode::Hidden.label(),
                        );
                        ui.selectable_value(
                            &mut self.weighted_average_mode,
                            WeightedAverageMode::Current,
                            WeightedAverageMode::Current.label(),
                        );
                        ui.selectable_value(
                            &mut self.weighted_average_mode,
                            WeightedAverageMode::History,
                            WeightedAverageMode::History.label(),
                        );
                    });
                ui.end_row();

                ui.label("加重標準偏差帯");
                ui.add_enabled(
                    self.weighted_average_mode != WeightedAverageMode::Hidden,
                    egui::Checkbox::new(&mut self.show_weighted_std_band, "表示"),
                );
                ui.end_row();
            });
    }

    pub fn clear_input(&mut self) {
        self.score_input_text.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_settings_button_dispatches_edit_item_action() {
        // 項目設定ボタンが押された場合に、項目編集モーダル起動アクションが生成されることを確認する。
        let action = item_settings_action("CatA", "ItemA", true);
        assert!(matches!(
            action,
            Some(Action::ShowEditItemModal(cat, item)) if cat == "CatA" && item == "ItemA"
        ));
    }

    #[test]
    fn item_settings_button_no_click_yields_no_action() {
        // 項目設定ボタンが押されていない場合はアクションが生成されないことを確認する。
        let action = item_settings_action("CatA", "ItemA", false);
        assert!(action.is_none());
    }

    #[test]
    fn central_panel_defaults_to_current_average_without_std_band() {
        // パネル初期状態は既存挙動の現在値表示で、標準偏差帯は非表示になっていることを確認する。
        let panel = CentralPanel::new();
        assert_eq!(panel.weighted_average_mode, WeightedAverageMode::Current);
        assert!(!panel.show_weighted_std_band);
    }

    #[test]
    fn settings_gaps_keep_the_compact_layout() {
        // 設定を通常表示領域に収めやすくしつつ、入力と各設定の間隔を維持する。
        const {
            assert!(INPUT_TO_ITEM_SETTINGS_GAP < 16.0);
            assert!(ITEM_TO_CHART_SETTINGS_GAP < 16.0);
            assert!(INPUT_TO_ITEM_SETTINGS_GAP > 0.0);
            assert!(ITEM_TO_CHART_SETTINGS_GAP > 0.0);
        }
    }
}
