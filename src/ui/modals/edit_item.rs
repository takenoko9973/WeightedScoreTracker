use super::{Modal, ModalResult, tag_chip};
use crate::action::Action;
use crate::constants::{MAX_DECAY_RATE, MIN_DECAY_RATE};
use crate::domain::{TagData, TagId};
use crate::utils::ime::ImeFocusExtension;
use eframe::egui;

const MODAL_DEFAULT_WIDTH: f32 = 360.0;
const MODAL_MAX_WIDTH: f32 = 420.0;
const EDIT_FIELD_WIDTH: f32 = 240.0;

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

    fn build_update_action(&self) -> Action {
        Action::UpdateItem(
            self.target_cat.clone(),
            self.target_item.clone(),
            self.input_cat.clone(),
            self.input_item.clone(),
            self.input_subtitle.clone(),
            self.input_decay.clone(),
            self.selected_tag_ids.clone(),
        )
    }
}

impl Modal for EditItemModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        egui::Window::new("項目編集")
            .collapsible(false)
            .resizable(false)
            .default_width(MODAL_DEFAULT_WIDTH)
            .max_width(MODAL_MAX_WIDTH)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                egui::Grid::new("edit_item_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("カテゴリ:");
                        egui::ComboBox::from_id_salt("edit_item_category_select")
                            .width(EDIT_FIELD_WIDTH)
                            .wrap_mode(egui::TextWrapMode::Truncate)
                            .selected_text(self.input_cat.clone())
                            .show_ui(ui, |ui| {
                                for cat in &self.available_categories {
                                    ui.selectable_value(&mut self.input_cat, cat.clone(), cat);
                                }
                            });
                        ui.end_row();

                        ui.label("項目名:");
                        let response = ui.add_sized(
                            [EDIT_FIELD_WIDTH, ui.spacing().interact_size.y],
                            egui::TextEdit::singleline(&mut self.input_item),
                        );
                        response.handle_ime_focus(ui);
                        ui.end_row();

                        ui.label("メモ:");
                        ui.add_sized(
                            [EDIT_FIELD_WIDTH, ui.spacing().interact_size.y],
                            egui::TextEdit::singleline(&mut self.input_subtitle),
                        );
                        ui.end_row();

                        ui.label("減衰率:");
                        ui.vertical(|ui| {
                            ui.add_sized(
                                [EDIT_FIELD_WIDTH, ui.spacing().interact_size.y],
                                egui::TextEdit::singleline(&mut self.input_decay),
                            );
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
                        ui.horizontal_wrapped(|ui| {
                            for tag in self.available_tags.clone() {
                                let selected = self.selected_tag_ids.contains(&tag.id);
                                if tag_chip(ui, &tag, selected).clicked() {
                                    self.toggle_tag(tag.id, !selected);
                                }
                            }
                        });
                    });

                ui.add_space(15.0);

                ui.horizontal(|ui| {
                    if ui.button("保存").clicked() {
                        result = ModalResult::Dispatch(self.build_update_action());
                    }
                    if ui.button("キャンセル").clicked() {
                        result = ModalResult::Close;
                    }
                });
            });

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(id: TagId, name: &str) -> TagData {
        TagData {
            id,
            name: name.to_string(),
            color: [1, 2, 3],
        }
    }

    #[test]
    fn current_tags_are_loaded_and_can_be_added_or_removed() {
        let mut modal = EditItemModal::new(
            "Cat".to_string(),
            "Item".to_string(),
            0.9,
            String::new(),
            vec![1],
            vec!["Cat".to_string()],
            vec![tag(1, "First"), tag(2, "Second")],
        );

        assert_eq!(modal.selected_tag_ids, vec![1]);
        modal.toggle_tag(2, true);
        modal.toggle_tag(1, false);

        assert_eq!(modal.selected_tag_ids, vec![2]);

        let action = modal.build_update_action();
        match action {
            Action::UpdateItem(_, _, _, _, _, _, tag_ids) => assert_eq!(tag_ids, vec![2]),
            other => panic!("unexpected action: {other:?}"),
        }
    }
}
