use crate::action::Action;
use crate::domain::TrackerModel;
use crate::persistence::{load_data, save_data};
use crate::ui::central_panel::CentralPanel;
use crate::ui::modals::ModalLayer;
use crate::ui::modals::add_category::AddCategoryModal;
use crate::ui::modals::add_item::AddItemModal;
use crate::ui::modals::confirm::ConfirmationModal;
use crate::ui::modals::edit_category::EditCategoryModal;
use crate::ui::modals::edit_decay::EditDecayModal;
use crate::ui::modals::edit_item::EditItemModal;
use crate::ui::side_panel::SidePanel;
use crate::ui::state::UiState;
use eframe::egui;

fn decay_str_parse(rate_str: &str) -> Result<f64, String> {
    match rate_str.parse::<f64>() {
        Ok(v) => Ok(v),
        Err(_) => Err("有効な数値を入力してください。".to_string()),
    }
}

// アプリケーション状態保存
pub struct WeightedScoreTracker {
    model: TrackerModel,
    state: UiState,

    side_panel: SidePanel,
    central_panel: CentralPanel,
    modal_layer: ModalLayer,
}

impl WeightedScoreTracker {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let data = load_data().unwrap_or_default();
        Self {
            model: TrackerModel::new(data),
            state: UiState::default(),

            side_panel: SidePanel::new(),
            central_panel: CentralPanel::new(),
            modal_layer: ModalLayer::new(),
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            // モーダル表示系
            Action::ShowAddCategoryModal => {
                self.modal_layer.open(AddCategoryModal::new());
            }
            Action::ShowAddItemModal(cat_name) => {
                self.modal_layer.open(AddItemModal::new(cat_name));
            }
            Action::ShowEditCategoryModal(cat_name) => {
                self.modal_layer.open(EditCategoryModal::new(cat_name));
            }
            Action::ShowEditItemModal(cat_name, item_name) => {
                // モデルからデータを取得してモーダルに渡す
                match self.model.get_item(&cat_name, &item_name) {
                    Ok(item) => {
                        let decay_rate = item.decay_rate;
                        let mut categories: Vec<_> =
                            self.model.data.categories.keys().cloned().collect();
                        categories.sort();

                        self.modal_layer.open(EditItemModal::new(
                            cat_name, item_name, decay_rate, categories,
                        ));
                    }
                    Err(e) => self.state.error_message = Some(e.to_string()),
                }
            }
            Action::ShowEditDecayModal(decay_rate) => {
                let Some(cat_name) = self.model.selection.category.clone() else {
                    return;
                };
                let Some(item_name) = self.model.selection.item.clone() else {
                    return;
                };

                self.modal_layer
                    .open(EditDecayModal::new(cat_name, item_name, decay_rate));
            }
            Action::ShowDeleteCategoryConfirm(cat_name) => {
                self.modal_layer
                    .open(ConfirmationModal::new_delete_category(cat_name));
            }
            Action::ShowDeleteItemConfirm(cat_name, item_name) => {
                self.modal_layer
                    .open(ConfirmationModal::new_delete_item(cat_name, item_name));
            }
            Action::ShowDeleteScoreConfirm(index) => {
                self.modal_layer
                    .open(ConfirmationModal::new_delete_score(index));
            }

            // データ操作系
            Action::SelectItem(cat, item) => {
                self.model.select_item(cat, item);

                // カテゴリが変わったら入力欄をリセット
                self.central_panel.clear_input();
            }
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
        if let Err(e) = save_data(&self.model.data) {
            self.state.error_message = Some(format!("保存に失敗しました: {}", e));
        }
    }

    // ======================================
    // データ操作
    // ======================================

    /// カテゴリ登録
    fn add_category(&mut self, name: String) {
        match self.model.add_category(name) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = None;
            }
            Err(e) => self.state.error_message = Some(e.to_string()),
        }
    }

    /// カテゴリ名変更
    fn rename_category(&mut self, old_name: String, new_name: String) {
        match self.model.rename_category(&old_name, new_name) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = None;
            }
            Err(e) => self.state.error_message = Some(e.to_string()),
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

        match self.model.add_item(&cat_name, name, decay_rate) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = None;
            }
            Err(e) => self.state.error_message = Some(e.to_string()),
        }
    }

    /// スコア追加
    fn add_score(&mut self, text: String) {
        let (Some(cat), Some(item)) = (&self.model.selection.category, &self.model.selection.item)
        else {
            return;
        };

        let score = match text.parse::<i64>() {
            Ok(v) => v,
            Err(_) => {
                self.state.error_message = Some("スコアには整数値を入力してください。".to_string());
                return;
            }
        };

        let cat = cat.clone();
        let item = item.clone();

        // 追加処理（マイナスチェックなどはモデル内で実行）
        match self.model.add_score(&cat, &item, score) {
            Ok(_) => {
                self.central_panel.clear_input();
                self.save_to_file();
            }
            Err(e) => self.state.error_message = Some(e.to_string()),
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
        let decay = match decay_str_parse(&decay_str) {
            Ok(v) => v,
            Err(msg) => {
                self.state.error_message = Some(msg);
                return;
            }
        };

        let old_loc = (old_cat.as_str(), old_item.as_str());
        let new_loc = (new_cat.as_str(), new_item.as_str());
        match self.model.update_item(old_loc, new_loc, decay) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = None;
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// 減衰率変更
    fn update_decay_rate(&mut self, decay_str: String) {
        let (Some(cat), Some(item)) = (&self.model.selection.category, &self.model.selection.item)
        else {
            return;
        };

        let decay = match decay_str_parse(&decay_str) {
            Ok(v) => v,
            Err(msg) => {
                self.state.error_message = Some(msg);
                return;
            }
        };

        let cat = cat.clone();
        let item = item.clone();
        match self.model.update_decay(&cat, &item, decay) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = None;
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// カテゴリ削除実行
    fn execute_delete_category(&mut self, name: String) {
        // モデルの処理を呼び出し、結果で分岐
        match self.model.remove_category(&name) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = None;
            }
            Err(e) => {
                self.state.error_message = Some(e.to_string());
            }
        }
    }

    /// 項目削除
    fn execute_delete_item(&mut self, cat: String, item: String) {
        match self.model.remove_item(&cat, &item) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = None;
            }
            Err(msg) => {
                self.state.error_message = Some(msg);
            }
        }
    }

    /// スコア削除
    fn execute_delete_score(&mut self, idx: usize) {
        let (Some(cat), Some(item)) = (&self.model.selection.category, &self.model.selection.item)
        else {
            return;
        };

        let cat = cat.clone();
        let item = item.clone();
        match self.model.remove_score(&cat, &item, idx) {
            Ok(_) => {
                self.save_to_file();
                self.state.active_modal = None;
            }
            Err(msg) => {
                self.state.error_message = Some(msg);
            }
        }
    }
}

impl eframe::App for WeightedScoreTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // モーダルが開いているかどうか（通常モーダル or エラーメッセージ）
        let is_modal_open = self.modal_layer.is_open() || self.state.error_message.is_some();
        let is_panel_enabled = !is_modal_open; // 開いている場合は無効化

        let side_act = self.side_panel.show(ctx, &self.model, is_panel_enabled);
        let central_act = self.central_panel.show(ctx, &self.model, is_panel_enabled);

        let modal_act = self.modal_layer.show(ctx, &mut self.state);

        let action = modal_act.or(side_act).or(central_act);

        if let Some(act) = action {
            self.handle_action(act);
        }
    }
}
