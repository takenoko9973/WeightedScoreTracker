use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScoreEntry {
    pub score: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemData {
    pub scores: Vec<ScoreEntry>,
    pub decay_rate: f64,

    #[serde(default = "default_created_at")] // 未設定の場合、現在時刻で埋める
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryData {
    pub items: HashMap<String, ItemData>,

    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
}

fn default_created_at() -> DateTime<Utc> {
    Utc::now()
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppData {
    pub categories: HashMap<String, CategoryData>,
}

impl AppData {
    /// 新しいカテゴリを追加
    pub fn add_category(&mut self, name: String) {
        self.categories.entry(name).or_insert_with(|| CategoryData {
            items: HashMap::new(),
            created_at: Utc::now(),
        });
    }

    /// カテゴリを削除
    pub fn remove_category(&mut self, name: &str) {
        self.categories.remove(name);
    }

    /// 指定したカテゴリの中に項目を追加
    pub fn add_item(&mut self, category_name: &str, item_name: String, decay_rate: f64) {
        if let Some(cat) = self.categories.get_mut(category_name) {
            cat.items.insert(
                item_name,
                ItemData {
                    scores: Vec::new(),
                    decay_rate,
                    created_at: Utc::now(),
                },
            );
        }
    }

    /// 指定したカテゴリの中の項目を削除
    pub fn remove_item(&mut self, category_name: &str, item_name: &str) {
        if let Some(cat) = self.categories.get_mut(category_name) {
            cat.items.remove(item_name);
        }
    }

    /// スコアを追加
    pub fn add_score(&mut self, category_name: &str, item_name: &str, score: i32) {
        if let Some(cat) = self.categories.get_mut(category_name)
            && let Some(item) = cat.items.get_mut(item_name)
        {
            item.scores.push(ScoreEntry {
                score,
                timestamp: Utc::now(),
            });
        }
    }

    /// スコアを削除
    pub fn remove_score(&mut self, category_name: &str, item_name: &str, index: usize) {
        if let Some(cat) = self.categories.get_mut(category_name)
            && let Some(item) = cat.items.get_mut(item_name)
            && index < item.scores.len()
        {
            item.scores.remove(index);
        }
    }

    /// 減衰率を更新
    pub fn update_decay_rate(&mut self, category_name: &str, item_name: &str, new_rate: f64) {
        if let Some(cat) = self.categories.get_mut(category_name)
            && let Some(item) = cat.items.get_mut(item_name)
        {
            item.decay_rate = new_rate;
        }
    }
}
