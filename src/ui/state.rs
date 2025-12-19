use crate::ui::modals::Modal;

/// メイン画面での選択・入力状態
#[derive(Default)]
pub struct SelectionState {
    pub current_category: Option<String>,
    pub current_item: Option<String>,
    pub selected_history_index: Option<usize>,
    pub input_score: String,
}

#[derive(Default)]
pub struct UiState {
    /// 常駐する画面の選択状態や入力欄
    pub selection: SelectionState,

    /// 現在開いているモーダル
    pub active_modal: Option<Box<dyn Modal>>,

    /// エラーメッセージ
    pub error_message: Option<String>,
}
