use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{CategoryData, DomainError, ItemData};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppData {
    pub categories: HashMap<String, CategoryData>,
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

    // ヘルパー関数

    /// カテゴリを検索、参照を返す
    pub fn get_category(&self, cat_name: &str) -> Result<&CategoryData, DomainError> {
        self.categories
            .get(cat_name)
            .ok_or_else(|| Self::category_not_found(cat_name))
    }

    /// カテゴリを検索、可変参照を返す
    fn get_category_mut(&mut self, cat_name: &str) -> Result<&mut CategoryData, DomainError> {
        self.categories
            .get_mut(cat_name)
            .ok_or_else(|| Self::category_not_found(cat_name))
    }

    /// 項目を検索、参照を返す
    pub fn get_item(&self, cat_name: &str, item_name: &str) -> Result<&ItemData, DomainError> {
        self.get_category(cat_name)?
            .items
            .get(item_name)
            .ok_or_else(|| Self::item_not_found(item_name))
    }

    /// 項目を検索、可変参照を返す
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

    // =======================================================================================

    /// 新しいカテゴリを追加
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
            created_at: Utc::now(),
        };

        self.categories.insert(name, cat);
        Ok(())
    }

    /// カテゴリを削除
    pub fn remove_category(&mut self, name: &str) -> Result<CategoryData, DomainError> {
        self.categories.remove(name).ok_or_else(|| {
            DomainError::NotFound(format!("削除対象のカテゴリ「{}」が見つかりません。", name))
        })
    }

    /// カテゴリ名変更
    pub fn rename_category(&mut self, old_name: &str, new_name: String) -> Result<(), DomainError> {
        let new_name = new_name.trim().to_string();
        if old_name == new_name {
            return Ok(()); // 更新なし
        }

        if new_name.is_empty() {
            return Err(DomainError::Validation(
                "新しいカテゴリ名を入力してください。".to_string(),
            ));
        }
        self.ensure_category_name_available(&new_name)?;

        let cat = self
            .categories
            .remove(old_name)
            .ok_or_else(|| Self::category_not_found(old_name))?;

        self.categories.insert(new_name, cat);
        Ok(())
    }

    // =======================

    /// 項目の追加
    pub fn add_item(
        &mut self,
        cat: &str,
        name: String,
        decay_rate: f64,
    ) -> Result<(), DomainError> {
        self.get_category_mut(cat)?.add_item(name, decay_rate)
    }

    /// 項目の削除
    pub fn remove_item(&mut self, cat: &str, item: &str) -> Result<(), DomainError> {
        let _ = self.get_category_mut(cat)?.remove_item(item);
        Ok(())
    }

    /// 項目名の変更
    pub fn rename_item(
        &mut self,
        cat: &str,
        old_name: &str,
        new_name: String,
    ) -> Result<(), DomainError> {
        self.get_category_mut(cat)?.rename_item(old_name, new_name)
    }

    /// 減衰率を変更
    pub fn update_decay(&mut self, cat: &str, item: &str, decay: f64) -> Result<(), DomainError> {
        self.get_item_mut(cat, item)?.update_decay_rate(decay)
    }

    /// 項目のカテゴリを変更
    pub fn move_item(
        &mut self,
        old_cat: &str,
        new_cat: &str,
        item: &str,
    ) -> Result<(), DomainError> {
        if old_cat == new_cat {
            return Ok(());
        }

        let item_data = self.get_category_mut(old_cat)?.remove_item(item)?;

        let target_cat = self.get_category_mut(new_cat)?;
        if target_cat.item_exists(item) {
            return Err(DomainError::AlreadyExists(format!(
                "移動先に同名の項目が存在します: {}",
                item
            )));
        }

        target_cat.items.insert(item.to_string(), item_data);
        Ok(())
    }

    // =======================

    /// スコアを追加
    pub fn add_score(&mut self, cat: &str, item: &str, score: i64) -> Result<(), DomainError> {
        self.get_item_mut(cat, item)?.add_score(score)
    }

    /// スコアを削除
    pub fn remove_score(&mut self, cat: &str, item: &str, index: usize) -> Result<(), DomainError> {
        self.get_item_mut(cat, item)?.remove_score(index)
    }
}
