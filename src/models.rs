use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScoreEntry {
    pub score: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryData {
    pub scores: Vec<ScoreEntry>,
    pub decay_rate: f64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppData {
    pub categories: HashMap<String, CategoryData>,
}

impl AppData {
    /// 新しいカテゴリを追加
    pub fn add_category(&mut self, name: String, decay_rate: f64) {
        self.categories.insert(
            name,
            CategoryData {
                scores: Vec::new(),
                decay_rate,
            },
        );
    }

    /// カテゴリを削除
    pub fn remove_category(&mut self, name: &str) {
        self.categories.remove(name);
    }

    /// 指定カテゴリにスコアを追加
    pub fn add_score(&mut self, category_name: &str, score: i32) {
        if let Some(cat) = self.categories.get_mut(category_name) {
            cat.scores.push(ScoreEntry {
                score,
                timestamp: Utc::now(),
            });
        }
    }

    /// 指定カテゴリの指定インデックスのスコアを削除
    pub fn remove_score(&mut self, category_name: &str, index: usize) {
        if let Some(cat) = self.categories.get_mut(category_name)
            && index < cat.scores.len()
        {
            cat.scores.remove(index);
        }
    }

    /// 減衰率を更新
    pub fn update_decay_rate(&mut self, category_name: &str, new_rate: f64) {
        if let Some(cat) = self.categories.get_mut(category_name) {
            cat.decay_rate = new_rate;
        }
    }
}
