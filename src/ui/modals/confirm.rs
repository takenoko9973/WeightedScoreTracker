use eframe::egui;

use super::{Modal, ModalResult};
use crate::ui::Action;

pub struct ConfirmationModal {
    title: String,
    message: String,
    ok_action: Action,
}

impl ConfirmationModal {
    pub fn new(title: impl Into<String>, message: impl Into<String>, action: Action) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            ok_action: action,
        }
    }

    /// カテゴリ削除
    pub fn new_delete_category(cat_name: String) -> Self {
        Self::new(
            "カテゴリ削除",
            format!(
                "カテゴリ「{}」を削除しますか？\n含まれるすべての項目と履歴が失われます。",
                cat_name
            ),
            Action::ExecuteDeleteCategory(cat_name),
        )
    }

    /// 項目削除
    pub fn new_delete_item(cat_name: String, item_name: String) -> Self {
        Self::new(
            "項目削除",
            format!(
                "項目「{}」を削除しますか？\nこの項目の履歴データもすべて失われます。",
                item_name
            ),
            Action::ExecuteDeleteItem(cat_name, item_name),
        )
    }

    /// スコア削除
    pub fn new_delete_score(index: usize) -> Self {
        Self::new(
            "スコア削除",
            format!("{}個目のスコアを削除しますか？", index),
            Action::ExecuteDeleteScore(index),
        )
    }
}

impl Modal for ConfirmationModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;

        // 汎用的な確認ダイアログの描画
        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // 中央表示
            .show(ctx, |ui| {
                ui.label(&self.message);

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    if ui.button("キャンセル").clicked() {
                        result = ModalResult::Close;
                    }

                    if ui.colored_label(egui::Color32::RED, "削除する").clicked() {
                        // 規定のアクションを返す
                        result = ModalResult::Dispatch(self.ok_action.clone());
                    }
                });
            });

        result
    }
}
