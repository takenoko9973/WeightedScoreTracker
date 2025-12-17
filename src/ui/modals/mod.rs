pub mod add_category;
pub mod add_item;
pub mod confirm;
pub mod edit_category;
pub mod edit_decay;
pub mod edit_item;
pub mod error;

use crate::ui::Action;
use crate::ui::modals::error::ErrorModal;
use crate::ui::state::UiState;
use eframe::egui;

/// モーダルの実行結果
pub enum ModalResult {
    KeepOpen,         // 開いたまま
    Close,            // 閉じる（キャンセルなど）
    Dispatch(Action), // アクションを実行して閉じる
}

/// モーダル用インターフェース
pub trait Modal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult;
}

pub fn show_active_modal(ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
    let mut action_to_dispatch = None;
    let mut should_close = false;

    // エラーモーダル
    if let Some(msg) = &state.error_message {
        let mut error_modal = ErrorModal::new(msg.to_string());

        match error_modal.show(ctx) {
            ModalResult::KeepOpen => {}
            ModalResult::Close => {
                state.error_message = None;
            }
            ModalResult::Dispatch(_) => {}
        }
    }

    // 通常モーダル
    if let Some(modal) = &mut state.active_modal {
        match modal.show(ctx) {
            ModalResult::KeepOpen => {}
            ModalResult::Close => should_close = true,
            ModalResult::Dispatch(act) => {
                action_to_dispatch = Some(act);
                should_close = true;
            }
        }
    }

    if should_close {
        state.active_modal = None;
    }

    action_to_dispatch
}
