pub mod add_category;
pub mod add_item;
pub mod confirm;
pub mod edit_category;
pub mod edit_decay;
pub mod edit_item;
pub mod error;

use crate::action::Action;
use crate::ui::modals::error::ErrorModal;
use crate::ui::state::UiState;
use eframe::egui;

/// モーダル管理
pub struct ModalLayer {
    active_modal: Option<Box<dyn Modal>>,
}

impl ModalLayer {
    pub fn new() -> Self {
        Self { active_modal: None }
    }

    /// モーダルを開く
    pub fn open<M: Modal + 'static>(&mut self, modal: M) {
        self.active_modal = Some(Box::new(modal));
    }

    /// モーダルを閉じる
    pub fn close(&mut self) {
        self.active_modal = None;
    }

    /// 描画処理
    pub fn show(&mut self, ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
        let mut action = None;
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
        if let Some(modal) = &mut self.active_modal {
            match modal.show(ctx) {
                ModalResult::KeepOpen => {}
                ModalResult::Close => should_close = true,
                ModalResult::Dispatch(act) => {
                    action = Some(act);
                    should_close = true;
                }
            }
        }

        if should_close {
            self.active_modal = None;
        }

        action
    }

    /// モーダルが開いているかどうか
    pub fn is_open(&self) -> bool {
        self.active_modal.is_some()
    }
}

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
