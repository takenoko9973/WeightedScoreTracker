use crate::app::{ModalType, SelectionState, UiState};
use crate::models::{AppData, MAX_DECAY_RATE, MIN_DECAY_RATE};
use crate::ui::Action;
use eframe::egui;

pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    if state.error_message.is_some() {
        draw_error_dialog(ctx, state);
    }

    let mut should_close = false;

    match &mut state.active_modal {
        ModalType::None => {}
        ModalType::AddCategory { input_name } => {
            action = draw_add_category_window(ctx, input_name, &mut should_close);
        }
        ModalType::RenameCategory {
            target,
            input_new_name,
        } => {
            action = draw_rename_category_window(ctx, target, input_new_name, &mut should_close);
        }
        ModalType::AddItem {
            target_category,
            input_name,
            input_decay,
        } => {
            action = draw_add_item_window(
                ctx,
                target_category,
                input_name,
                input_decay,
                &mut should_close,
            );
        }
        ModalType::EditDecay { input_decay } => {
            // 現在の対象情報は state.selection から取得する必要があるため、少し特殊
            if let (Some(cat), Some(item)) = (
                &state.selection.current_category,
                &state.selection.current_item,
            ) {
                action = draw_edit_decay_window(ctx, cat, item, input_decay, &mut should_close);
            } else {
                should_close = true; // 対象がないなら閉じる
            }
        }
        ModalType::ConfirmDeleteCategory { target } => {
            action = draw_delete_category_confirm(ctx, target, &mut should_close);
        }
        ModalType::ConfirmDeleteItem {
            target_cat,
            target_item,
        } => {
            action = draw_delete_item_confirm(ctx, target_cat, target_item, &mut should_close);
        }
        ModalType::ConfirmDeleteScore { index } => {
            action =
                draw_delete_score_confirm(ctx, data, &state.selection, *index, &mut should_close);
        }
    }

    if should_close {
        state.active_modal = ModalType::None;
    }

    action
}

fn draw_error_dialog(ctx: &egui::Context, state: &mut UiState) {
    if state.error_message.is_none() {
        return;
    }

    let mut open = true;
    let mut should_close = false;
    let msg = state.error_message.as_ref().unwrap().clone();

    egui::Window::new("エラー")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(msg);
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                if ui.button("OK").clicked() {
                    should_close = true;
                }
            });
        });

    if !open || should_close {
        state.error_message = None;
    }
}

/// カテゴリ追加ウィンドウ
fn draw_add_category_window(
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

fn draw_rename_category_window(
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

fn draw_add_item_window(
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

fn draw_edit_decay_window(
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

fn draw_delete_category_confirm(
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

fn draw_delete_item_confirm(
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

fn draw_delete_score_confirm(
    ctx: &egui::Context,
    data: &AppData,
    selection_state: &SelectionState,
    delete_idx: usize,
    should_close: &mut bool,
) -> Option<Action> {
    let mut action = None;
    let mut open = true;

    // 削除対象のスコア情報を取得して表示
    let mut target_info = String::new();
    if let (Some(cat_name), Some(item_name)) = (
        &selection_state.current_category,
        &selection_state.current_item,
    ) && let Some(cat_data) = data.categories.get(cat_name)
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
