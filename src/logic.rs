use crate::models::ScoreEntry;

pub fn calculate_stats(scores: &[ScoreEntry], decay_rate: f64) -> (f64, usize, Vec<f64>) {
    if scores.is_empty() {
        return (0.0, 0, Vec::new());
    }

    let n = scores.len();
    let mut weights = Vec::new();
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for (i, entry) in scores.iter().enumerate() {
        let exponent = (n - 1) - i;
        let w = decay_rate.powi(exponent as i32);

        weights.push(w);
        weighted_sum += entry.score as f64 * w;
        total_weight += w;
    }

    let avg = if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        0.0
    };
    (avg, n, weights)
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
    let max_score = *relevant_scores.iter().max().unwrap_or(&i32::MAX);

    // 余白計算
    let range = (max_score - min_score) as f64;
    let padding = range * 0.5;
    let bar_base = (min_score as f64 - padding).max(0.0);

    PlotParams { bar_base }
}
