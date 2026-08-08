use super::{Modal, ModalResult};
use crate::action::Action;
use crate::constants::{MAX_DECAY_RATE, MIN_DECAY_RATE};
use crate::domain::{TagData, TagId};
use crate::utils::ime::ImeFocusExtension;
use eframe::egui;

pub struct EditItemModal {
    target_cat: String,
    target_item: String,
    input_cat: String,
    input_item: String,
    input_subtitle: String,
    input_decay: String,
    available_categories: Vec<String>,
    available_tags: Vec<TagData>,
    selected_tag_ids: Vec<TagId>,
}

impl EditItemModal {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cat_name: String,
        item_name: String,
        current_decay: f64,
        current_subtitle: String,
        current_tag_ids: Vec<TagId>,
        categories: Vec<String>,
        mut tags: Vec<TagData>,
    ) -> Self {
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            target_cat: cat_name.clone(),
            target_item: item_name.clone(),
            input_cat: cat_name,
            input_item: item_name,
            input_subtitle: current_subtitle,
            input_decay: current_decay.to_string(),
            available_categories: categories,
            available_tags: tags,
            selected_tag_ids: current_tag_ids,
        }
    }

    fn toggle_tag(&mut self, id: TagId, selected: bool) {
        if selected {
            if !self.selected_tag_ids.contains(&id) {
                self.selected_tag_ids.push(id);
            }
        } else {
            self.selected_tag_ids.retain(|tag_id| *tag_id != id);
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
                        egui::ComboBox::from_id_salt("edit_item_category_select")
                            .selected_text(self.input_cat.clone())
                            .show_ui(ui, |ui| {
                                for cat in &self.available_categories {
                                    ui.selectable_value(&mut self.input_cat, cat.clone(), cat);
                                }
                            });
                        ui.end_row();

                        ui.label("項目名:");
                        let response = ui.text_edit_singleline(&mut self.input_item);
                        response.handle_ime_focus(ui);
                        ui.end_row();

                        ui.label("補足:");
                        ui.text_edit_singleline(&mut self.input_subtitle);
                        ui.end_row();

                        ui.label("減衰率:");
                        ui.vertical(|ui| {
                            ui.text_edit_singleline(&mut self.input_decay);
                            ui.label(
                                egui::RichText::new(format!(
                                    "({:.2} - {:.2})",
                                    MIN_DECAY_RATE, MAX_DECAY_RATE
                                ))
                                .size(10.0)
                                .color(egui::Color32::GRAY),
                            );
                        });
                        ui.end_row();
                    });

                ui.add_space(5.0);
                ui.label("タグ:");
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .show(ui, |ui| {
                        for tag in self.available_tags.clone() {
                            let mut selected = self.selected_tag_ids.contains(&tag.id);
                            let mut changed = false;
                            ui.horizontal(|ui| {
                                ui.colored_label(
                                    egui::Color32::from_rgb(
                                        tag.color[0],
                                        tag.color[1],
                                        tag.color[2],
                                    ),
                                    "●",
                                );
                                changed = ui.checkbox(&mut selected, &tag.name).changed();
                            });
                            if changed {
                                self.toggle_tag(tag.id, selected);
                            }
                        }
                    });

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui.button("保存").clicked() {
                        result = ModalResult::Dispatch(Action::UpdateItem(
                            self.target_cat.clone(),
                            self.target_item.clone(),
                            self.input_cat.clone(),
                            self.input_item.clone(),
                            self.input_subtitle.clone(),
                            self.input_decay.clone(),
                            self.selected_tag_ids.clone(),
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
