use super::{Modal, ModalResult};
use crate::action::Action;
use crate::domain::TagData;
use eframe::egui;

pub struct TagManagerModal {
    tags: Vec<TagData>,
    new_name: String,
    new_color: [u8; 3],
}

impl TagManagerModal {
    pub fn new(mut tags: Vec<TagData>) -> Self {
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            tags,
            new_name: String::new(),
            new_color: [100, 149, 237],
        }
    }
}

impl Modal for TagManagerModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        egui::Window::new("タグ管理")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                if self.tags.is_empty() {
                    ui.label("タグはありません。");
                }

                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .show(ui, |ui| {
                        for tag in &mut self.tags {
                            ui.push_id(tag.id, |ui| {
                                ui.horizontal(|ui| {
                                    ui.color_edit_button_srgb(&mut tag.color);
                                    ui.add_sized(
                                        [140.0, 24.0],
                                        egui::TextEdit::singleline(&mut tag.name),
                                    );
                                    if ui.button("保存").clicked() {
                                        result = ModalResult::Dispatch(Action::UpdateTag(
                                            tag.id,
                                            tag.name.clone(),
                                            tag.color,
                                        ));
                                    }
                                    if ui.button("削除").clicked() {
                                        result = ModalResult::Dispatch(Action::DeleteTag(tag.id));
                                    }
                                });
                            });
                        }
                    });

                ui.separator();
                ui.label("新しいタグ");
                ui.horizontal(|ui| {
                    ui.color_edit_button_srgb(&mut self.new_color);
                    ui.add_sized(
                        [140.0, 24.0],
                        egui::TextEdit::singleline(&mut self.new_name),
                    );
                    if ui.button("追加").clicked() {
                        result = ModalResult::Dispatch(Action::CreateTag(
                            self.new_name.clone(),
                            self.new_color,
                        ));
                    }
                });

                ui.add_space(10.0);
                if ui.button("閉じる").clicked() {
                    result = ModalResult::Close;
                }
            });

        result
    }
}
