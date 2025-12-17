use crate::ui::modals::{
    Modal, add_category::AddCategoryModal, add_item::AddItemModal, confirm::ConfirmationModal,
    edit_category::EditCategoryModal, edit_decay::EditDecayModal, edit_item::EditItemModal,
};

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

    /// エラーメッセージ（グローバル）
    pub error_message: Option<String>,
}

impl UiState {
    fn open_modal<M: Modal + 'static>(&mut self, modal: M) {
        self.active_modal = Some(Box::new(modal));
    }

    // ==============================================================

    /// カテゴリ追加
    pub fn open_add_category_modal(&mut self) {
        self.open_modal(AddCategoryModal::new());
    }

    /// カテゴリ名変更
    pub fn open_edit_category_modal(&mut self, cat_name: String) {
        self.open_modal(EditCategoryModal::new(cat_name));
    }

    /// 項目追加
    pub fn open_add_item_modal(&mut self, cat_name: String) {
        self.open_modal(AddItemModal::new(cat_name));
    }

    /// 項目名・減衰率変更
    pub fn open_edit_item_modal(
        &mut self,
        cat_name: String,
        item_name: String,
        decay_rate: f64,
        categories: Vec<String>,
    ) {
        self.open_modal(EditItemModal::new(
            cat_name, item_name, decay_rate, categories,
        ));
    }

    /// 減衰率変更
    pub fn open_edit_decay_modal(&mut self, decay_rate: f64) {
        let Some(cat_name) = self.selection.current_category.clone() else {
            self.active_modal = None;
            return;
        };
        let Some(item_name) = self.selection.current_item.clone() else {
            self.active_modal = None;
            return;
        };

        self.open_modal(EditDecayModal::new(cat_name, item_name, decay_rate));
    }

    // ==============================================================

    // 削除確認系
    /// カテゴリ削除確認
    pub fn show_delete_category_confirm_modal(&mut self, cat_name: String) {
        self.open_modal(ConfirmationModal::new_delete_category(cat_name));
    }

    /// 項目削除確認
    pub fn show_delete_item_confirm_modal(&mut self, cat_name: String, item_name: String) {
        self.open_modal(ConfirmationModal::new_delete_item(cat_name, item_name));
    }

    /// スコア削除確認
    pub fn show_delete_score_confirm_modal(&mut self, index: usize) {
        self.open_modal(ConfirmationModal::new_delete_score(index));
    }
}
