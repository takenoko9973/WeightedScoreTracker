use super::{Modal, ModalResult};
use crate::action::Action;
use crate::constants::{DEFAULT_DECAY_RATE, MAX_DECAY_RATE, MIN_DECAY_RATE};
use crate::domain::{TagData, TagId};
use crate::utils::ime::ImeFocusExtension;
use eframe::egui;

pub struct AddItemModal {
    target_cat: String,
    input_item: String,
    input_subtitle: String,
    input_decay: String,
    available_tags: Vec<TagData>,
    selected_tag_ids: Vec<TagId>,
}

impl AddItemModal {
    pub fn new(cat_name: String, mut tags: Vec<TagData>) -> Self {
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            target_cat: cat_name,
            input_item: String::new(),
            input_subtitle: String::new(),
            input_decay: DEFAULT_DECAY_RATE.to_string(),
            available_tags: tags,
            selected_tag_ids: Vec::new(),
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

impl Modal for AddItemModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        egui::Window::new("項目追加")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!("追加先カテゴリ: {}", self.target_cat));

                ui.label("項目名:");
                let response = ui.text_edit_singleline(&mut self.input_item);
                response.handle_ime_focus(ui);

                ui.label("補足:");
                ui.text_edit_singleline(&mut self.input_subtitle);

                ui.label(format!(
                    "減衰率 ({:.2} - {:.2}):",
                    MIN_DECAY_RATE, MAX_DECAY_RATE
                ));
                ui.text_edit_singleline(&mut self.input_decay);

                ui.label("タグ:");
                egui::ScrollArea::vertical()
                    .max_height(100.0)
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

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("追加").clicked() {
                        result = ModalResult::Dispatch(Action::AddItem(
                            self.target_cat.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_tag_adds_and_removes_id_without_duplicates() {
        let mut modal = AddItemModal::new("Cat".to_string(), Vec::new());
        modal.toggle_tag(3, true);
        modal.toggle_tag(3, true);
        assert_eq!(modal.selected_tag_ids, vec![3]);
        modal.toggle_tag(3, false);
        assert!(modal.selected_tag_ids.is_empty());
    }
}
