use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::app::{ItemData, default_created_at};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryData {
    pub items: HashMap<String, ItemData>,

    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
}

impl CategoryData {
    pub fn item_exists(&self, item: &str) -> bool {
        self.items.contains_key(item)
    }

    pub fn add_item(&mut self, name: String, decay_rate: f64) -> Result<(), String> {
        let name = name.trim().to_string();
        if self.items.contains_key(&name) {
            return Err(format!("項目「{}」は既に存在します。", name));
        }

        let now = Utc::now();
        let item = ItemData {
            scores: Vec::new(),
            decay_rate,
            updated_at: now,
        };

        self.items.insert(name, item);
        Ok(())
    }

    pub fn rename_item(&mut self, old_name: &str, new_name: String) -> Result<(), String> {
        let new_name = new_name.trim().to_string();
        if old_name == new_name {
            return Ok(()); // 更新なし
        }

        if new_name.is_empty() {
            return Err("項目名を入力してください。".to_string());
        }
        if self.item_exists(&new_name) {
            return Err(format!("項目「{}」は既に存在します。", new_name));
        }

        let item = self
            .items
            .remove(old_name)
            .ok_or_else(|| "変更元の項目が見つかりません。".to_string())?;

        self.items.insert(new_name, item);
        Ok(())
    }

    pub fn remove_item(&mut self, item_name: &str) -> Result<ItemData, String> {
        self.items
            .remove(item_name)
            .ok_or_else(|| "削除対象の項目が見つかりません。".to_string())
    }
}
