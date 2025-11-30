use crate::app::UiState;
use crate::models::AppData;
use crate::ui::Action;
use eframe::egui;

pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
    // エラーダイアログ
    draw_error_dialog(ctx, state);

    // カテゴリ追加ウィンドウ
    let add_category_action = draw_add_category_window(ctx, state);

    // 減衰率変更ウィンドウ
    let edit_decay_action = draw_edit_decay_window(ctx, state);

    // カテゴリ削除確認ダイアログ
    let del_category_action = draw_delete_category_confirm_dialog(ctx, state);

    // スコア削除確認ダイアログ
    let del_score_action = draw_delete_score_confirm_dialog(ctx, data, state);

    add_category_action
        .or(edit_decay_action)
        .or(del_score_action)
        .or(del_category_action)
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

fn draw_delete_score_confirm_dialog(
    ctx: &egui::Context,
    data: &AppData,
    state: &mut UiState,
) -> Option<Action> {
    let mut action = None;

    let Some(delete_idx) = state.pending_delete_index else {
        // 未選択の場合は即リターン
        return action;
    };

    let mut open = true;
    let mut should_close = false;
    let mut confirmed = false;

    let mut target_info = String::new();
    if let Some(cat) = &state.current_category
        && let Some(d) = data.categories.get(cat)
        && let Some(entry) = d.scores.get(delete_idx)
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
                    confirmed = true;
                    should_close = true;
                }
                if ui.button("いいえ").clicked() {
                    should_close = true;
                }
            });
        });

    if confirmed {
        action = Some(Action::ExecuteDeleteScore(delete_idx));
    }

    if !open || should_close {
        state.pending_delete_index = None;
    }

    action
}

fn draw_delete_category_confirm_dialog(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    let Some(target_name) = &state.pending_delete_category else {
        return action;
    };

    let mut open = true;
    let mut should_close = false;
    let mut confirmed = false;

    // 借用エラー回避のため名前をクローン
    // let target_name = &cat_name.clone();

    egui::Window::new("削除確認")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(format!("\"{}\" を削除しますか？", target_name));
            ui.label(
                egui::RichText::new("※含まれる全てのスコアデータが完全に消去されます。")
                    .color(egui::Color32::RED),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("削除する").clicked() {
                    confirmed = true;
                    should_close = true;
                }
                if ui.button("キャンセル").clicked() {
                    should_close = true;
                }
            });
        });

    if confirmed {
        action = Some(Action::ExecuteDeleteCategory(target_name.clone()));
    }

    if !open || should_close {
        state.pending_delete_category = None;
    }

    action
}

fn draw_add_category_window(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    if !state.show_add_category_window {
        return action;
    }

    let mut open = true;
    let mut close_requested = false;

    egui::Window::new("項目追加")
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("項目名:");
            ui.text_edit_singleline(&mut state.input_category);
            ui.label("減衰率 (0.01 - 1.00):");
            ui.text_edit_singleline(&mut state.input_decay);

            ui.horizontal(|ui| {
                if ui.button("追加").clicked() {
                    action = Some(Action::AddCategory(
                        state.input_category.clone(),
                        state.input_decay.clone(),
                    ));
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

fn draw_edit_decay_window(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    if !state.show_edit_decay_window {
        return action;
    }

    let mut open = true;
    let mut close_requested = false;

    egui::Window::new("減衰率変更")
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
