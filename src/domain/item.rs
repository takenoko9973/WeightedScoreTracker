use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DomainError, default_created_at};
use crate::constants::{MAX_DECAY_RATE, MIN_DECAY_RATE};

// バリデーションヘルパー関数
pub fn validate_decay_rate_range(rate: f64) -> Result<(), DomainError> {
    if (MIN_DECAY_RATE..=MAX_DECAY_RATE).contains(&rate) {
        Ok(())
    } else {
        Err(DomainError::Validation(format!(
            "減衰率は {:.2} ～ {:.2} の範囲で指定してください。",
            MIN_DECAY_RATE, MAX_DECAY_RATE
        )))
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
    pub fn add_score(&mut self, score: i64) -> Result<(), DomainError> {
        if score < 0 {
            return Err(DomainError::Validation(
                "スコアにマイナスの値は入力できません。".to_string(),
            ));
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

    pub fn remove_score(&mut self, index: usize) -> Result<(), DomainError> {
        if index >= self.scores.len() {
            return Err(DomainError::Validation(
                "指定されたスコアのインデックスが範囲外です。".to_string(),
            ));
        }

        self.scores.remove(index);
        self.updated_at = Utc::now();

        Ok(())
    }

    pub fn update_decay_rate(&mut self, new_rate: f64) -> Result<(), DomainError> {
        validate_decay_rate_range(new_rate)?;

        self.decay_rate = new_rate;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{MAX_DECAY_RATE, MIN_DECAY_RATE};

    fn sample_item() -> ItemData {
        ItemData {
            scores: Vec::new(),
            decay_rate: 0.9,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn validate_decay_rate_accepts_boundaries_and_rejects_out_of_range() {
        // 減衰率の境界値は許可され、範囲外の値はバリデーションエラーになることを確認する。
        assert!(validate_decay_rate_range(MIN_DECAY_RATE).is_ok());
        assert!(validate_decay_rate_range(MAX_DECAY_RATE).is_ok());
        assert!(matches!(
            validate_decay_rate_range(MIN_DECAY_RATE - 0.001),
            Err(DomainError::Validation(_))
        ));
        assert!(matches!(
            validate_decay_rate_range(MAX_DECAY_RATE + 0.001),
            Err(DomainError::Validation(_))
        ));
    }

    #[test]
    fn add_score_rejects_negative_value() {
        // 負のスコアを追加しようとするとエラーになり履歴が増えないことを確認する。
        let mut item = sample_item();
        let err = item.add_score(-1).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
        assert!(item.scores.is_empty());
    }

    #[test]
    fn remove_score_rejects_out_of_bounds_index() {
        // 範囲外インデックスの削除がエラーとなり既存スコアが保持されることを確認する。
        let mut item = sample_item();
        item.add_score(42).unwrap();

        let err = item.remove_score(1).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
        assert_eq!(item.scores.len(), 1);
    }

    #[test]
    fn update_decay_rate_changes_value_on_success() {
        // 有効な減衰率で更新した場合に値が正しく反映されることを確認する。
        let mut item = sample_item();
        item.update_decay_rate(0.5).unwrap();
        assert_eq!(item.decay_rate, 0.5);
    }
}
