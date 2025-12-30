use crate::ui::Action;
use eframe::egui;

pub struct ScoreInput;

impl ScoreInput {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, input_text: &mut String) -> Option<Action> {
        let mut action = None;

        ui.vertical(|ui| {
            ui.label("スコア入力");
            ui.horizontal(|ui| {
                let res = ui.text_edit_singleline(input_text);

                let is_clicked_button = ui.button("追加").clicked();
                let is_enter = res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                if is_clicked_button || is_enter {
                    if !input_text.is_empty() {
                        action = Some(Action::AddScore(input_text.clone()));
                    }
                    // エンターを押されるとフォーカスが切れるため、空白かどうか関係なくフォーカスし直し
                    res.request_focus();
                }
            });
        });

        action
    }
}
