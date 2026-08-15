use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::{CategoryData, DomainError, ItemData};

pub type TagId = u64;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TagData {
    pub id: TagId,
    pub name: String,
    pub color: [u8; 3],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppData {
    pub categories: HashMap<String, CategoryData>,
    #[serde(default)]
    pub tags: HashMap<TagId, TagData>,
    #[serde(default = "default_next_tag_id")]
    next_tag_id: TagId,
}

fn default_next_tag_id() -> TagId {
    1
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            categories: HashMap::new(),
            tags: HashMap::new(),
            next_tag_id: default_next_tag_id(),
        }
    }
}

impl AppData {
    fn category_not_found(cat_name: &str) -> DomainError {
        DomainError::NotFound(format!("カテゴリ「{}」が見つかりません。", cat_name))
    }

    fn item_not_found(item_name: &str) -> DomainError {
        DomainError::NotFound(format!("項目「{}」が見つかりません。", item_name))
    }

    fn ensure_category_name_available(&self, category_name: &str) -> Result<(), DomainError> {
        if self.categories.contains_key(category_name) {
            return Err(DomainError::AlreadyExists(format!(
                "カテゴリ「{}」は既に使用されています。",
                category_name
            )));
        }
        Ok(())
    }

    fn ensure_tag_name_available(
        &self,
        tag_name: &str,
        except_id: Option<TagId>,
    ) -> Result<(), DomainError> {
        if self
            .tags
            .values()
            .any(|tag| Some(tag.id) != except_id && tag.name == tag_name)
        {
            return Err(DomainError::AlreadyExists(format!(
                "タグ「{}」は既に存在します。",
                tag_name
            )));
        }
        Ok(())
    }

    pub fn get_category(&self, cat_name: &str) -> Result<&CategoryData, DomainError> {
        self.categories
            .get(cat_name)
            .ok_or_else(|| Self::category_not_found(cat_name))
    }

    fn get_category_mut(&mut self, cat_name: &str) -> Result<&mut CategoryData, DomainError> {
        self.categories
            .get_mut(cat_name)
            .ok_or_else(|| Self::category_not_found(cat_name))
    }

    pub fn get_item(&self, cat_name: &str, item_name: &str) -> Result<&ItemData, DomainError> {
        self.get_category(cat_name)?
            .items
            .get(item_name)
            .ok_or_else(|| Self::item_not_found(item_name))
    }

    /// 読み込み後に旧データのカテゴリ日時とタグ参照を補正する。
    pub fn normalize(&mut self) {
        let valid_tag_ids: HashSet<_> = self.tags.keys().copied().collect();
        for category in self.categories.values_mut() {
            category.normalize_timestamps();
            for item in category.items.values_mut() {
                item.tag_ids.retain(|id| valid_tag_ids.contains(id));
                deduplicate(&mut item.tag_ids);
            }
        }

        let max_known_id = self.tags.keys().copied().max().unwrap_or_default();
        self.next_tag_id = self.next_tag_id.max(max_known_id.saturating_add(1)).max(1);
    }

    pub fn ordered_category_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self.categories.keys().map(String::as_str).collect();
        names.sort_by(|a, b| {
            self.categories[*b]
                .updated_at
                .cmp(&self.categories[*a].updated_at)
                .then_with(|| compare_names(a, b))
        });
        names
    }

    pub fn ordered_item_names(&self, cat_name: &str) -> Result<Vec<&str>, DomainError> {
        let category = self.get_category(cat_name)?;
        let mut names: Vec<_> = category.items.keys().map(String::as_str).collect();
        names.sort_by(|a, b| {
            category.items[*b]
                .updated_at
                .cmp(&category.items[*a].updated_at)
                .then_with(|| compare_names(a, b))
        });
        Ok(names)
    }

    pub fn add_category(&mut self, name: String) -> Result<(), DomainError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(DomainError::Validation(
                "カテゴリ名を入力してください。".to_string(),
            ));
        }
        self.ensure_category_name_available(&name)?;
        self.categories.insert(name, CategoryData::new());
        Ok(())
    }

    pub fn remove_category(&mut self, name: &str) -> Result<CategoryData, DomainError> {
        self.categories.remove(name).ok_or_else(|| {
            DomainError::NotFound(format!("削除対象のカテゴリ「{}」が見つかりません。", name))
        })
    }

    pub fn rename_category(&mut self, old_name: &str, new_name: String) -> Result<(), DomainError> {
        let new_name = new_name.trim().to_string();
        if old_name == new_name {
            return Ok(());
        }
        if new_name.is_empty() {
            return Err(DomainError::Validation(
                "新しいカテゴリ名を入力してください。".to_string(),
            ));
        }
        self.ensure_category_name_available(&new_name)?;

        let mut category = self
            .categories
            .remove(old_name)
            .ok_or_else(|| Self::category_not_found(old_name))?;
        category.touch();
        self.categories.insert(new_name, category);
        Ok(())
    }

    pub fn add_item(
        &mut self,
        cat: &str,
        name: String,
        subtitle: String,
        decay_rate: f64,
        tag_ids: Vec<TagId>,
    ) -> Result<(), DomainError> {
        self.validate_tag_ids(&tag_ids)?;
        self.get_category_mut(cat)?
            .add_item(name, subtitle, decay_rate, unique(tag_ids))
    }

    pub fn remove_item(&mut self, cat: &str, item: &str) -> Result<(), DomainError> {
        self.get_category_mut(cat)?.remove_item(item).map(|_| ())
    }

    pub fn rename_item(
        &mut self,
        cat: &str,
        old_name: &str,
        new_name: String,
    ) -> Result<(), DomainError> {
        self.get_category_mut(cat)?.rename_item(old_name, new_name)
    }

    pub fn update_item_metadata(
        &mut self,
        cat: &str,
        item: &str,
        subtitle: String,
        tag_ids: Vec<TagId>,
    ) -> Result<(), DomainError> {
        self.validate_tag_ids(&tag_ids)?;
        let tag_ids = unique(tag_ids);
        let category = self.get_category_mut(cat)?;
        let item_data = category
            .items
            .get_mut(item)
            .ok_or_else(|| Self::item_not_found(item))?;
        let changed = item_data.subtitle != subtitle || item_data.tag_ids != tag_ids;
        item_data.update_metadata(subtitle, tag_ids);
        if changed {
            category.touch();
        }
        Ok(())
    }

    pub fn update_decay(&mut self, cat: &str, item: &str, decay: f64) -> Result<(), DomainError> {
        let category = self.get_category_mut(cat)?;
        let item_data = category
            .items
            .get_mut(item)
            .ok_or_else(|| Self::item_not_found(item))?;
        let changed = item_data.decay_rate != decay;
        item_data.update_decay_rate(decay)?;
        if changed {
            category.touch();
        }
        Ok(())
    }

    pub fn move_item(
        &mut self,
        old_cat: &str,
        new_cat: &str,
        item: &str,
    ) -> Result<(), DomainError> {
        if old_cat == new_cat {
            return Ok(());
        }
        if !self.get_category(old_cat)?.item_exists(item) {
            return Err(Self::item_not_found(item));
        }
        if self.get_category(new_cat)?.item_exists(item) {
            return Err(DomainError::AlreadyExists(format!(
                "移動先に同名の項目が存在します: {}",
                item
            )));
        }

        let item_data = self.get_category_mut(old_cat)?.remove_item(item)?;
        let target = self.get_category_mut(new_cat)?;
        target.items.insert(item.to_string(), item_data);
        target.touch();
        Ok(())
    }

    pub fn add_score(&mut self, cat: &str, item: &str, score: i64) -> Result<(), DomainError> {
        let category = self.get_category_mut(cat)?;
        category
            .items
            .get_mut(item)
            .ok_or_else(|| Self::item_not_found(item))?
            .add_score(score)?;
        category.touch();
        Ok(())
    }

    pub fn remove_score(&mut self, cat: &str, item: &str, index: usize) -> Result<(), DomainError> {
        let category = self.get_category_mut(cat)?;
        category
            .items
            .get_mut(item)
            .ok_or_else(|| Self::item_not_found(item))?
            .remove_score(index)?;
        category.touch();
        Ok(())
    }

    pub fn create_tag(&mut self, name: String, color: [u8; 3]) -> Result<TagId, DomainError> {
        let name = normalize_tag_name(name)?;
        self.ensure_tag_name_available(&name, None)?;
        let id = self.next_available_tag_id();
        self.tags.insert(id, TagData { id, name, color });
        Ok(id)
    }

    pub fn update_tag(
        &mut self,
        id: TagId,
        name: String,
        color: [u8; 3],
    ) -> Result<(), DomainError> {
        let name = normalize_tag_name(name)?;
        self.ensure_tag_name_available(&name, Some(id))?;
        let tag = self
            .tags
            .get_mut(&id)
            .ok_or_else(|| DomainError::NotFound("タグが見つかりません。".to_string()))?;
        tag.name = name;
        tag.color = color;
        Ok(())
    }

    pub fn delete_tag(&mut self, id: TagId) -> Result<(), DomainError> {
        self.tags
            .remove(&id)
            .ok_or_else(|| DomainError::NotFound("タグが見つかりません。".to_string()))?;
        for category in self.categories.values_mut() {
            let mut changed = false;
            for item in category.items.values_mut() {
                let original_len = item.tag_ids.len();
                item.tag_ids.retain(|tag_id| *tag_id != id);
                changed |= item.tag_ids.len() != original_len;
            }
            if changed {
                category.touch();
            }
        }
        Ok(())
    }

    fn validate_tag_ids(&self, tag_ids: &[TagId]) -> Result<(), DomainError> {
        if tag_ids.iter().all(|id| self.tags.contains_key(id)) {
            Ok(())
        } else {
            Err(DomainError::Validation(
                "存在しないタグは設定できません。".to_string(),
            ))
        }
    }

    fn next_available_tag_id(&mut self) -> TagId {
        while self.tags.contains_key(&self.next_tag_id) {
            self.next_tag_id = self.next_tag_id.saturating_add(1).max(1);
        }
        let id = self.next_tag_id;
        self.next_tag_id = self.next_tag_id.saturating_add(1).max(1);
        id
    }
}

fn compare_names(a: &str, b: &str) -> std::cmp::Ordering {
    a.to_lowercase()
        .cmp(&b.to_lowercase())
        .then_with(|| a.cmp(b))
}

fn normalize_tag_name(name: String) -> Result<String, DomainError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        Err(DomainError::Validation(
            "タグ名を入力してください。".to_string(),
        ))
    } else {
        Ok(name)
    }
}

fn unique(mut values: Vec<TagId>) -> Vec<TagId> {
    deduplicate(&mut values);
    values
}

fn deduplicate<T: Eq + std::hash::Hash + Copy>(values: &mut Vec<T>) {
    let mut seen = HashSet::new();
    values.retain(|value| seen.insert(*value));
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    fn timestamp(value: &str) -> DateTime<Utc> {
        value.parse().unwrap()
    }

    #[test]
    fn items_and_categories_are_ordered_by_update_time_then_name() {
        let mut data = AppData::default();
        data.add_category("beta".to_string()).unwrap();
        data.add_category("Alpha".to_string()).unwrap();
        data.add_item("beta", "b".to_string(), String::new(), 0.9, vec![])
            .unwrap();
        data.add_item("beta", "A".to_string(), String::new(), 0.9, vec![])
            .unwrap();
        let same_time = timestamp("2024-01-01T00:00:00Z");
        data.categories.get_mut("beta").unwrap().updated_at = same_time;
        data.categories.get_mut("Alpha").unwrap().updated_at = same_time;
        data.categories
            .get_mut("beta")
            .unwrap()
            .items
            .get_mut("b")
            .unwrap()
            .updated_at = same_time;
        data.categories
            .get_mut("beta")
            .unwrap()
            .items
            .get_mut("A")
            .unwrap()
            .updated_at = same_time;

        assert_eq!(data.ordered_category_names(), vec!["Alpha", "beta"]);
        assert_eq!(data.ordered_item_names("beta").unwrap(), vec!["A", "b"]);
    }

    #[test]
    fn normalize_removes_invalid_tag_references_and_deduplicates() {
        let mut data = AppData::default();
        data.add_category("Cat".to_string()).unwrap();
        let tag = data.create_tag("Pinned".to_string(), [1, 2, 3]).unwrap();
        data.add_item("Cat", "Item".to_string(), String::new(), 0.9, vec![tag])
            .unwrap();
        data.categories
            .get_mut("Cat")
            .unwrap()
            .items
            .get_mut("Item")
            .unwrap()
            .tag_ids = vec![tag, 999, tag];
        data.normalize();

        assert_eq!(data.get_item("Cat", "Item").unwrap().tag_ids, vec![tag]);
    }

    #[test]
    fn tags_support_multiple_assignment_and_deletion_only_unlinks_items() {
        let mut data = AppData::default();
        data.add_category("Cat".to_string()).unwrap();
        let first = data.create_tag("First".to_string(), [255, 0, 0]).unwrap();
        let second = data.create_tag("Second".to_string(), [0, 0, 255]).unwrap();
        data.add_item("Cat", "Item".to_string(), String::new(), 0.9, vec![])
            .unwrap();
        data.update_item_metadata("Cat", "Item", "note".to_string(), vec![first, second])
            .unwrap();
        data.delete_tag(first).unwrap();

        let item = data.get_item("Cat", "Item").unwrap();
        assert_eq!(item.subtitle, "note");
        assert_eq!(item.tag_ids, vec![second]);
    }

    #[test]
    fn tag_names_are_trimmed_and_unique() {
        let mut data = AppData::default();
        data.create_tag("  Work  ".to_string(), [1, 1, 1]).unwrap();
        assert!(matches!(
            data.create_tag("Work".to_string(), [1, 1, 1]),
            Err(DomainError::AlreadyExists(_))
        ));
        assert!(matches!(
            data.create_tag("  ".to_string(), [1, 1, 1]),
            Err(DomainError::Validation(_))
        ));
    }

    #[test]
    fn normalize_backfills_legacy_category_timestamps_from_items() {
        let json = r#"{"categories":{"Legacy":{"items":{"Old":{"scores":[{"score":1,"timestamp":"2020-01-01T00:00:00Z"}],"decay_rate":0.9,"updated_at":"2021-01-01T00:00:00Z"},"New":{"scores":[{"score":2,"timestamp":"2022-01-01T00:00:00Z"}],"decay_rate":0.9,"updated_at":"2023-01-01T00:00:00Z"}}}}}"#;
        let mut data: AppData = serde_json::from_str(json).unwrap();
        data.normalize();

        let category = data.get_category("Legacy").unwrap();
        assert_eq!(category.created_at, timestamp("2020-01-01T00:00:00Z"));
        assert_eq!(category.updated_at, timestamp("2023-01-01T00:00:00Z"));
    }

    #[test]
    fn normalize_uses_current_time_for_a_legacy_empty_category() {
        let before = chrono::Utc::now();
        let mut data: AppData =
            serde_json::from_str(r#"{"categories":{"Empty":{"items":{}}}}"#).unwrap();
        data.normalize();
        let after = chrono::Utc::now();

        let category = data.get_category("Empty").unwrap();
        assert!(category.created_at >= before && category.created_at <= after);
        assert_eq!(category.updated_at, category.created_at);
    }

    #[test]
    fn serialization_drops_legacy_manual_order_fields() {
        let json = r#"{"category_order":["Legacy"],"categories":{"Legacy":{"item_order":["Item"],"items":{"Item":{"scores":[],"decay_rate":0.9,"updated_at":"2024-01-01T00:00:00Z"}},"created_at":"2024-01-01T00:00:00Z","updated_at":"2024-01-01T00:00:00Z"}}}"#;
        let mut data: AppData = serde_json::from_str(json).unwrap();
        data.normalize();
        let serialized = serde_json::to_value(data).unwrap();

        assert!(serialized.get("category_order").is_none());
        assert!(
            serialized["categories"]["Legacy"]
                .get("item_order")
                .is_none()
        );
    }

    #[test]
    fn category_timestamp_updates_only_after_successful_data_changes() {
        let mut data = AppData::default();
        data.add_category("Cat".to_string()).unwrap();
        data.add_item("Cat", "Item".to_string(), String::new(), 0.9, vec![])
            .unwrap();
        data.categories.get_mut("Cat").unwrap().updated_at = timestamp("2020-01-01T00:00:00Z");

        data.update_decay("Cat", "Item", 0.8).unwrap();
        let after_change = data.get_category("Cat").unwrap().updated_at;
        assert!(after_change > timestamp("2020-01-01T00:00:00Z"));

        data.update_decay("Cat", "Item", 0.8).unwrap();
        assert_eq!(data.get_category("Cat").unwrap().updated_at, after_change);
        assert!(data.update_decay("Cat", "Missing", 0.5).is_err());
        assert_eq!(data.get_category("Cat").unwrap().updated_at, after_change);
    }
}
