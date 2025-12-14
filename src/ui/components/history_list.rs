use crate::models::app::{ItemData, ScoreEntry};
use crate::ui::Action;
use crate::ui::state::UiState;
use eframe::egui;

// 履歴カラムの描画
pub fn show(ui: &mut egui::Ui, item_data: &ItemData, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    ui.vertical(|ui| {
        ui.label("【履歴】");

        egui::ScrollArea::vertical()
            .id_salt("history")
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                let total = item_data.scores.len();
                action = item_data
                    .scores
                    .iter()
                    .rev()
                    .enumerate()
                    .filter_map(|(i, entry)| draw_history_row(ui, entry, i, total, state))
                    .last();
            });
    });

    action
}

fn draw_history_row(
    ui: &mut egui::Ui,
    entry: &ScoreEntry,
    rev_index: usize,
    total: usize,
    state: &mut UiState,
) -> Option<Action> {
    let mut action = None;
    let original_idx = total - 1 - rev_index;

    ui.horizontal(|ui| {
        // 削除ボタン
        if ui.button("🗑").clicked() {
            action = Some(Action::ShowDeleteScoreConfirm(original_idx));
        }

        // ラベル作成
        let local_time = entry.timestamp.with_timezone(&chrono::Local);
        let time_str = local_time.format("%Y-%m-%d %H:%M").to_string();
        let label_text = format!("[{}] {}回目: {}", time_str, original_idx + 1, entry.score);

        // 選択可能ラベルの描画
        let is_selected = state.selection.selected_history_index == Some(original_idx);
        let response = ui.selectable_label(is_selected, label_text);

        // クリック時の処理 (State更新)
        if response.clicked() {
            state.selection.selected_history_index = Some(original_idx);
        }

        // 自動スクロール
        if is_selected {
            response.scroll_to_me(Some(egui::Align::Center));
        }
    });

    action
}
