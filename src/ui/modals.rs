use crate::app::UiState;
use crate::models::AppData;
use eframe::egui;

pub fn draw(ctx: &egui::Context, data: &mut AppData, state: &mut UiState) -> bool {
    let mut save_needed = false;

    // エラーダイアログ
    draw_error_dialog(ctx, state);

    // 削除確認ダイアログ
    if draw_delete_score_confirm_dialog(ctx, data, state) {
        save_needed = true;
    }

    if draw_delete_category_confirm_dialog(ctx, data, state) {
        save_needed = true;
    }

    // カテゴリ追加ウィンドウ
    if draw_add_category_window(ctx, data, state) {
        save_needed = true;
    }

    // 減衰率変更ウィンドウ
    if draw_edit_decay_window(ctx, data, state) {
        save_needed = true;
    }

    save_needed
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
    data: &mut AppData,
    state: &mut UiState,
) -> bool {
    let Some(delete_idx) = state.pending_delete_index else {
        // 未選択の場合は即リターン
        return false;
    };

    let mut save_needed = false;

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
            ui.label("この記録を削除しますか？");
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

    if confirmed && let Some(cat) = &state.current_category {
        data.remove_score(cat, delete_idx);

        save_needed = true;
        state.selected_history_index = None;
    }

    if !open || should_close {
        state.pending_delete_index = None;
    }

    save_needed
}

fn draw_delete_category_confirm_dialog(
    ctx: &egui::Context,
    data: &mut AppData,
    state: &mut UiState,
) -> bool {
    let Some(target_name) = &state.pending_delete_category else {
        return false;
    };

    let mut save_needed = false;

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
        data.remove_category(target_name);

        // 削除したカテゴリが表示中だった場合、選択を解除する
        if state.current_category.as_ref() == Some(target_name) {
            state.current_category = None;
            state.input_score.clear();
        }
        save_needed = true;
    }

    if !open || should_close {
        state.pending_delete_category = None;
    }

    save_needed
}

fn draw_add_category_window(ctx: &egui::Context, data: &mut AppData, state: &mut UiState) -> bool {
    if !state.show_add_category_window {
        return false;
    }

    let mut save_needed = false;

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
                if ui.button("追加").clicked() && !state.input_category.is_empty() {
                    let rate = state.input_decay.parse::<f64>().unwrap_or(0.95);
                    data.add_category(state.input_category.clone(), rate);

                    save_needed = true;
                    close_requested = true;
                }

                if ui.button("キャンセル").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open || close_requested {
        state.show_add_category_window = false;
    }

    save_needed
}

fn draw_edit_decay_window(ctx: &egui::Context, data: &mut AppData, state: &mut UiState) -> bool {
    if !state.show_edit_decay_window {
        return false;
    }

    let mut save_needed = false;

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
                    if let Some(cat) = &state.current_category
                        && let Ok(rate) = state.input_decay.parse::<f64>()
                    {
                        data.update_decay_rate(cat, rate);
                        save_needed = true;
                    }
                    close_requested = true;
                }

                if ui.button("キャンセル").clicked() {
                    close_requested = true;
                }
            });
        });

    if !open || close_requested {
        state.show_edit_decay_window = false;
    }

    save_needed
}
