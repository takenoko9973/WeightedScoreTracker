use super::{Modal, ModalResult};
use crate::action::Action;
use crate::constants::MIN_DECAY_RATE;
use eframe::egui;

pub struct EditItemModal {
    target_cat: String,
    target_item: String,
    input_cat: String,
    input_item: String,
    input_decay: String,

    available_categories: Vec<String>,
}

impl EditItemModal {
    pub fn new(
        cat_name: String,
        item_name: String,
        current_decay: f64,
        categories: Vec<String>,
    ) -> Self {
        Self {
            target_cat: cat_name.clone(),
            target_item: item_name.clone(),
            input_cat: cat_name,
            input_item: item_name,
            input_decay: current_decay.to_string(),
            available_categories: categories,
        }
    }
}

impl Modal for EditItemModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        egui::Window::new("項目編集")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                egui::Grid::new("edit_item_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("カテゴリ:");
                        egui::ComboBox::from_id_salt("cat_select")
                            .selected_text(self.input_cat.clone())
                            .show_ui(ui, |ui| {
                                // 存在するカテゴリを一覧表示
                                self.available_categories.iter().for_each(|cat| {
                                    ui.selectable_value(&mut self.input_cat, cat.clone(), cat);
                                });
                            });
                        ui.end_row();

                        ui.label("項目名:");
                        ui.text_edit_singleline(&mut self.input_item);
                        ui.end_row();

                        ui.label("減衰率:");
                        ui.vertical(|ui| {
                            ui.text_edit_singleline(&mut self.input_decay);
                            ui.label(
                                egui::RichText::new(format!(
                                    "({:.2} - {:.2})",
                                    MIN_DECAY_RATE, MIN_DECAY_RATE
                                ))
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                            );
                        });
                        ui.end_row();
                    });

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui.button("保存").clicked() {
                        result = ModalResult::Dispatch(Action::UpdateItem(
                            self.target_cat.clone(),
                            self.target_item.clone(),
                            self.input_cat.clone(),
                            self.input_item.clone(),
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
