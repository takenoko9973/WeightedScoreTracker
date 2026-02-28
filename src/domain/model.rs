use crate::domain::error::DomainError;

use super::{AppData, ItemData, SelectionState};

/// アプリケーションのドメインロジックと状態を一元管理するモデル
pub struct TrackerModel {
    pub data: AppData,
    pub selection: SelectionState,
}

impl TrackerModel {
    pub fn new(data: AppData) -> Self {
        Self {
            data,
            selection: SelectionState::default(),
        }
    }

    // --- 参照系ヘルパー ---

    pub fn get_item(&self, cat: &str, item: &str) -> Result<&ItemData, DomainError> {
        self.data.get_item(cat, item)
    }

    // --- 操作系ロジック ---

    pub fn add_category(&mut self, name: String) -> Result<(), DomainError> {
        self.data.add_category(name)
    }

    pub fn rename_category(&mut self, old_name: &str, new_name: String) -> Result<(), DomainError> {
        let normalized_name = new_name.trim().to_string();
        self.data
            .rename_category(old_name, normalized_name.clone())?;

        // 選択状態の自動追従
        if self.selection.category.as_deref() == Some(old_name) {
            self.selection.category = Some(normalized_name);
        }

        Ok(())
    }

    pub fn remove_category(&mut self, name: &str) -> Result<(), DomainError> {
        self.data.remove_category(name)?;

        // 選択中のカテゴリが削除されたら選択解除
        if self.selection.category.as_deref() == Some(name) {
            self.selection.clear();
        }

        Ok(())
    }

    pub fn add_item(
        &mut self,
        cat_name: &str,
        item_name: String,
        decay: f64,
    ) -> Result<(), DomainError> {
        self.data.add_item(cat_name, item_name, decay)
    }

    pub fn remove_item(&mut self, cat_name: &str, item_name: &str) -> Result<(), DomainError> {
        self.data.remove_item(cat_name, item_name)?;

        if self.selection.category.as_deref() == Some(cat_name)
            && self.selection.item.as_deref() == Some(item_name)
        {
            self.selection.clear();
        }

        Ok(())
    }

    pub fn update_item(
        &mut self,
        old_loc: (&str, &str),
        new_loc: (&str, &str),
        decay: f64,
    ) -> Result<(), DomainError> {
        let (old_cat, old_item) = old_loc;
        let (new_cat, new_item) = new_loc;

        let mut temp_data = self.data.clone();

        temp_data.move_item(old_cat, new_cat, old_item)?;
        temp_data.rename_item(new_cat, old_item, new_item.to_string())?;
        temp_data.update_decay(new_cat, new_item, decay)?;

        // エラーが発生しなければ、上書き
        self.data = temp_data;

        // 選択更新
        if self.selection.category.as_deref() == Some(old_cat)
            && self.selection.item.as_deref() == Some(old_item)
        {
            self.select_item(new_cat.to_string(), new_item.to_string());
        }

        Ok(())
    }

    pub fn add_score(
        &mut self,
        cat_name: &str,
        item_name: &str,
        score: i64,
    ) -> Result<(), DomainError> {
        self.data.add_score(cat_name, item_name, score)
    }

    pub fn remove_score(
        &mut self,
        cat_name: &str,
        item_name: &str,
        index: usize,
    ) -> Result<(), DomainError> {
        self.data.remove_score(cat_name, item_name, index)?;
        self.selection.history_index = None;
        Ok(())
    }

    // 選択操作
    pub fn select_item(&mut self, cat: String, item: String) {
        self.selection.category = Some(cat);
        self.selection.item = Some(item);
        self.selection.history_index = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_model() -> TrackerModel {
        let mut model = TrackerModel::new(AppData::default());
        model.add_category("A".to_string()).unwrap();
        model.add_category("B".to_string()).unwrap();
        model.add_item("A", "item1".to_string(), 0.9).unwrap();
        model
    }

    #[test]
    fn rename_category_updates_selection() {
        // 選択中カテゴリをリネームした際に選択状態も新しいカテゴリ名へ追従することを確認する。
        let mut model = TrackerModel::new(AppData::default());
        model.add_category("Old".to_string()).unwrap();

        // 選択する
        model.select_item("Old".to_string(), "Item".to_string());

        // リネーム実行
        model.rename_category("Old", "New".to_string()).unwrap();

        // データが更新されているか
        assert!(model.data.categories.contains_key("New"));
        assert!(!model.data.categories.contains_key("Old"));

        // ★選択状態も自動で更新されているか確認★
        assert_eq!(model.selection.category.as_deref(), Some("New"));
    }

    #[test]
    fn remove_category_clears_selection_when_target_selected() {
        // 選択中のカテゴリを削除したときに選択状態が初期化されることを確認する。
        let mut model = seed_model();
        model.select_item("A".to_string(), "item1".to_string());

        model.remove_category("A").unwrap();

        assert_eq!(model.selection, SelectionState::default());
    }

    #[test]
    fn remove_item_clears_selection_when_target_selected() {
        // 選択中の項目を削除したときに選択状態が初期化されることを確認する。
        let mut model = seed_model();
        model.select_item("A".to_string(), "item1".to_string());

        model.remove_item("A", "item1").unwrap();

        assert_eq!(model.selection, SelectionState::default());
    }

    #[test]
    fn update_item_moves_renames_and_updates_selection() {
        // 項目更新で移動・名称変更・減衰率更新が行われ、選択状態も更新されることを確認する。
        let mut model = seed_model();
        model.select_item("A".to_string(), "item1".to_string());

        model
            .update_item(("A", "item1"), ("B", "item2"), 0.6)
            .unwrap();

        assert!(model.get_item("A", "item1").is_err());
        let moved = model.get_item("B", "item2").unwrap();
        assert_eq!(moved.decay_rate, 0.6);
        assert_eq!(model.selection.category.as_deref(), Some("B"));
        assert_eq!(model.selection.item.as_deref(), Some("item2"));
    }

    #[test]
    fn update_item_is_atomic_when_destination_is_missing() {
        // 項目更新が失敗した場合に元データが破壊されず一貫性が保たれることを確認する。
        let mut model = seed_model();

        let err = model
            .update_item(("A", "item1"), ("MissingCat", "item2"), 0.5)
            .unwrap_err();
        assert!(matches!(err, DomainError::NotFound(_)));

        assert!(model.get_item("A", "item1").is_ok());
        assert!(model.get_item("MissingCat", "item2").is_err());
    }

    #[test]
    fn remove_score_resets_history_selection() {
        // スコア削除後に履歴選択インデックスが解除されることを確認する。
        let mut model = seed_model();
        model.add_score("A", "item1", 100).unwrap();
        model.selection.history_index = Some(0);

        model.remove_score("A", "item1", 0).unwrap();

        assert_eq!(model.selection.history_index, None);
    }
}
