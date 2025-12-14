use crate::models::app::AppData;
use crate::ui::Action;
use crate::ui::state::SelectionState;
use eframe::egui;

pub fn show_delete_category(
    ctx: &egui::Context,
    target_cat: &str,
    should_close: &mut bool,
) -> Option<Action> {
    let mut action = None;
    let mut open = true;

    egui::Window::new("削除確認")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(format!("カテゴリ \"{}\" を削除しますか？", target_cat));
            ui.label(
                egui::RichText::new("※含まれる全ての項目とスコアが完全に消去されます。")
                    .color(egui::Color32::RED),
            );
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("削除する").clicked() {
                    action = Some(Action::ExecuteDeleteCategory(target_cat.to_string()));
                }
                if ui.button("キャンセル").clicked() {
                    open = false;
                }
            });
        });

    if !open {
        *should_close = true;
    }
    action
}

pub fn show_delete_item(
    ctx: &egui::Context,
    target_cat: &str,
    target_item: &str,
    should_close: &mut bool,
) -> Option<Action> {
    let mut action = None;
    let mut open = true;

    egui::Window::new("削除確認")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(format!("\"{}\" を削除しますか？", target_item));
            ui.label(
                egui::RichText::new("※含まれる全てのスコアデータが完全に消去されます。")
                    .color(egui::Color32::RED),
            );
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("削除する").clicked() {
                    action = Some(Action::ExecuteDeleteItem(
                        target_cat.to_string(),
                        target_item.to_string(),
                    ));
                }
                if ui.button("キャンセル").clicked() {
                    open = false;
                }
            });
        });

    if !open {
        *should_close = true;
    }
    action
}

pub fn show_delete_score(
    ctx: &egui::Context,
    data: &AppData,
    selection: &SelectionState,
    delete_idx: usize,
    should_close: &mut bool,
) -> Option<Action> {
    let mut action = None;
    let mut open = true;

    // 削除対象の情報を表示用に取得
    let mut target_info = String::new();
    if let (Some(cat_name), Some(item_name)) =
        (&selection.current_category, &selection.current_item)
        && let Some(cat_data) = data.categories.get(cat_name)
        && let Some(item_data) = cat_data.items.get(item_name)
        && let Some(entry) = item_data.scores.get(delete_idx)
    {
        target_info = format!("{}回目: {}", delete_idx + 1, entry.score);
    }

    egui::Window::new("削除確認")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("このスコアを削除しますか？");
            ui.label(egui::RichText::new(target_info).strong());
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("はい").clicked() {
                    action = Some(Action::ExecuteDeleteScore(delete_idx));
                }
                if ui.button("いいえ").clicked() {
                    open = false;
                }
            });
        });

    if !open {
        *should_close = true;
    }
    action
}
