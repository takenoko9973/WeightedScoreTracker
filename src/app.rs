use crate::models::app::AppData;
use crate::persistence::{load_data, save_data};
use crate::ui::Action;
use crate::ui::modals::types::ModalType;
use crate::ui::state::UiState;
use crate::ui::{central_panel, modals, side_panel};
use eframe::egui;

fn decay_str_parse(rate_str: &str) -> Result<f64, String> {
    match rate_str.parse::<f64>() {
        Ok(v) => Ok(v),
        Err(_) => Err("有効な数値を入力してください。".to_string()),
    }
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
            Action::ShowAddCategoryModal => {
                self.state.open_add_category_modal();
            }
            Action::ShowRenameCategoryModal(cat_name) => {
                self.state.open_rename_category_modal(cat_name);
            }
            Action::ShowAddItemModal(cat_name) => {
                self.state.open_add_item_modal(cat_name);
            }
            Action::ShowEditItemModal(cat_name, item_name) => {
                let decay_rate = match self.data.get_item_decay(&cat_name, &item_name) {
                    Ok(item) => item,
                    Err(msg) => {
                        self.state.error_message = Some(msg);
                        return;
                    }
                };
                self.state
                    .open_edit_item_modal(cat_name, item_name, decay_rate);
            }
            Action::ShowEditDecayModal(decay_rate) => self.state.open_edit_decay_modal(decay_rate),
            Action::ShowDeleteCategoryConfirm(cat_name) => {
                self.state.show_delete_category_confirm_modal(cat_name);
            }
            Action::ShowDeleteItemConfirm(cat_name, item_name) => {
                self.state
                    .show_delete_item_confirm_modal(cat_name, item_name);
            }
            Action::ShowDeleteScoreConfirm(index) => {
                self.state.show_delete_score_confirm_modal(index);
            }

            // データ操作系
            Action::SelectItem(cat, item) => self.select_item(cat, item),
            Action::AddCategory(name) => self.add_category(name),
            Action::RenameCategory(old_name, new_name) => self.rename_category(old_name, new_name),
            Action::AddItem(cat, name, decay) => self.add_item(cat, name, decay),
            Action::AddScore(text) => self.add_score(text),
            Action::UpdateItem(old_cat, old_item, new_cat, new_name, decay_str) => {
                self.update_item(old_cat, old_item, new_cat, new_name, decay_str);
            }
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
        match self.data.add_category(name) {
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

        match self.data.rename_category(&old_name, new_name.clone()) {
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

        match self.data.add_item(&cat_name, name, decay_rate) {
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

        let score = match text.parse::<i64>() {
            Ok(v) => v,
            Err(_) => {
                self.state.error_message = Some("スコアには整数値を入力してください。".to_string());
                return;
            }
        };

        // 追加処理（マイナスチェックなどはモデル内で実行）
        match self.data.add_score(cat, item, score) {
            Ok(_) => {
                self.state.selection.input_score.clear();
                self.save_to_file();
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// 項目の更新処理
    fn update_item(
        &mut self,
        old_cat: String,
        old_item: String,
        new_cat: String,
        new_item: String,
        decay_str: String,
    ) {
        // 操作をシミュレーション
        let mut temp_data = self.data.clone();

        let result = (|| -> Result<(), String> {
            // シミュレート
            let decay = decay_str_parse(&decay_str)?;
            temp_data.move_item(&old_cat, &new_cat, &old_item)?;
            temp_data.rename_item(&new_cat, &old_item, new_item.clone())?;
            temp_data.update_decay(&new_cat, &new_item, decay)?;
            Ok(())
        })();

        match result {
            Ok(_) => {
                self.data = temp_data; // 成功した場合はシミュレーション結果に上書き

                // 選択状態の追従 : もし編集していた項目を選択中だったら、選択情報を更新する
                if self.state.selection.current_category.as_ref() == Some(&old_cat)
                    && self.state.selection.current_item.as_ref() == Some(&old_item)
                {
                    self.state.selection.current_category = Some(new_cat);
                    self.state.selection.current_item = Some(new_item);
                }

                self.save_to_file();
                self.state.active_modal = ModalType::None;
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// 減衰率変更
    fn update_decay_rate(&mut self, decay_str: String) {
        let (Some(cat), Some(item)) = (
            &self.state.selection.current_category,
            &self.state.selection.current_item,
        ) else {
            return;
        };

        let decay = match decay_str_parse(&decay_str) {
            Ok(v) => v,
            Err(msg) => {
                self.state.error_message = Some(msg);
                return;
            }
        };

        match self.data.update_decay(cat, item, decay) {
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
        match self.data.remove_category(&name) {
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
        match self.data.remove_item(&cat, &item) {
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

        match self.data.remove_score(cat, item, idx) {
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
