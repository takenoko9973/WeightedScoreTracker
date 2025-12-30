use super::{Modal, ModalResult};
use crate::action::Action;
use eframe::egui;

pub struct AddCategoryModal {
    input_cat: String,
}

impl AddCategoryModal {
    pub fn new() -> Self {
        Self {
            input_cat: String::new(),
        }
    }
}

impl Modal for AddCategoryModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        egui::Window::new("カテゴリ追加")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("カテゴリ名:");
                ui.text_edit_singleline(&mut self.input_cat);

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("追加").clicked() {
                        result = ModalResult::Dispatch(Action::AddCategory(self.input_cat.clone()));
                    }
                    if ui.button("キャンセル").clicked() {
                        result = ModalResult::Close;
                    }
                });
            });

        result
    }
}
