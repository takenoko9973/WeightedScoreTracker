use crate::action::Action;
use crate::application::TrackerService;
use crate::constants::DATA_FILENAME;
use crate::infrastructure::JsonFileStore;
use crate::ui::central_panel::CentralPanel;
use crate::ui::modals::ModalLayer;
use crate::ui::modals::add_category::AddCategoryModal;
use crate::ui::modals::add_item::AddItemModal;
use crate::ui::modals::confirm::ConfirmationModal;
use crate::ui::modals::edit_category::EditCategoryModal;
use crate::ui::modals::edit_item::EditItemModal;
use crate::ui::modals::tag_manager::TagManagerModal;
use crate::ui::side_panel::SidePanel;
use crate::ui::state::UiState;
use eframe::egui;

// アプリケーション状態保存
pub struct WeightedScoreTracker {
    service: TrackerService<JsonFileStore>,
    state: UiState,

    side_panel: SidePanel,
    central_panel: CentralPanel,
    modal_layer: ModalLayer,
}

impl WeightedScoreTracker {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        disable_label_selection(&cc.egui_ctx);

        let mut state = UiState::default();

        let service = match TrackerService::new(JsonFileStore::new(DATA_FILENAME)) {
            Ok(service) => service,
            Err(err) => {
                state.error_message = Some(err.to_string());
                TrackerService::empty(JsonFileStore::new(DATA_FILENAME))
            }
        };

        Self {
            service,
            state,

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
                let tags = self.service.model().tags().values().cloned().collect();
                self.modal_layer.open(AddItemModal::new(cat_name, tags));
            }
            Action::ShowEditCategoryModal(cat_name) => {
                self.modal_layer.open(EditCategoryModal::new(cat_name));
            }
            Action::ShowEditItemModal(cat_name, item_name) => {
                // モデルからデータを取得してモーダルに渡す
                match self.service.model().get_item(&cat_name, &item_name) {
                    Ok(item) => {
                        let decay_rate = item.decay_rate;
                        let subtitle = item.subtitle.clone();
                        let tag_ids = item.tag_ids.clone();
                        let categories: Vec<_> = self
                            .service
                            .model()
                            .data
                            .ordered_category_names()
                            .into_iter()
                            .map(str::to_owned)
                            .collect();
                        let tags = self.service.model().tags().values().cloned().collect();

                        self.modal_layer.open(EditItemModal::new(
                            cat_name, item_name, decay_rate, subtitle, tag_ids, categories, tags,
                        ));
                    }
                    Err(e) => self.state.error_message = Some(e.to_string()),
                }
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
            Action::ShowTagManagerModal => {
                self.open_tag_manager();
            }

            // データ操作系
            Action::SelectItem(cat, item) => {
                self.service.select_item(cat, item);

                // カテゴリが変わったら入力欄をリセット
                self.central_panel.clear_input();
            }
            Action::AddCategory(name) => self.add_category(name),
            Action::RenameCategory(old_name, new_name) => self.rename_category(old_name, new_name),
            Action::AddItem(cat, name, subtitle, decay, tag_ids) => {
                self.add_item(cat, name, subtitle, decay, tag_ids)
            }
            Action::AddScore(text) => self.add_score(text),
            Action::UpdateItem(
                old_cat,
                old_item,
                new_cat,
                new_name,
                subtitle,
                decay_str,
                tag_ids,
            ) => {
                self.update_item(
                    old_cat, old_item, new_cat, new_name, subtitle, decay_str, tag_ids,
                );
            }
            Action::ExecuteDeleteCategory(name) => self.execute_delete_category(name),
            Action::ExecuteDeleteItem(cat, item) => self.execute_delete_item(cat, item),
            Action::ExecuteDeleteScore(idx) => self.execute_delete_score(idx),
            Action::CreateTag(name, color) => self.create_tag(name, color),
            Action::UpdateTag(id, name, color) => self.update_tag(id, name, color),
            Action::DeleteTag(id) => self.delete_tag(id),
        };
    }

    // ======================================
    // データ操作
    // ======================================

    /// カテゴリ登録
    fn add_category(&mut self, name: String) {
        if let Err(err) = self.service.add_category(name) {
            self.state.error_message = Some(err.to_string());
        }
    }

    /// カテゴリ名変更
    fn rename_category(&mut self, old_name: String, new_name: String) {
        if let Err(err) = self.service.rename_category(&old_name, new_name) {
            self.state.error_message = Some(err.to_string());
        }
    }

    /// 項目追加
    fn add_item(
        &mut self,
        cat_name: String,
        name: String,
        subtitle: String,
        decay_str: String,
        tag_ids: Vec<crate::domain::TagId>,
    ) {
        if let Err(err) = self
            .service
            .add_item(&cat_name, name, subtitle, &decay_str, tag_ids)
        {
            self.state.error_message = Some(err.to_string());
        }
    }

    /// スコア追加
    fn add_score(&mut self, text: String) {
        match self.service.add_score_to_selection(&text) {
            Ok(_) => {
                self.central_panel.clear_input();
            }
            Err(err) => self.state.error_message = Some(err.to_string()),
        }
    }

    /// 項目の更新処理
    #[allow(clippy::too_many_arguments)]
    fn update_item(
        &mut self,
        old_cat: String,
        old_item: String,
        new_cat: String,
        new_item: String,
        subtitle: String,
        decay_str: String,
        tag_ids: Vec<crate::domain::TagId>,
    ) {
        let old_loc = (old_cat.as_str(), old_item.as_str());
        let new_loc = (new_cat.as_str(), new_item.as_str());

        if let Err(err) = self
            .service
            .update_item(old_loc, new_loc, subtitle, &decay_str, tag_ids)
        {
            self.state.error_message = Some(err.to_string());
        }
    }

    fn create_tag(&mut self, name: String, color: [u8; 3]) {
        match self.service.create_tag(name, color) {
            Ok(_) => self.open_tag_manager(),
            Err(err) => self.state.error_message = Some(err.to_string()),
        }
    }

    fn update_tag(&mut self, id: crate::domain::TagId, name: String, color: [u8; 3]) {
        match self.service.update_tag(id, name, color) {
            Ok(()) => self.open_tag_manager(),
            Err(err) => self.state.error_message = Some(err.to_string()),
        }
    }

    fn delete_tag(&mut self, id: crate::domain::TagId) {
        if let Err(err) = self.service.delete_tag(id) {
            self.state.error_message = Some(err.to_string());
        } else {
            self.state.selected_tag_ids.remove(&id);
            self.open_tag_manager();
        }
    }

    fn open_tag_manager(&mut self) {
        let tags = self.service.model().tags().values().cloned().collect();
        self.modal_layer.open(TagManagerModal::new(tags));
    }

    /// カテゴリ削除実行
    fn execute_delete_category(&mut self, name: String) {
        if let Err(err) = self.service.delete_category(&name) {
            self.state.error_message = Some(err.to_string());
        }
    }

    /// 項目削除
    fn execute_delete_item(&mut self, cat: String, item: String) {
        if let Err(err) = self.service.delete_item(&cat, &item) {
            self.state.error_message = Some(err.to_string());
        }
    }

    /// スコア削除
    fn execute_delete_score(&mut self, idx: usize) {
        if let Err(err) = self.service.delete_score_from_selection(idx) {
            self.state.error_message = Some(err.to_string());
        }
    }
}

fn disable_label_selection(ctx: &egui::Context) {
    // システムテーマ切替後も表示専用ラベルを選択不可に保つため、両テーマのスタイルへ適用する。
    ctx.all_styles_mut(|style| {
        style.interaction.selectable_labels = false;
        style.interaction.multi_widget_text_select = false;
    });
}

impl eframe::App for WeightedScoreTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // モーダルが開いているかどうか（通常モーダル or エラーメッセージ）
        let is_modal_open = self.modal_layer.is_open() || self.state.error_message.is_some();
        let is_panel_enabled = !is_modal_open; // 開いている場合は無効化

        let side_act =
            self.side_panel
                .show(ctx, self.service.model(), &mut self.state, is_panel_enabled);
        let central_act = self
            .central_panel
            .show(ctx, self.service.model(), is_panel_enabled);

        let modal_act = self.modal_layer.show(ctx, &mut self.state);

        let action = modal_act.or(side_act).or(central_act);

        if let Some(act) = action {
            self.handle_action(act);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_selection_is_disabled_for_both_themes() {
        let ctx = egui::Context::default();

        disable_label_selection(&ctx);

        for theme in [egui::Theme::Dark, egui::Theme::Light] {
            let interaction = &ctx.style_of(theme).interaction;
            assert!(!interaction.selectable_labels);
            assert!(!interaction.multi_widget_text_select);
        }
    }
}
