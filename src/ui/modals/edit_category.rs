use super::{Modal, ModalResult};
use crate::action::Action;
use eframe::egui;

pub struct EditCategoryModal {
    target_cat: String,
    input_cat: String,
}

impl EditCategoryModal {
    pub fn new(cat_name: String) -> Self {
        Self {
            target_cat: cat_name.clone(),
            input_cat: cat_name,
        }
    }
}

impl Modal for EditCategoryModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        egui::Window::new("カテゴリ名変更")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!("対象: {}", self.target_cat));
                ui.label("新しい名前:");
                ui.text_edit_singleline(&mut self.input_cat);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("変更").clicked() {
                        result = ModalResult::Dispatch(Action::RenameCategory(
                            self.target_cat.clone(),
                            self.input_cat.clone(),
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
