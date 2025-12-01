use crate::ui::{Action, state::UiState};
use eframe::egui;

/// 入力カラムの描画
pub fn show(ui: &mut egui::Ui, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    ui.vertical(|ui| {
        ui.label("【スコア入力】");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            // 記録ボタン
            let is_clicked = ui.button("記録").clicked();

            // 入力欄
            let response = ui.add(
                egui::TextEdit::singleline(&mut state.selection.input_score)
                    .desired_width(f32::INFINITY),
            );
            // 入力欄でのenter入力
            let is_enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if is_enter || is_clicked {
                action = Some(Action::AddScore(state.selection.input_score.clone()));
                if is_enter {
                    response.request_focus();
                }
            }
        });
    });

    action
}
