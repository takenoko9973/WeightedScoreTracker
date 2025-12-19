use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::app::{CategoryData, ItemData};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AppData {
    pub categories: HashMap<String, CategoryData>,
}

impl AppData {
    // ヘルパー関数

    /// カテゴリを検索、参照を返す
    pub fn get_category(&self, cat_name: &str) -> Result<&CategoryData, String> {
        self.categories
            .get(cat_name)
            .ok_or_else(|| format!("カテゴリ「{}」が見つかりません。", cat_name))
    }

    /// カテゴリを検索、可変参照を返す
    fn get_category_mut(&mut self, cat_name: &str) -> Result<&mut CategoryData, String> {
        self.categories
            .get_mut(cat_name)
            .ok_or_else(|| format!("カテゴリ「{}」が見つかりません。", cat_name))
    }

    /// 項目を検索、参照を返す
    pub fn get_item(&self, cat_name: &str, item_name: &str) -> Result<&ItemData, String> {
        self.get_category(cat_name)?
            .items
            .get(item_name)
            .ok_or_else(|| format!("項目「{}」が見つかりません。", item_name))
    }

    /// 項目を検索、可変参照を返す
    fn get_item_mut(&mut self, cat_name: &str, item_name: &str) -> Result<&mut ItemData, String> {
        self.get_category_mut(cat_name)?
            .items
            .get_mut(item_name)
            .ok_or_else(|| format!("項目「{}」が見つかりません。", item_name))
    }

    /// 減衰率を取得
    pub fn get_item_decay(&self, cat_name: &str, item_name: &str) -> Result<f64, String> {
        Ok(self.get_item(cat_name, item_name)?.decay_rate)
    }

    // =======================================================================================

    /// カテゴリ存在確認
    pub fn category_exists(&self, cat: &str) -> bool {
        self.categories.contains_key(cat)
    }

    /// 項目存在確認
    pub fn item_exists(&self, cat: &str, item: &str) -> bool {
        self.get_category(cat)
            .map(|cat| cat.item_exists(item))
            .unwrap_or(false)
    }

    // =======================================================================================

    /// 新しいカテゴリを追加
    pub fn add_category(&mut self, name: String) -> Result<(), String> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err("カテゴリ名を入力してください。".to_string());
        }
        if self.category_exists(&name) {
            return Err(format!("カテゴリ「{}」は既に使用されています。", name));
        }

        let cat = CategoryData {
            items: HashMap::new(),
            created_at: Utc::now(),
        };

        self.categories.insert(name, cat);
        Ok(())
    }

    /// カテゴリを削除
    pub fn remove_category(&mut self, name: &str) -> Result<CategoryData, String> {
        self.categories
            .remove(name)
            .ok_or_else(|| format!("削除対象のカテゴリ「{}」が見つかりません。", name))
    }

    /// カテゴリ名変更
    pub fn rename_category(&mut self, old_name: &str, new_name: String) -> Result<(), String> {
        let new_name = new_name.trim().to_string();
        if old_name == new_name {
            return Ok(()); // 更新なし
        }

        if new_name.is_empty() {
            return Err("新しいカテゴリ名を入力してください。".to_string());
        }
        if self.category_exists(&new_name) {
            return Err(format!("カテゴリ「{}」は既に使用されています。", new_name));
        }

        let cat = self
            .categories
            .remove(old_name)
            .ok_or_else(|| format!("変更元のカテゴリ「{}」が見つかりません。", old_name))?;

        self.categories.insert(new_name, cat);
        Ok(())
    }

    // =======================

    /// 項目の追加
    pub fn add_item(&mut self, cat: &str, name: String, decay_rate: f64) -> Result<(), String> {
        self.get_category_mut(cat)?.add_item(name, decay_rate)
    }

    /// 項目の削除
    pub fn remove_item(&mut self, cat: &str, item: &str) -> Result<(), String> {
        let _ = self.get_category_mut(cat)?.remove_item(item);
        Ok(())
    }

    /// 項目名の変更
    pub fn rename_item(
        &mut self,
        cat: &str,
        old_name: &str,
        new_name: String,
    ) -> Result<(), String> {
        self.get_category_mut(cat)?.rename_item(old_name, new_name)
    }

    /// 減衰率を変更
    pub fn update_decay(&mut self, cat: &str, item: &str, decay: f64) -> Result<(), String> {
        self.get_item_mut(cat, item)?.update_decay_rate(decay)
    }

    /// 項目のカテゴリを変更
    pub fn move_item(&mut self, old_cat: &str, new_cat: &str, item: &str) -> Result<(), String> {
        if old_cat == new_cat {
            return Ok(());
        }

        if !self.category_exists(old_cat) {
            return Err(format!("移動元のカテゴリ「{}」が存在しません。", old_cat));
        }
        if !self.category_exists(new_cat) {
            return Err(format!("移動先のカテゴリ「{}」が存在しません。", new_cat));
        }
        if self.item_exists(new_cat, item) {
            return Err(format!(
                "カテゴリ「{}」内に項目「{}」は既に存在します。",
                new_cat, item
            ));
        }

        let item_data = self.get_category_mut(old_cat)?.remove_item(item)?;

        self.get_category_mut(new_cat)?
            .items
            .insert(item.to_string(), item_data);

        Ok(())
    }

    // =======================

    /// スコアを追加
    pub fn add_score(&mut self, cat: &str, item: &str, score: i64) -> Result<(), String> {
        self.get_item_mut(cat, item)?.add_score(score)
    }

    /// スコアを削除
    pub fn remove_score(&mut self, cat: &str, item: &str, index: usize) -> Result<(), String> {
        self.get_item_mut(cat, item)?.remove_score(index)
    }
}
