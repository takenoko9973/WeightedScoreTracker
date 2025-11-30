use crate::app::UiState;
use crate::models::AppData;
use crate::ui::Action;
use eframe::egui;

pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
    draw_error_dialog(ctx, state);

    let act_add_cat = draw_add_category_window(ctx, state);
    let act_add_item = draw_add_item_window(ctx, state);
    let act_edit_decay = draw_edit_decay_window(ctx, state);
    let act_del_cat = draw_delete_category_confirm(ctx, state);
    let act_del_item = draw_delete_item_confirm(ctx, state);
    let act_del_score = draw_delete_score_confirm(ctx, data, state);

    // 結合して返す
    act_add_cat
        .or(act_add_item)
        .or(act_edit_decay)
        .or(act_del_cat)
        .or(act_del_item)
        .or(act_del_score)
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
fn draw_add_category_window(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    if !state.show_add_category_window {
        return None;
    }

    let mut action = None;

    let mut open = true;
    let mut close_requested = false;

    egui::Window::new("カテゴリ追加")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("カテゴリ名:");
            ui.text_edit_singleline(&mut state.input_category);

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("追加").clicked() {
                    action = Some(Action::AddCategory(state.input_category.clone()));
                }

                if ui.button("キャンセル").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open || close_requested {
        state.show_add_category_window = false;
    }

    action
}

fn draw_add_item_window(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    if !state.show_add_item_window {
        return None;
    }
    let Some(target_cat) = &state.target_category_for_new_item else {
        return None;
    };

    let mut action = None;

    let mut open = true;
    let mut close_requested = false;

    egui::Window::new("項目追加")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!("追加先カテゴリ: {}", target_cat));

            ui.label("項目名:");
            ui.text_edit_singleline(&mut state.input_item);
            ui.label("減衰率 (0.01 - 1.00):");
            ui.text_edit_singleline(&mut state.input_decay);

            ui.horizontal(|ui| {
                if ui.button("追加").clicked() {
                    action = Some(Action::AddItem(
                        target_cat.clone(),
                        state.input_item.clone(),
                        state.input_decay.clone(),
                    ));
                }

                if ui.button("キャンセル").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open || close_requested {
        state.show_add_item_window = false;
    }

    action
}

fn draw_edit_decay_window(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    if !state.show_edit_decay_window {
        return None;
    }

    let mut action = None;

    let mut open = true;
    let mut close_requested = false;

    egui::Window::new("減衰率変更")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!(
                "対象: {}",
                state.current_category.as_deref().unwrap_or("")
            ));
            ui.text_edit_singleline(&mut state.input_decay);

            ui.horizontal(|ui| {
                if ui.button("更新").clicked() {
                    action = Some(Action::UpdateDecayRate(state.input_decay.clone()));
                }

                if ui.button("キャンセル").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open || close_requested {
        state.show_edit_decay_window = false;
    }

    action
}

fn draw_delete_category_confirm(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    let Some(cat_name) = &state.pending_delete_category else {
        return None;
    };

    let mut action = None;

    let mut open = true;
    let mut close_requested = false;

    egui::Window::new("削除確認")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!("カテゴリ \"{}\" を削除しますか？", cat_name));
            ui.label(
                egui::RichText::new("※含まれる全ての項目とスコアが完全に消去されます。")
                    .color(egui::Color32::RED),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("削除する").clicked() {
                    action = Some(Action::ExecuteDeleteCategory(cat_name.clone()));
                    close_requested = true;
                }
                if ui.button("キャンセル").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open || close_requested {
        state.pending_delete_category = None;
    }

    action
}

fn draw_delete_item_confirm(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    let Some((cat, item)) = &state.pending_delete_item else {
        return None;
    };

    let mut action = None;

    let mut open = true;
    let mut close_requested = false;

    egui::Window::new("削除確認")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!("\"{}\" を削除しますか？", item));
            ui.label(
                egui::RichText::new("※含まれる全てのスコアデータが完全に消去されます。")
                    .color(egui::Color32::RED),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("削除する").clicked() {
                    action = Some(Action::ExecuteDeleteItem(cat.clone(), item.clone()));
                    close_requested = true;
                }
                if ui.button("キャンセル").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open || close_requested {
        state.pending_delete_item = None;
    }

    action
}

fn draw_delete_score_confirm(
    ctx: &egui::Context,
    data: &AppData,
    state: &mut UiState,
) -> Option<Action> {
    let delete_idx = state.pending_delete_score?;

    let mut action = None;

    let mut open = true;
    let mut close_requested = false;

    let mut target_info = String::new();
    if let (Some(cat_name), Some(item_name)) = (&state.current_category, &state.current_item)
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
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("このスコアを削除しますか？");
            ui.label(egui::RichText::new(target_info).strong());
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("はい").clicked() {
                    action = Some(Action::ExecuteDeleteScore(delete_idx));
                    close_requested = true;
                }
                if ui.button("いいえ").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open || close_requested {
        state.pending_delete_score = None;
    }

    action
}
