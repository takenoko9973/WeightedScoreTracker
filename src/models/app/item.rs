use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::constants::{MAX_DECAY_RATE, MIN_DECAY_RATE};
use crate::models::app::default_created_at;

// バリデーションヘルパー関数
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
    pub score: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemData {
    pub scores: Vec<ScoreEntry>,
    pub decay_rate: f64,

    // 古いJSONファイルの "created_at" も読み込む
    #[serde(alias = "created_at", default = "default_created_at")]
    // 未設定の場合、現在時刻で埋める
    pub updated_at: DateTime<Utc>,
}

impl ItemData {
    pub fn add_score(&mut self, score: i64) -> Result<(), String> {
        if score < 0 {
            return Err("スコアにマイナスの値は入力できません。".to_string());
        }

        let now = Utc::now();
        let score_entry = ScoreEntry {
            score,
            timestamp: now,
        };

        self.scores.push(score_entry);
        self.updated_at = now;

        Ok(())
    }

    pub fn remove_score(&mut self, index: usize) -> Result<(), String> {
        if index >= self.scores.len() {
            return Err("指定されたスコアのインデックスが範囲外です。".to_string());
        }

        self.scores.remove(index);
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn update_decay_rate(&mut self, new_rate: f64) -> Result<(), String> {
        validate_decay_rate_range(new_rate)?;

        self.decay_rate = new_rate;
        Ok(())
    }
}
