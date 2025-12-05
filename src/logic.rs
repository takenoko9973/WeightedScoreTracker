use crate::{models::ScoreEntry, utils};

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

    let mean = utils::weighted_mean(&score_values, &weights);
    let std = utils::weighted_std(&score_values, &weights);

    (mean, std, n, weights)
}

pub struct PlotParams {
    pub bar_base: f64,
}

/// 重みに基づいて、グラフの適切な表示範囲（底と天井）を計算する
pub fn calculate_plot_params(scores: &[ScoreEntry], weights: &[f64]) -> PlotParams {
    let weight_threshold = 0.1;

    // 重みが一定以上のスコアだけを抽出（なければ全データ）
    let relevant_scores = Some(
        scores
            .iter()
            .zip(weights.iter())
            .filter_map(|(entry, &w)| (w >= weight_threshold).then_some(entry.score))
            .collect::<Vec<_>>(),
    )
    .filter(|v| !v.is_empty())
    .unwrap_or_else(|| scores.iter().map(|s| s.score).collect());

    let min_score = *relevant_scores.iter().min().unwrap_or(&0);
    let max_score = *relevant_scores.iter().max().unwrap_or(&i64::MAX);

    // 余白計算
    let range = (max_score - min_score) as f64;
    let padding = range * 0.5;
    let bar_base = (min_score as f64 - padding).max(0.0);

    PlotParams { bar_base }
}
