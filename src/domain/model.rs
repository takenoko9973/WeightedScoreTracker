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

    pub fn update_decay(
        &mut self,
        cat_name: &str,
        item_name: &str,
        decay: f64,
    ) -> Result<(), DomainError> {
        self.data.update_decay(cat_name, item_name, decay)
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

// === テストの書きやすさの証明 ===
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_category_updates_selection() {
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
}
