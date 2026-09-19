use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use super::{DomainError, ItemData};

#[derive(Serialize, Clone, Debug)]
pub struct CategoryData {
    pub items: HashMap<String, ItemData>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip)]
    missing_created_at: bool,
    #[serde(skip)]
    missing_updated_at: bool,
}

#[derive(Deserialize)]
struct StoredCategoryData {
    items: HashMap<String, ItemData>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
}

impl<'de> Deserialize<'de> for CategoryData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let stored = StoredCategoryData::deserialize(deserializer)?;
        let now = Utc::now();
        Ok(Self {
            items: stored.items,
            created_at: stored.created_at.unwrap_or(now),
            updated_at: stored.updated_at.unwrap_or(now),
            missing_created_at: stored.created_at.is_none(),
            missing_updated_at: stored.updated_at.is_none(),
        })
    }
}

impl CategoryData {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            items: HashMap::new(),
            created_at: now,
            updated_at: now,
            missing_created_at: false,
            missing_updated_at: false,
        }
    }

    pub(crate) fn normalize_timestamps(&mut self) {
        if self.missing_created_at {
            self.created_at = self
                .items
                .values()
                .flat_map(|item| item.scores.iter().map(|score| score.timestamp))
                .min()
                .unwrap_or_else(Utc::now);
            self.missing_created_at = false;
        }
        if self.missing_updated_at {
            self.updated_at = self
                .items
                .values()
                .map(|item| item.updated_at)
                .max()
                .unwrap_or(self.created_at);
            self.missing_updated_at = false;
        }
    }

    pub(crate) fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

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
        self.items.insert(
            name,
            ItemData {
                scores: Vec::new(),
                decay_rate,
                subtitle,
                tag_ids,
                updated_at: now,
            },
        );
        self.touch();
        Ok(())
    }

    pub fn rename_item(&mut self, old_name: &str, new_name: String) -> Result<(), DomainError> {
        let new_name = new_name.trim().to_string();
        if old_name == new_name {
            return Ok(());
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
        self.items.insert(new_name, item);
        self.touch();
        Ok(())
    }

    pub fn remove_item(&mut self, item_name: &str) -> Result<ItemData, DomainError> {
        let item = self
            .items
            .remove(item_name)
            .ok_or_else(|| DomainError::NotFound("削除対象の項目が見つかりません。".to_string()))?;
        self.touch();
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_item_trims_name_and_rejects_duplicate() {
        let mut category = CategoryData::new();
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
    fn rename_item_keeps_item_timestamp_but_updates_category_timestamp() {
        let mut category = CategoryData::new();
        category
            .add_item("Old".to_string(), String::new(), 0.9, Vec::new())
            .unwrap();
        let item_updated_at = category.items["Old"].updated_at;
        category.updated_at = Utc::now() - chrono::Duration::seconds(1);
        let category_updated_at = category.updated_at;

        category.rename_item("Old", "New".to_string()).unwrap();

        assert_eq!(category.items["New"].updated_at, item_updated_at);
        assert!(category.updated_at > category_updated_at);
    }

    #[test]
    fn remove_item_returns_error_when_missing() {
        let mut category = CategoryData::new();
        let err = category.remove_item("Nope").unwrap_err();
        assert!(matches!(err, DomainError::NotFound(_)));
    }
}
