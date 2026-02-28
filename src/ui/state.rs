use crate::ui::modals::Modal;

#[derive(Default)]
pub struct UiState {
    /// 現在開いているモーダル
    pub active_modal: Option<Box<dyn Modal>>,

    /// エラーメッセージ
    pub error_message: Option<String>,
}
