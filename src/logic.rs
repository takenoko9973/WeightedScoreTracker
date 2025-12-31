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
