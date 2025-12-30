use super::{Modal, ModalResult};
use crate::action::Action;
use crate::constants::{DEFAULT_DECAY_RATE, MAX_DECAY_RATE, MIN_DECAY_RATE};
use eframe::egui;

// 必要なデータはすべてフィールドとして持つ！
pub struct AddItemModal {
    target_cat: String,
    input_item: String,
    input_decay: String,
}

impl AddItemModal {
    // コンストラクタで初期値を受け取る
    pub fn new(cat_name: String) -> Self {
        Self {
            target_cat: cat_name.clone(),
            input_item: String::new(),
            input_decay: DEFAULT_DECAY_RATE.to_string(),
        }
    }
}

// トレイトの実装（ここがメソッド本体）
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
                ui.text_edit_singleline(&mut self.input_item);

                ui.label(format!(
                    "減衰率 ({:.2} - {:.2}):",
                    MIN_DECAY_RATE, MAX_DECAY_RATE
                ));
                ui.text_edit_singleline(&mut self.input_decay);

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("追加").clicked() {
                        result = ModalResult::Dispatch(Action::AddItem(
                            self.target_cat.to_string(),
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
