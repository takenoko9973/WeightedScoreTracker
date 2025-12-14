use eframe::egui;

use crate::ui::Action;

pub fn show_add(
    ctx: &egui::Context,
    input_name: &mut String,
    should_close: &mut bool,
) -> Option<Action> {
    let mut action = None;
    let mut open = true;

    egui::Window::new("カテゴリ追加")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label("カテゴリ名:");
            ui.text_edit_singleline(input_name);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("追加").clicked() {
                    action = Some(Action::AddCategory(input_name.clone()));
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

pub fn show_rename(
    ctx: &egui::Context,
    target_cat: &str,
    input_new_name: &mut String,
    should_close: &mut bool,
) -> Option<Action> {
    let mut action = None;
    let mut open = true;

    egui::Window::new("カテゴリ名変更")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(format!("対象: {}", target_cat));
            ui.label("新しい名前:");
            ui.text_edit_singleline(input_new_name);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("変更").clicked() {
                    action = Some(Action::RenameCategory(
                        target_cat.to_string(),
                        input_new_name.clone(),
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
