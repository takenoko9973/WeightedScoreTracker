use crate::models::{MAX_DECAY_RATE, MIN_DECAY_RATE};
use crate::ui::Action;
use eframe::egui;

pub fn show_add(
    ctx: &egui::Context,
    target_cat: &str,
    input_name: &mut String,
    input_decay: &mut String,
    should_close: &mut bool,
) -> Option<Action> {
    let mut action = None;
    let mut open = true;

    egui::Window::new("項目追加")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(format!("追加先カテゴリ: {}", target_cat));
            ui.label("項目名:");
            ui.text_edit_singleline(input_name);
            ui.label(format!(
                "減衰率 ({:.2} - {:.2}):",
                MIN_DECAY_RATE, MAX_DECAY_RATE
            ));
            ui.text_edit_singleline(input_decay);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("追加").clicked() {
                    action = Some(Action::AddItem(
                        target_cat.to_string(),
                        input_name.clone(),
                        input_decay.clone(),
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

pub fn show_edit_decay(
    ctx: &egui::Context,
    cat_name: &str,
    item_name: &str,
    input_decay: &mut String,
    should_close: &mut bool,
) -> Option<Action> {
    let mut action = None;
    let mut open = true;

    egui::Window::new("減衰率変更")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(format!("対象: {} > {}", cat_name, item_name));
            ui.label(format!(
                "新しい減衰率 ({:.2} - {:.2}):",
                MIN_DECAY_RATE, MAX_DECAY_RATE
            ));
            ui.text_edit_singleline(input_decay);
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("更新").clicked() {
                    action = Some(Action::UpdateDecayRate(input_decay.clone()));
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
