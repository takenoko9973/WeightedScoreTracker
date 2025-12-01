#[derive(Default)]
pub struct UiState {
    /// 常駐する画面の選択状態や入力欄
    pub selection: SelectionState,

    /// 現在開いているモーダル
    pub active_modal: ModalType,

    /// エラーメッセージ（グローバル）
    pub error_message: Option<String>,
}

/// メイン画面での選択・入力状態
#[derive(Default)]
pub struct SelectionState {
    pub current_category: Option<String>,
    pub current_item: Option<String>,
    pub selected_history_index: Option<usize>,
    pub input_score: String,
}

//・ モーダルの定義
#[derive(Default)]
pub enum ModalType {
    #[default]
    None,
    // カテゴリ追加画面の状態
    AddCategory {
        input_name: String,
    },
    // カテゴリ名変更画面の状態
    RenameCategory {
        target: String,
        input_new_name: String,
    },
    // 項目追加画面の状態
    AddItem {
        target_category: String,
        input_name: String,
        input_decay: String,
    },
    // 減衰率変更画面の状態
    EditDecay {
        input_decay: String,
    },
    // 削除確認ダイアログ
    ConfirmDeleteCategory {
        target: String,
    },
    ConfirmDeleteItem {
        target_cat: String,
        target_item: String,
    },
    ConfirmDeleteScore {
        index: usize,
    },
}
