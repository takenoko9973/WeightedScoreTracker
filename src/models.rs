use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const MIN_DECAY_RATE: f64 = 0.01;
pub const MAX_DECAY_RATE: f64 = 1.00;
pub const DEFAULT_DECAY_RATE: f64 = 0.90;

// バリデーションヘルパー関数（再利用のため）
pub fn validate_decay_rate_range(rate: f64) -> Result<(), String> {
    if (MIN_DECAY_RATE..=MAX_DECAY_RATE).contains(&rate) {
        Ok(())
    } else {
        Err(format!(
            "減衰率は {:.2} ～ {:.2} の範囲で指定してください。",
            MIN_DECAY_RATE, MAX_DECAY_RATE
        ))
    }
}

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
    pub fn try_add_category(&mut self, name: String) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("カテゴリ名を入力してください。".to_string());
        }
        if self.categories.contains_key(&name) {
            return Err("そのカテゴリ名は既に使用されています。".to_string());
        }

        self.categories.insert(
            name,
            CategoryData {
                items: HashMap::new(),
                created_at: Utc::now(),
            },
        );
        Ok(())
    }

    /// カテゴリ名変更
    pub fn try_rename_category(&mut self, old_name: &str, new_name: String) -> Result<(), String> {
        if new_name.trim().is_empty() {
            return Err("新しいカテゴリ名を入力してください。".to_string());
        }
        if self.categories.contains_key(&new_name) {
            return Err("その名前は既に使用されています。".to_string());
        }

        let Some(data) = self.categories.remove(old_name) else {
            return Err("変更元のカテゴリが見つかりません。".to_string());
        };
        self.categories.insert(new_name, data);
        Ok(())
    }

    /// カテゴリを削除
    pub fn try_remove_category(&mut self, name: &str) -> Result<(), String> {
        if self.categories.remove(name).is_some() {
            Ok(())
        } else {
            Err("削除対象のカテゴリが見つかりません。".to_string())
        }
    }

    /// 指定したカテゴリの中に項目を追加
    pub fn try_add_item(
        &mut self,
        category_name: &str,
        item_name: String,
        decay_rate: f64,
    ) -> Result<(), String> {
        validate_decay_rate_range(decay_rate)?;

        if item_name.trim().is_empty() {
            return Err("項目名を入力してください。".to_string());
        }

        let Some(cat) = self.categories.get_mut(category_name) else {
            return Err("対象のカテゴリが見つかりません。".to_string());
        };
        if cat.items.contains_key(&item_name) {
            return Err("その項目名は既に存在します。".to_string());
        }

        cat.items.insert(
            item_name,
            ItemData {
                scores: Vec::new(),
                decay_rate,
                created_at: Utc::now(),
            },
        );
        Ok(())
    }

    /// 項目の情報を一括更新（移動、リネーム、減衰率変更）
    pub fn try_update_item(
        &mut self,
        old_cat_name: &str,
        old_item_name: &str,
        new_cat_name: &str,
        new_item_name: String,
        new_decay: f64,
    ) -> Result<(), String> {
        // バリデーション
        validate_decay_rate_range(new_decay)?;
        if new_item_name.trim().is_empty() {
            return Err("項目名を入力してください。".to_string());
        }

        // 1. 移動・変更がない場合は何もしない
        if old_cat_name == new_cat_name
            && old_item_name == new_item_name
            && self
                .categories
                .get(old_cat_name)
                .and_then(|c| c.items.get(old_item_name))
                .map(|i| i.decay_rate)
                == Some(new_decay)
        {
            return Ok(());
        }

        // 2. 移動先のカテゴリが存在するか確認
        if !self.categories.contains_key(new_cat_name) {
            return Err("移動先のカテゴリが存在しません。".to_string());
        }

        // 3. 名前変更または移動が発生する場合、移動先での名前重複チェック
        let name_changed_or_moved = old_cat_name != new_cat_name || old_item_name != new_item_name;
        if name_changed_or_moved
            && let Some(dest_cat) = self.categories.get(new_cat_name)
            && dest_cat.items.contains_key(&new_item_name)
        {
            return Err(format!(
                "カテゴリ「{}」内に項目「{}」は既に存在します。",
                new_cat_name, new_item_name
            ));
        }

        // 4. データを取り出す（元データ削除）
        let mut item_data = if let Some(src_cat) = self.categories.get_mut(old_cat_name) {
            src_cat
                .items
                .remove(old_item_name)
                .ok_or("編集対象の項目が見つかりません。".to_string())?
        } else {
            return Err("元のカテゴリが見つかりません。".to_string());
        };

        // 5. データを更新して新しい場所に挿入
        item_data.decay_rate = new_decay;

        if let Some(dest_cat) = self.categories.get_mut(new_cat_name) {
            dest_cat.items.insert(new_item_name, item_data);
            Ok(())
        } else {
            // 万が一削除後に挿入先が消えていた場合の救済（通常ありえない）
            Err("移動先のカテゴリへの保存に失敗しました。".to_string())
        }
    }

    /// 指定したカテゴリの中の項目を削除
    pub fn try_remove_item(&mut self, category_name: &str, item_name: &str) -> Result<(), String> {
        let Some(cat) = self.categories.get_mut(category_name) else {
            return Err("親カテゴリが見つかりません。".to_string());
        };

        if cat.items.remove(item_name).is_some() {
            return Ok(());
        }
        return Err("削除対象の項目が見つかりません。".to_string());
    }

    /// スコアを追加
    pub fn try_add_score(
        &mut self,
        category_name: &str,
        item_name: &str,
        score: i32,
    ) -> Result<(), String> {
        if score < 0 {
            return Err("スコアにマイナスの値は入力できません。".to_string());
        }

        self.categories
            .get_mut(category_name)
            .and_then(|cat| cat.items.get_mut(item_name))
            .map(|item| {
                item.scores.push(ScoreEntry {
                    score,
                    timestamp: Utc::now(),
                });
            })
            .ok_or_else(|| "カテゴリまたは項目が見つかりません。".to_string())
    }

    /// スコアを削除
    pub fn try_remove_score(
        &mut self,
        category_name: &str,
        item_name: &str,
        index: usize,
    ) -> Result<(), String> {
        let Some(cat) = self.categories.get_mut(category_name) else {
            return Err("カテゴリが見つかりません。".to_string());
        };
        let Some(item) = cat.items.get_mut(item_name) else {
            return Err("項目が見つかりません。".to_string());
        };
        if index >= item.scores.len() {
            return Err("指定されたスコアのインデックスが範囲外です。".to_string());
        }

        item.scores.remove(index);
        Ok(())
    }

    /// 減衰率を更新
    pub fn try_update_decay_rate(
        &mut self,
        category_name: &str,
        item_name: &str,
        new_rate: f64,
    ) -> Result<(), String> {
        validate_decay_rate_range(new_rate)?;

        self.categories
            .get_mut(category_name)
            .and_then(|cat| cat.items.get_mut(item_name))
            .map(|item| {
                item.decay_rate = new_rate;
            })
            .ok_or_else(|| "減衰率の変更に失敗しました。".to_string())
    }
}
