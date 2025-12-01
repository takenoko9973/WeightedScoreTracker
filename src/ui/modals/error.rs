use crate::ui::state::UiState;
use eframe::egui;

pub fn show(ctx: &egui::Context, state: &mut UiState) {
    let mut open = true;

    // unwrapが安全なのは、呼び出し元で is_some() をチェックしているからだが、
    // 万が一のために if let を使うのがより堅牢
    if let Some(msg) = &state.error_message {
        let msg_clone = msg.clone();

        egui::Window::new("エラー")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(msg_clone);
                ui.add_space(10.0);
                ui.vertical_centered(|ui| {
                    if ui.button("OK").clicked() {
                        open = false;
                    }
                });
            });
    }

    if !open {
        state.error_message = None;
    }
}
