use crate::domain::SelectionState;
use crate::ui::modals::Modal;

#[derive(Default)]
pub struct UiState {
    /// 常駐する画面の選択状態や入力欄
    pub selection: SelectionState,

    /// 現在開いているモーダル
    pub active_modal: Option<Box<dyn Modal>>,

    /// エラーメッセージ
    pub error_message: Option<String>,
}
