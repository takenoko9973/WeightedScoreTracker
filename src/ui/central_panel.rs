use crate::logic::calculate_stats;
use crate::models::{AppData, ItemData};
use crate::ui::Action;
use crate::ui::components::{history_list, score_chart, score_input};
use crate::ui::state::UiState;
use eframe::egui::{self};

pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
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

            // データ取得: カテゴリ -> 項目
            let Some(cat_data) = data.categories.get(cat_name) else {
                ui.label("カテゴリデータ読み込みエラー");
                return None;
            };
            let Some(item_data) = cat_data.items.get(item_name) else {
                ui.label("項目データ読み込みエラー");
                return None;
            };

            // ===========================================

            // ヘッダー
            let header_action = draw_header(ui, item_data);
            ui.separator();

            // グラフ
            score_chart::show(ui, item_data, &mut state.selection.selected_history_index);

            ui.add_space(10.0);

            // 入力と履歴
            let (input_action, history_action) = ui.columns(2, |columns| {
                (
                    // 左カラム: 入力
                    score_input::show(&mut columns[0], state),
                    // 右カラム: 履歴
                    history_list::show(&mut columns[1], item_data, state),
                )
            });

            header_action.or(input_action).or(history_action)
        })
        .inner
}

/// ヘッダー（統計情報と設定ボタン）の描画
fn draw_header(ui: &mut egui::Ui, item_data: &ItemData) -> Option<Action> {
    let (avg, std, count, _) = calculate_stats(&item_data.scores, item_data.decay_rate);
    let mut action = None;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("現在の加重平均: {:.2}", avg))
                .size(16.0)
                .strong(),
        );
        ui.label(format!("(加重標準偏差: {:.2})", std));
        ui.label(format!("(データ数: {})", count));

        // 右寄せ配置 (右から左に順番に設置)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("設定変更").clicked() {
                action = Some(Action::ShowEditDecayModal);
            }
            ui.label(format!("減衰率: {:.2}", item_data.decay_rate));
        });
    });

    action
}
