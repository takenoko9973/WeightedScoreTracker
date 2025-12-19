use crate::ui::Action;
use eframe::egui;

pub struct ScoreInput {
    input_text: String,
}

impl ScoreInput {
    pub fn new() -> Self {
        Self {
            input_text: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<Action> {
        let mut action = None;

        ui.vertical(|ui| {
            ui.label("スコア入力");
            ui.horizontal(|ui| {
                let res = ui.text_edit_singleline(&mut self.input_text);

                let is_clicked_button = ui.button("追加").clicked();
                let is_enter = res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if (is_clicked_button || is_enter) && !self.input_text.is_empty() {
                    action = Some(Action::AddScore(self.input_text.clone()));
                    self.input_text.clear(); // 追加したらクリア
                }
            });
        });

        action
    }
}
