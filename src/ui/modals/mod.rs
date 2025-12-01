pub mod category;
pub mod confirm;
pub mod error;
pub mod item;

use crate::models::AppData;
use crate::ui::Action;
use crate::ui::state::{ModalType, UiState};
use eframe::egui;

/// モーダル描画のエントリーポイント
pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    // 1. エラーダイアログ (グローバル)
    if state.error_message.is_some() {
        error::show(ctx, state);
    }

    // 2. アクティブなモーダルの描画
    let mut should_close = false;

    match &mut state.active_modal {
        ModalType::None => {}

        ModalType::AddCategory { input_name } => {
            action = category::show_add(ctx, input_name, &mut should_close);
        }

        ModalType::RenameCategory {
            target,
            input_new_name,
        } => {
            action = category::show_rename(ctx, target, input_new_name, &mut should_close);
        }

        ModalType::AddItem {
            target_category,
            input_name,
            input_decay,
        } => {
            action = item::show_add(
                ctx,
                target_category,
                input_name,
                input_decay,
                &mut should_close,
            );
        }

        ModalType::EditDecay { input_decay } => {
            if let (Some(cat), Some(item_name)) = (
                &state.selection.current_category,
                &state.selection.current_item,
            ) {
                action = item::show_edit_decay(ctx, cat, item_name, input_decay, &mut should_close);
            } else {
                should_close = true;
            }
        }

        ModalType::ConfirmDeleteCategory { target } => {
            action = confirm::show_delete_category(ctx, target, &mut should_close);
        }

        ModalType::ConfirmDeleteItem {
            target_cat,
            target_item,
        } => {
            action = confirm::show_delete_item(ctx, target_cat, target_item, &mut should_close);
        }

        ModalType::ConfirmDeleteScore { index } => {
            // SelectionState のみを渡す
            action =
                confirm::show_delete_score(ctx, data, &state.selection, *index, &mut should_close);
        }
    }

    // 3. 閉じるフラグの処理
    if should_close {
        state.active_modal = ModalType::None;
    }

    action
}
