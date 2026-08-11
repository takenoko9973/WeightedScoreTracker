use crate::constants::PLOT_WEIGHT_THRESHOLD;
use crate::domain::ScoreEntry;
use crate::utils::weighted_statistics::{weighted_mean, weighted_std};
use std::iter::zip;

fn generate_weight(decay_rate: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| decay_rate.powi(i as i32))
        .rev() // 最初のデータほど重みは少ない
        .collect::<Vec<_>>()
}

pub fn calculate_stats(scores: &[ScoreEntry], decay_rate: f64) -> (f64, f64, usize, Vec<f64>) {
    if scores.is_empty() {
        return (0.0, 0.0, 0, Vec::new());
    }

    let n = scores.len();
    let weights = generate_weight(decay_rate, n);
    let score_values = scores.iter().map(|s| s.score as f64).collect::<Vec<_>>();

    let mean = weighted_mean(&score_values, &weights);
    let std = weighted_std(&score_values, &weights);

    (mean, std, n, weights)
}

/// 各時点までの履歴から得られる加重統計値。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatsPoint {
    pub mean: f64,
    pub std: f64,
}

/// 履歴の各時点までに対する加重平均と加重標準偏差を生成する。
pub fn calculate_stats_series(scores: &[ScoreEntry], decay_rate: f64) -> Vec<StatsPoint> {
    scores
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let (mean, std, _, _) = calculate_stats(&scores[..=index], decay_rate);
            StatsPoint { mean, std }
        })
        .collect()
}

pub struct PlotParams {
    pub max_y: f64,
    pub min_y: f64,
}

fn filtered_scores(scores: &[ScoreEntry], weights: &[f64], threshold: f64) -> Option<Vec<i64>> {
    let v: Vec<_> = zip(scores.iter(), weights.iter())
        .filter_map(|(entry, &w)| (w >= threshold).then_some(entry.score))
        .collect();

    (!v.is_empty()).then_some(v) // フィルターによって要素がない場合は、すべて返す
}

/// 重みに基づいて、グラフの適切な表示範囲を計算する
pub fn calculate_plot_params(scores: &[ScoreEntry], weights: &[f64]) -> PlotParams {
    // 重みが一定以上のスコアだけを抽出（なければ全データ）
    let relevant_scores = filtered_scores(scores, weights, PLOT_WEIGHT_THRESHOLD)
        .unwrap_or_else(|| scores.iter().map(|s| s.score).collect());

    let min_score = *relevant_scores.iter().min().unwrap_or(&0);
    let max_score = *relevant_scores.iter().max().unwrap_or(&0);

    // 余白計算
    let range = (max_score - min_score) as f64;
    let padding = range * 0.1;
    let max_y = max_score as f64 + padding;
    let min_y = (min_score as f64 - padding).max(0.0);

    PlotParams { max_y, min_y }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn score_entries(values: &[i64]) -> Vec<ScoreEntry> {
        values
            .iter()
            .map(|&score| ScoreEntry {
                score,
                timestamp: Utc::now(),
            })
            .collect()
    }

    fn assert_close(actual: f64, expected: f64) {
        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-9,
            "expected {expected}, got {actual}, diff {diff}"
        );
    }

    #[test]
    fn calculate_stats_returns_default_for_empty_scores() {
        // スコア履歴が空のときに統計値と重みが既定値で返ることを確認する。
        let (mean, std, n, weights) = calculate_stats(&[], 0.9);
        assert_eq!(mean, 0.0);
        assert_eq!(std, 0.0);
        assert_eq!(n, 0);
        assert!(weights.is_empty());
    }

    #[test]
    fn calculate_stats_generates_decay_weights_and_statistics() {
        // 減衰率に基づく重み配列と加重平均・標準偏差が期待値になることを確認する。
        let scores = score_entries(&[10, 20, 30]);
        let (mean, std, n, weights) = calculate_stats(&scores, 0.5);

        assert_eq!(n, 3);
        assert_eq!(weights, vec![0.25, 0.5, 1.0]);
        assert_close(mean, 24.285714285714285);
        assert_close(std, 7.284313590846315);
    }

    #[test]
    fn calculate_stats_series_returns_empty_for_empty_scores() {
        let series = calculate_stats_series(&[], 0.9);

        assert!(series.is_empty());
    }

    #[test]
    fn calculate_stats_series_returns_score_and_zero_std_for_one_score() {
        let scores = score_entries(&[42]);

        let series = calculate_stats_series(&scores, 0.9);

        assert_eq!(
            series,
            vec![StatsPoint {
                mean: 42.0,
                std: 0.0
            }]
        );
    }

    #[test]
    fn calculate_stats_series_is_independent_for_each_prefix() {
        let prefix_scores = score_entries(&[10, 20]);
        let scores_with_suffix = score_entries(&[10, 20, 100]);

        let prefix_series = calculate_stats_series(&prefix_scores, 0.5);
        let series_with_suffix = calculate_stats_series(&scores_with_suffix, 0.5);

        for (prefix_point, full_point) in prefix_series.iter().zip(&series_with_suffix) {
            assert_close(prefix_point.mean, full_point.mean);
            assert_close(prefix_point.std, full_point.std);
            assert!(prefix_point.mean.is_finite());
            assert!(prefix_point.std.is_finite());
        }
    }

    #[test]
    fn calculate_stats_series_final_point_matches_calculate_stats() {
        let scores = score_entries(&[10, 20, 30]);
        let decay_rate = 0.5;

        let series = calculate_stats_series(&scores, decay_rate);
        let (mean, std, _, _) = calculate_stats(&scores, decay_rate);
        let final_point = series.last().expect("複数スコアの系列には最終点がある");

        assert_close(final_point.mean, mean);
        assert_close(final_point.std, std);
    }

    #[test]
    fn calculate_stats_series_reflects_decay_rate() {
        let scores = score_entries(&[10, 30]);

        let decayed = calculate_stats_series(&scores, 0.5);
        let equally_weighted = calculate_stats_series(&scores, 1.0);

        assert_close(decayed[1].mean, 70.0 / 3.0);
        assert_close(equally_weighted[1].mean, 20.0);
        assert_ne!(decayed[1].mean, equally_weighted[1].mean);
    }

    #[test]
    fn calculate_plot_params_uses_only_scores_above_weight_threshold() {
        // 重み閾値以上のスコアのみを使って表示範囲が計算されることを確認する。
        let scores = score_entries(&[10, 100, 20]);
        let weights = vec![0.05, 0.2, 1.0];

        let params = calculate_plot_params(&scores, &weights);
        assert_close(params.min_y, 12.0);
        assert_close(params.max_y, 108.0);
    }

    #[test]
    fn calculate_plot_params_falls_back_to_all_scores_when_filtered_empty() {
        // 閾値フィルタ後に対象が空の場合は全スコアで表示範囲を再計算することを確認する。
        let scores = score_entries(&[10, 30]);
        let weights = vec![0.01, 0.02];

        let params = calculate_plot_params(&scores, &weights);
        assert_close(params.min_y, 8.0);
        assert_close(params.max_y, 32.0);
    }

    #[test]
    fn calculate_plot_params_clamps_min_y_to_zero() {
        // 表示範囲の下限が負値にならないよう 0.0 にクランプされることを確認する。
        let scores = score_entries(&[0, 5]);
        let weights = vec![1.0, 1.0];

        let params = calculate_plot_params(&scores, &weights);
        assert_close(params.min_y, 0.0);
        assert_close(params.max_y, 5.5);
    }
}
