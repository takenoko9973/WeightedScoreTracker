use super::{Modal, ModalResult, show_tag_chips, sort_tags};
use crate::action::Action;
use crate::constants::{DEFAULT_DECAY_RATE, MAX_DECAY_RATE, MIN_DECAY_RATE};
use crate::domain::{TagData, TagId};
use crate::utils::ime::ImeFocusExtension;
use eframe::egui;

const MODAL_DEFAULT_WIDTH: f32 = 360.0;
const MODAL_MAX_WIDTH: f32 = 420.0;

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
        sort_tags(&mut tags);
        Self {
            target_cat: cat_name,
            input_item: String::new(),
            input_subtitle: String::new(),
            input_decay: DEFAULT_DECAY_RATE.to_string(),
            available_tags: tags,
            selected_tag_ids: Vec::new(),
        }
    }
}

impl Modal for AddItemModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        egui::Window::new("項目追加")
            .collapsible(false)
            .resizable(false)
            .default_width(MODAL_DEFAULT_WIDTH)
            .max_width(MODAL_MAX_WIDTH)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let target_width = ui.available_width();
                ui.add_sized(
                    [target_width, ui.spacing().interact_size.y],
                    egui::Label::new(format!("追加先カテゴリ: {}", self.target_cat)).truncate(),
                );

                ui.label("項目名:");
                let response = ui.text_edit_singleline(&mut self.input_item);
                response.handle_ime_focus(ui);

                ui.label("メモ:");
                ui.add_sized(
                    [ui.available_width(), ui.spacing().interact_size.y],
                    egui::TextEdit::singleline(&mut self.input_subtitle),
                );

                ui.label(format!(
                    "減衰率 ({:.2} - {:.2}):",
                    MIN_DECAY_RATE, MAX_DECAY_RATE
                ));
                ui.add_sized(
                    [ui.available_width(), ui.spacing().interact_size.y],
                    egui::TextEdit::singleline(&mut self.input_decay),
                );

                ui.label("タグ:");
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .show(ui, |ui| {
                        show_tag_chips(ui, &self.available_tags, &mut self.selected_tag_ids);
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
    use super::super::toggle_tag;
    use super::*;

    #[test]
    fn toggling_tag_adds_and_removes_id_without_duplicates() {
        let mut modal = AddItemModal::new("Cat".to_string(), Vec::new());
        toggle_tag(&mut modal.selected_tag_ids, 3, true);
        toggle_tag(&mut modal.selected_tag_ids, 3, true);
        assert_eq!(modal.selected_tag_ids, vec![3]);
        toggle_tag(&mut modal.selected_tag_ids, 3, false);
        assert!(modal.selected_tag_ids.is_empty());
    }
}
