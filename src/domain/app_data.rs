use chrono::Utc;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveDirection {
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppData {
    pub categories: HashMap<String, CategoryData>,
    #[serde(default)]
    pub tags: HashMap<TagId, TagData>,
    #[serde(default)]
    pub category_order: Vec<String>,
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
            category_order: Vec::new(),
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

    fn get_item_mut(
        &mut self,
        cat_name: &str,
        item_name: &str,
    ) -> Result<&mut ItemData, DomainError> {
        self.get_category_mut(cat_name)?
            .items
            .get_mut(item_name)
            .ok_or_else(|| Self::item_not_found(item_name))
    }

    /// 読み込み後または編集後に、手動順とタグ参照をデータ本体に合わせる。
    /// 旧JSONで順序が無い場合は旧UIと同じ日時降順を採用する。
    pub fn normalize(&mut self) {
        self.category_order = normalized_names(
            &self.category_order,
            self.categories
                .iter()
                .map(|(name, category)| (name, category.created_at)),
        );

        let valid_tag_ids: HashSet<_> = self.tags.keys().copied().collect();
        for category in self.categories.values_mut() {
            category.item_order = normalized_names(
                &category.item_order,
                category
                    .items
                    .iter()
                    .map(|(name, item)| (name, item.updated_at)),
            );
            for item in category.items.values_mut() {
                item.tag_ids.retain(|id| valid_tag_ids.contains(id));
                deduplicate(&mut item.tag_ids);
            }
        }

        let max_known_id = self.tags.keys().copied().max().unwrap_or_default();
        self.next_tag_id = self.next_tag_id.max(max_known_id.saturating_add(1)).max(1);
    }

    pub fn ordered_category_names(&self) -> Vec<&str> {
        self.category_order.iter().map(String::as_str).collect()
    }

    pub fn ordered_item_names(&self, cat_name: &str) -> Result<Vec<&str>, DomainError> {
        Ok(self
            .get_category(cat_name)?
            .item_order
            .iter()
            .map(String::as_str)
            .collect())
    }

    pub fn add_category(&mut self, name: String) -> Result<(), DomainError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(DomainError::Validation(
                "カテゴリ名を入力してください。".to_string(),
            ));
        }
        self.ensure_category_name_available(&name)?;

        let cat = CategoryData {
            items: HashMap::new(),
            item_order: Vec::new(),
            created_at: Utc::now(),
        };
        self.categories.insert(name.clone(), cat);
        self.category_order.insert(0, name);
        Ok(())
    }

    pub fn remove_category(&mut self, name: &str) -> Result<CategoryData, DomainError> {
        let category = self.categories.remove(name).ok_or_else(|| {
            DomainError::NotFound(format!("削除対象のカテゴリ「{}」が見つかりません。", name))
        })?;
        self.category_order
            .retain(|category_name| category_name != name);
        Ok(category)
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

        let category = self
            .categories
            .remove(old_name)
            .ok_or_else(|| Self::category_not_found(old_name))?;
        self.categories.insert(new_name.clone(), category);
        if let Some(entry) = self
            .category_order
            .iter_mut()
            .find(|entry| entry.as_str() == old_name)
        {
            *entry = new_name;
        }
        Ok(())
    }

    pub fn move_category(
        &mut self,
        name: &str,
        direction: MoveDirection,
    ) -> Result<(), DomainError> {
        move_in_order(&mut self.category_order, name, direction, "カテゴリ")
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
        self.get_item_mut(cat, item)?
            .update_metadata(subtitle, unique(tag_ids));
        Ok(())
    }

    pub fn update_decay(&mut self, cat: &str, item: &str, decay: f64) -> Result<(), DomainError> {
        self.get_item_mut(cat, item)?.update_decay_rate(decay)
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
        target.item_order.push(item.to_string());
        Ok(())
    }

    pub fn move_item_in_order(
        &mut self,
        cat: &str,
        item: &str,
        direction: MoveDirection,
    ) -> Result<(), DomainError> {
        move_in_order(
            &mut self.get_category_mut(cat)?.item_order,
            item,
            direction,
            "項目",
        )
    }

    pub fn add_score(&mut self, cat: &str, item: &str, score: i64) -> Result<(), DomainError> {
        self.get_item_mut(cat, item)?.add_score(score)
    }

    pub fn remove_score(&mut self, cat: &str, item: &str, index: usize) -> Result<(), DomainError> {
        self.get_item_mut(cat, item)?.remove_score(index)
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
            for item in category.items.values_mut() {
                item.tag_ids.retain(|tag_id| *tag_id != id);
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

fn normalized_names<'a, T: Ord + Copy>(
    saved: &[String],
    values: impl Iterator<Item = (&'a String, T)>,
) -> Vec<String> {
    let value_map: HashMap<_, _> = values.map(|(name, value)| (name.as_str(), value)).collect();
    let mut seen = HashSet::new();
    let mut order: Vec<_> = saved
        .iter()
        .filter(|name| value_map.contains_key(name.as_str()) && seen.insert(name.as_str()))
        .cloned()
        .collect();
    let mut missing: Vec<_> = value_map
        .into_iter()
        .filter(|(name, _)| !seen.contains(name))
        .collect();
    missing.sort_by(|(name_a, time_a), (name_b, time_b)| {
        time_b.cmp(time_a).then_with(|| name_a.cmp(name_b))
    });
    order.extend(missing.into_iter().map(|(name, _)| name.to_string()));
    order
}

fn unique(mut values: Vec<TagId>) -> Vec<TagId> {
    deduplicate(&mut values);
    values
}

fn deduplicate<T: Eq + std::hash::Hash + Copy>(values: &mut Vec<T>) {
    let mut seen = HashSet::new();
    values.retain(|value| seen.insert(*value));
}

fn move_in_order(
    order: &mut [String],
    name: &str,
    direction: MoveDirection,
    subject: &str,
) -> Result<(), DomainError> {
    let index = order
        .iter()
        .position(|entry| entry == name)
        .ok_or_else(|| {
            DomainError::NotFound(format!("{}「{}」が見つかりません。", subject, name))
        })?;
    let target = match direction {
        MoveDirection::Up if index > 0 => index - 1,
        MoveDirection::Down if index + 1 < order.len() => index + 1,
        _ => return Ok(()),
    };
    order.swap(index, target);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_data() -> AppData {
        let mut data = AppData::default();
        data.add_category("CatA".to_string()).unwrap();
        data.add_category("CatB".to_string()).unwrap();
        data.add_item("CatA", "Item1".to_string(), String::new(), 0.9, vec![])
            .unwrap();
        data
    }

    #[test]
    fn deserialize_old_data_and_normalize_uses_legacy_sort_order() {
        let json = r#"{"categories":{"Older":{"items":{"A":{"scores":[],"decay_rate":0.9,"updated_at":"2020-01-01T00:00:00Z"}},"created_at":"2020-01-01T00:00:00Z"},"Newer":{"items":{},"created_at":"2021-01-01T00:00:00Z"}}}"#;
        let mut data: AppData = serde_json::from_str(json).unwrap();
        data.normalize();

        assert_eq!(data.ordered_category_names(), vec!["Newer", "Older"]);
        assert_eq!(data.get_item("Older", "A").unwrap().subtitle, "");
        assert!(data.get_item("Older", "A").unwrap().tag_ids.is_empty());
        assert_eq!(data.ordered_item_names("Older").unwrap(), vec!["A"]);
    }

    #[test]
    fn normalize_removes_invalid_tag_references_and_deduplicates() {
        let mut data = seed_data();
        let tag = data.create_tag("Pinned".to_string(), [1, 2, 3]).unwrap();
        data.get_item_mut("CatA", "Item1").unwrap().tag_ids = vec![tag, 999, tag];
        data.normalize();
        assert_eq!(data.get_item("CatA", "Item1").unwrap().tag_ids, vec![tag]);
    }

    #[test]
    fn tags_support_multiple_assignment_and_deletion_only_unlinks_items() {
        let mut data = seed_data();
        let first = data.create_tag("First".to_string(), [255, 0, 0]).unwrap();
        let second = data.create_tag("Second".to_string(), [0, 0, 255]).unwrap();
        data.update_item_metadata("CatA", "Item1", "note".to_string(), vec![first, second])
            .unwrap();
        data.delete_tag(first).unwrap();

        let item = data.get_item("CatA", "Item1").unwrap();
        assert_eq!(item.subtitle, "note");
        assert_eq!(item.tag_ids, vec![second]);
        assert!(data.get_item("CatA", "Item1").is_ok());
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
    fn manual_orders_follow_rename_move_and_delete() {
        let mut data = seed_data();
        assert_eq!(data.ordered_category_names(), vec!["CatB", "CatA"]);
        data.move_category("CatA", MoveDirection::Up).unwrap();
        assert_eq!(data.ordered_category_names(), vec!["CatA", "CatB"]);

        data.add_item("CatA", "Item2".to_string(), String::new(), 0.9, vec![])
            .unwrap();
        data.move_item_in_order("CatA", "Item2", MoveDirection::Up)
            .unwrap();
        assert_eq!(
            data.ordered_item_names("CatA").unwrap(),
            vec!["Item2", "Item1"]
        );
        data.rename_item("CatA", "Item2", "Renamed".to_string())
            .unwrap();
        assert_eq!(
            data.ordered_item_names("CatA").unwrap(),
            vec!["Renamed", "Item1"]
        );
        data.move_item("CatA", "CatB", "Renamed").unwrap();
        assert_eq!(data.ordered_item_names("CatA").unwrap(), vec!["Item1"]);
        assert_eq!(data.ordered_item_names("CatB").unwrap(), vec!["Renamed"]);
        data.remove_item("CatB", "Renamed").unwrap();
        data.rename_category("CatB", "Other".to_string()).unwrap();
        assert_eq!(data.ordered_category_names(), vec!["CatA", "Other"]);
        data.remove_category("Other").unwrap();
        data.normalize();

        assert_eq!(data.ordered_category_names(), vec!["CatA"]);
        assert_eq!(data.ordered_item_names("CatA").unwrap(), vec!["Item1"]);
    }
}
