use crate::models::{AppData, DEFAULT_DECAY_RATE};
use crate::persistence::{load_data, save_data};
use crate::ui::Action;
use crate::ui::{central_panel, modals, side_panel};
use eframe::egui;

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

// アプリケーション状態保存
pub struct ScoreTracker {
    data: AppData,
    state: UiState,
}

impl ScoreTracker {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let data = load_data().unwrap_or_default();
        Self {
            data,
            state: UiState::default(),
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            // モーダル表示系
            Action::ShowAddCategoryModal => self.open_add_category_modal(),
            Action::ShowRenameCategoryModal(now_name) => self.open_rename_category_modal(now_name),
            Action::ShowAddItemModal(cat_name) => self.open_add_item_modal(cat_name),
            Action::ShowEditDecayModal => self.open_edit_decay_modal(),
            Action::ShowDeleteCategoryConfirm(name) => {
                self.state.active_modal = ModalType::ConfirmDeleteCategory { target: name };
            }
            Action::ShowDeleteItemConfirm(cat, item) => {
                self.state.active_modal = ModalType::ConfirmDeleteItem {
                    target_cat: cat,
                    target_item: item,
                };
            }
            Action::ShowDeleteScoreConfirm(idx) => {
                self.state.active_modal = ModalType::ConfirmDeleteScore { index: idx }
            }

            // データ操作系
            Action::SelectItem(cat, item) => self.select_item(cat, item),
            Action::AddCategory(name) => self.add_category(name),
            Action::RenameCategory(old_name, new_name) => self.rename_category(old_name, new_name),
            Action::AddItem(cat, name, decay) => self.add_item(cat, name, decay),
            Action::AddScore(text) => self.add_score(text),
            Action::UpdateDecayRate(rate) => self.update_decay_rate(rate),
            Action::ExecuteDeleteCategory(name) => self.execute_delete_category(name),
            Action::ExecuteDeleteItem(cat, item) => self.execute_delete_item(cat, item),
            Action::ExecuteDeleteScore(idx) => self.execute_delete_score(idx),
        };
    }

    // ======================================
    // 共通処理
    // ======================================
    fn save_to_file(&mut self) {
        if let Err(e) = save_data(&self.data) {
            self.state.error_message = Some(format!("保存に失敗しました: {}", e));
        }
    }

    // ======================================
    // モーダル表示
    // ======================================

    /// カテゴリ登録
    fn open_add_category_modal(&mut self) {
        self.state.active_modal = ModalType::AddCategory {
            input_name: String::new(),
        };
    }

    fn open_rename_category_modal(&mut self, target_cat: String) {
        self.state.active_modal = ModalType::RenameCategory {
            input_new_name: target_cat.clone(), // 初期値は現在の名前
            target: target_cat.clone(),
        };
    }

    fn open_add_item_modal(&mut self, cat_name: String) {
        self.state.active_modal = ModalType::AddItem {
            target_category: cat_name,
            input_name: String::new(),
            input_decay: DEFAULT_DECAY_RATE.to_string(),
        };
    }

    /// 減衰率変更
    fn open_edit_decay_modal(&mut self) {
        let (Some(c), Some(i)) = (
            &self.state.selection.current_category,
            &self.state.selection.current_item,
        ) else {
            return;
        };

        if let Some(cat) = self.data.categories.get(c)
            && let Some(item) = cat.items.get(i)
        {
            self.state.active_modal = ModalType::EditDecay {
                input_decay: item.decay_rate.to_string(),
            };
        }
    }

    // ======================================
    // データ操作
    // ======================================

    /// 項目選択
    fn select_item(&mut self, cat: String, item: String) {
        self.state.selection.current_category = Some(cat);
        self.state.selection.current_item = Some(item);

        // カテゴリが変わったら入力欄と選択状態をリセット
        self.state.selection.input_score.clear();
        self.state.selection.selected_history_index = None;
    }

    /// カテゴリ登録
    fn add_category(&mut self, name: String) {
        match self.data.try_add_category(name) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = ModalType::None;
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// カテゴリ名変更
    fn rename_category(&mut self, old_name: String, new_name: String) {
        if old_name == new_name {
            self.state.active_modal = ModalType::None;
            return;
        }

        match self.data.try_rename_category(&old_name, new_name.clone()) {
            Ok(_) => {
                if self.state.selection.current_category.as_ref() == Some(&old_name) {
                    self.state.selection.current_category = Some(new_name);
                }
                self.save_to_file();
                self.state.active_modal = ModalType::None;
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// 項目追加
    fn add_item(&mut self, cat_name: String, name: String, decay_str: String) {
        // UI層で変換を行う
        let decay_rate = match decay_str.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                self.state.error_message =
                    Some("減衰率には有効な数値を入力してください。".to_string());
                return;
            }
        };

        match self.data.try_add_item(&cat_name, name, decay_rate) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = ModalType::None;
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// スコア追加
    fn add_score(&mut self, text: String) {
        let (Some(cat), Some(item)) = (
            &self.state.selection.current_category,
            &self.state.selection.current_item,
        ) else {
            return;
        };

        let score = match text.parse::<i32>() {
            Ok(v) => v,
            Err(_) => {
                self.state.error_message = Some("スコアには整数値を入力してください。".to_string());
                return;
            }
        };

        // 追加処理（マイナスチェックなどはモデル内で実行）
        match self.data.try_add_score(cat, item, score) {
            Ok(_) => {
                self.state.selection.input_score.clear();
                self.save_to_file();
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// 減衰率変更
    fn update_decay_rate(&mut self, rate_str: String) {
        let (Some(cat), Some(item)) = (
            &self.state.selection.current_category,
            &self.state.selection.current_item,
        ) else {
            return;
        };

        let rate = match rate_str.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                self.state.error_message = Some("有効な数値を入力してください。".to_string());
                return;
            }
        };

        match self.data.try_update_decay_rate(cat, item, rate) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = ModalType::None;
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// カテゴリ削除実行
    fn execute_delete_category(&mut self, name: String) {
        // モデルの処理を呼び出し、結果で分岐
        match self.data.try_remove_category(&name) {
            Ok(_) => {
                if self.state.selection.current_category.as_ref() == Some(&name) {
                    self.state.selection.current_category = None;
                    self.state.selection.current_item = None;
                }
                self.save_to_file();
                self.state.active_modal = ModalType::None;
            }
            Err(msg) => {
                self.state.error_message = Some(msg);
            }
        }
    }

    /// 項目削除
    fn execute_delete_item(&mut self, cat: String, item: String) {
        match self.data.try_remove_item(&cat, &item) {
            Ok(_) => {
                if self.state.selection.current_category.as_ref() == Some(&cat)
                    && self.state.selection.current_item.as_ref() == Some(&item)
                {
                    self.state.selection.current_item = None;
                    self.state.selection.selected_history_index = None;
                }
                self.save_to_file();
                self.state.active_modal = ModalType::None;
            }
            Err(msg) => {
                self.state.error_message = Some(msg);
            }
        }
    }

    /// スコア削除
    fn execute_delete_score(&mut self, idx: usize) {
        let (Some(cat), Some(item)) = (
            &self.state.selection.current_category,
            &self.state.selection.current_item,
        ) else {
            return;
        };

        match self.data.try_remove_score(cat, item, idx) {
            Ok(_) => {
                self.state.selection.selected_history_index = None;
                self.save_to_file();
                self.state.active_modal = ModalType::None;
            }
            Err(msg) => {
                self.state.error_message = Some(msg);
            }
        }
    }
}

impl eframe::App for ScoreTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let side_act = side_panel::draw(ctx, &self.data, &mut self.state);
        let central_act = central_panel::draw(ctx, &self.data, &mut self.state);
        let modal_act = modals::draw(ctx, &self.data, &mut self.state);

        let action = modal_act.or(side_act).or(central_act);

        if let Some(act) = action {
            self.handle_action(act);
        }
    }
}
