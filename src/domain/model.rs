use crate::domain::error::DomainError;

use super::{AppData, CategoryData, ItemData, SelectionState};

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

    pub fn get_category(&self, name: &str) -> Result<&CategoryData, DomainError> {
        self.data
            .categories
            .get(name)
            .ok_or_else(|| DomainError::NotFound(format!("カテゴリ「{}」", name)))
    }

    pub fn get_item(&self, cat: &str, item: &str) -> Result<&ItemData, DomainError> {
        self.get_category(cat)?
            .items
            .get(item)
            .ok_or_else(|| DomainError::NotFound(format!("項目「{}」", item)))
    }

    // --- 操作系ロジック ---

    pub fn add_category(&mut self, name: String) -> Result<(), DomainError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(DomainError::Validation("カテゴリ名が空です".into()));
        }
        if self.data.categories.contains_key(&name) {
            return Err(DomainError::AlreadyExists(name));
        }

        self.data.add_category(name);
        Ok(())
    }

    pub fn rename_category(&mut self, old_name: &str, new_name: String) -> Result<(), DomainError> {
        let new_name = new_name.trim().to_string();
        if old_name == new_name {
            return Ok(());
        }
        if new_name.is_empty() {
            return Err(DomainError::Validation("新しい名前が空です".into()));
        }
        if self.data.categories.contains_key(&new_name) {
            return Err(DomainError::AlreadyExists(new_name));
        }

        // データ移動
        let category_data = self
            .data
            .categories
            .remove(old_name)
            .ok_or_else(|| DomainError::NotFound(old_name.to_string()))?;
        self.data.categories.insert(new_name.clone(), category_data);

        // 選択状態の自動追従
        if self.selection.category.as_deref() == Some(old_name) {
            self.selection.category = Some(new_name);
        }

        Ok(())
    }

    pub fn remove_category(&mut self, name: &str) -> Result<(), DomainError> {
        if self.data.categories.remove(name).is_none() {
            return Err(DomainError::NotFound(name.to_string()));
        }

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
        let item_name = item_name.trim().to_string();
        if item_name.is_empty() {
            return Err(DomainError::Validation("項目名が空です".into()));
        }

        let category = self
            .data
            .categories
            .get_mut(cat_name)
            .ok_or_else(|| DomainError::NotFound(cat_name.to_string()))?;

        if category.items.contains_key(&item_name) {
            return Err(DomainError::AlreadyExists(item_name));
        }

        category
            .add_item(item_name, decay)
            .map_err(DomainError::Validation)?;
        Ok(())
    }

    pub fn remove_item(&mut self, cat_name: &str, item_name: &str) -> Result<(), String> {
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
    ) -> Result<(), String> {
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

    pub fn rename_item(
        &mut self,
        cat_name: &str,
        old_item: &str,
        new_item: String,
    ) -> Result<(), DomainError> {
        let new_item = new_item.trim().to_string();
        if old_item == new_item {
            return Ok(());
        }

        let category = self
            .data
            .categories
            .get_mut(cat_name)
            .ok_or_else(|| DomainError::NotFound(cat_name.to_string()))?;

        if category.items.contains_key(&new_item) {
            return Err(DomainError::AlreadyExists(new_item));
        }

        let item_data = category
            .items
            .remove(old_item)
            .ok_or_else(|| DomainError::NotFound(old_item.to_string()))?;

        category.items.insert(new_item.clone(), item_data);

        // 選択状態の追従
        if self.selection.category.as_deref() == Some(cat_name)
            && self.selection.item.as_deref() == Some(old_item)
        {
            self.selection.item = Some(new_item);
        }

        Ok(())
    }

    pub fn update_decay(
        &mut self,
        cat_name: &str,
        item_name: &str,
        decay: f64,
    ) -> Result<(), String> {
        self.data.update_decay(cat_name, item_name, decay)
    }

    pub fn add_score(
        &mut self,
        cat_name: &str,
        item_name: &str,
        score: i64,
    ) -> Result<(), DomainError> {
        self.data.add_score(cat_name, item_name, score);
        Ok(())
    }

    pub fn remove_score(
        &mut self,
        cat_name: &str,
        item_name: &str,
        index: usize,
    ) -> Result<(), String> {
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
