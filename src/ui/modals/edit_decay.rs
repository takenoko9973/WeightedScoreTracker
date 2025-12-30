use super::{Modal, ModalResult};
use crate::action::Action;
use crate::constants::{MAX_DECAY_RATE, MIN_DECAY_RATE};
use eframe::egui;

pub struct EditDecayModal {
    cat_name: String,
    item_name: String,
    input_decay: String,
}

impl EditDecayModal {
    pub fn new(cat_name: String, item_name: String, now_decay: f64) -> Self {
        Self {
            cat_name,
            item_name,
            input_decay: now_decay.to_string(),
        }
    }
}

impl Modal for EditDecayModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        egui::Window::new("減衰率変更")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!("対象: {} > {}", self.cat_name, self.item_name));
                ui.label(format!(
                    "新しい減衰率 ({:.2} - {:.2}):",
                    MIN_DECAY_RATE, MAX_DECAY_RATE
                ));
                ui.text_edit_singleline(&mut self.input_decay);
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("更新").clicked() {
                        result = ModalResult::Dispatch(Action::UpdateDecayRate(
                            self.input_decay.clone(),
                        ));
                    }
                    if ui.button("キャンセル").clicked() {
                        result = ModalResult::Close;
                    }
                });
            });

        result
    }
}
