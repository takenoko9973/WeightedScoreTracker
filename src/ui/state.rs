use crate::{constants::DEFAULT_DECAY_RATE, ui::modals::types::ModalType};

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
    pub active_modal: ModalType,

    /// エラーメッセージ（グローバル）
    pub error_message: Option<String>,
}

impl UiState {
    /// カテゴリ追加
    pub fn open_add_category_modal(&mut self) {
        self.active_modal = ModalType::AddCategory {
            input_name: String::new(),
        };
    }

    /// カテゴリ名変更
    pub fn open_rename_category_modal(&mut self, cat_name: String) {
        self.active_modal = ModalType::RenameCategory {
            target: cat_name,
            input_new_name: String::new(),
        };
    }

    /// 項目追加
    pub fn open_add_item_modal(&mut self, cat_name: String) {
        self.active_modal = ModalType::AddItem {
            target_category: cat_name,
            input_name: String::new(),
            input_decay: DEFAULT_DECAY_RATE.to_string(),
        };
    }

    pub fn open_edit_item_modal(&mut self, cat_name: String, item_name: String, decay_rate: f64) {
        self.active_modal = ModalType::EditItem {
            target_cat: cat_name.clone(),
            target_item: item_name.clone(),
            input_cat: cat_name,
            input_item: item_name,
            input_decay: decay_rate.to_string(),
        };
    }

    /// 減衰率変更
    pub fn open_edit_decay_modal(&mut self, decay_rate: f64) {
        self.active_modal = ModalType::EditDecay {
            input_decay: decay_rate.to_string(),
        };
    }

    // ==============================================================

    // 削除確認系
    /// カテゴリ削除確認
    pub fn show_delete_category_confirm_modal(&mut self, cat_name: String) {
        self.active_modal = ModalType::ConfirmDeleteCategory { target: cat_name }
    }

    /// 項目削除確認
    pub fn show_delete_item_confirm_modal(&mut self, cat_name: String, item_name: String) {
        self.active_modal = ModalType::ConfirmDeleteItem {
            target_cat: cat_name,
            target_item: item_name,
        }
    }

    /// スコア削除確認
    pub fn show_delete_score_confirm_modal(&mut self, index: usize) {
        self.active_modal = ModalType::ConfirmDeleteScore { index }
    }
}
