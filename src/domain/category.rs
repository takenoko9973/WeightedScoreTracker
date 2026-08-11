use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{DomainError, ItemData, default_created_at};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryData {
    pub items: HashMap<String, ItemData>,

    /// 手動表示順。未登録の項目は AppData::normalize で補われる。
    #[serde(default)]
    pub item_order: Vec<String>,

    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
}

impl CategoryData {
    fn ensure_item_name_available(&self, item_name: &str) -> Result<(), DomainError> {
        if self.items.contains_key(item_name) {
            return Err(DomainError::AlreadyExists(format!(
                "項目「{}」は既に存在します。",
                item_name
            )));
        }
        Ok(())
    }

    pub fn item_exists(&self, item: &str) -> bool {
        self.items.contains_key(item)
    }

    pub fn add_item(
        &mut self,
        name: String,
        subtitle: String,
        decay_rate: f64,
        tag_ids: Vec<super::TagId>,
    ) -> Result<(), DomainError> {
        let name = name.trim().to_string();
        self.ensure_item_name_available(&name)?;

        let now = Utc::now();
        let item = ItemData {
            scores: Vec::new(),
            decay_rate,
            subtitle,
            tag_ids,
            updated_at: now,
        };

        self.items.insert(name.clone(), item);
        self.item_order.push(name);
        Ok(())
    }

    pub fn rename_item(&mut self, old_name: &str, new_name: String) -> Result<(), DomainError> {
        let new_name = new_name.trim().to_string();
        if old_name == new_name {
            return Ok(()); // 更新なし
        }

        if new_name.is_empty() {
            return Err(DomainError::Validation(
                "項目名を入力してください。".to_string(),
            ));
        }
        self.ensure_item_name_available(&new_name)?;

        let item = self
            .items
            .remove(old_name)
            .ok_or_else(|| DomainError::NotFound("変更元の項目が見つかりません。".to_string()))?;

        self.items.insert(new_name.clone(), item);
        if let Some(entry) = self
            .item_order
            .iter_mut()
            .find(|entry| entry.as_str() == old_name)
        {
            *entry = new_name;
        }
        Ok(())
    }

    pub fn remove_item(&mut self, item_name: &str) -> Result<ItemData, DomainError> {
        let item = self
            .items
            .remove(item_name)
            .ok_or_else(|| DomainError::NotFound("削除対象の項目が見つかりません。".to_string()))?;
        self.item_order.retain(|name| name != item_name);
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_category() -> CategoryData {
        CategoryData {
            items: HashMap::new(),
            item_order: Vec::new(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn add_item_trims_name_and_rejects_duplicate() {
        // 項目名の前後空白が除去され、同名項目の追加が拒否されることを確認する。
        let mut category = empty_category();
        category
            .add_item("  A  ".to_string(), String::new(), 0.9, Vec::new())
            .unwrap();
        assert!(category.item_exists("A"));

        let err = category
            .add_item("A".to_string(), String::new(), 0.9, Vec::new())
            .unwrap_err();
        assert!(matches!(err, DomainError::AlreadyExists(_)));
    }

    #[test]
    fn rename_item_validates_and_moves_entry() {
        // 項目名変更でエントリが移動し、空名と不存在項目がエラーになることを確認する。
        let mut category = empty_category();
        category
            .add_item("Old".to_string(), String::new(), 0.9, Vec::new())
            .unwrap();
        let updated_at = category.items["Old"].updated_at;

        category.rename_item("Old", "  New  ".to_string()).unwrap();
        assert!(category.item_exists("New"));
        assert!(!category.item_exists("Old"));
        assert_eq!(category.items["New"].updated_at, updated_at);

        let err = category.rename_item("New", "   ".to_string()).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));

        let err = category
            .rename_item("Missing", "X".to_string())
            .unwrap_err();
        assert!(matches!(err, DomainError::NotFound(_)));
    }

    #[test]
    fn remove_item_returns_error_when_missing() {
        // 存在しない項目を削除しようとした場合に NotFound エラーになることを確認する。
        let mut category = empty_category();
        let err = category.remove_item("Nope").unwrap_err();
        assert!(matches!(err, DomainError::NotFound(_)));
    }
}
