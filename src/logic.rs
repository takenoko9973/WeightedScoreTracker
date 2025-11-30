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
