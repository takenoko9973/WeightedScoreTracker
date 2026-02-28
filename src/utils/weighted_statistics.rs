pub fn weighted_mean(values: &[f64], weights: &[f64]) -> f64 {
    assert_eq!(values.len(), weights.len());

    let weight_sum: f64 = weights.iter().sum();
    values.iter().zip(weights).map(|(x, w)| x * w).sum::<f64>() / weight_sum
}

pub fn weighted_variance(values: &[f64], weights: &[f64]) -> f64 {
    assert_eq!(values.len(), weights.len());

    let mean = weighted_mean(values, weights);
    let weight_sum: f64 = weights.iter().sum();

    values
        .iter()
        .zip(weights)
        .map(|(x, w)| w * (x - mean).powi(2))
        .sum::<f64>()
        / weight_sum
}

pub fn weighted_std(values: &[f64], weights: &[f64]) -> f64 {
    weighted_variance(values, weights).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-12,
            "expected {expected}, got {actual}, diff {diff}"
        );
    }

    #[test]
    fn weighted_mean_returns_expected_value() {
        // 与えた値と重みに対して加重平均が期待通りに計算されることを確認する。
        let values = [10.0, 20.0, 30.0];
        let weights = [1.0, 2.0, 1.0];
        let mean = weighted_mean(&values, &weights);
        assert_close(mean, 20.0);
    }

    #[test]
    fn weighted_variance_and_std_return_expected_values() {
        // 加重分散と加重標準偏差が既知の期待値に一致することを確認する。
        let values = [1.0, 3.0];
        let weights = [1.0, 3.0];
        let var = weighted_variance(&values, &weights);
        let std = weighted_std(&values, &weights);
        assert_close(var, 0.75);
        assert_close(std, 0.8660254037844386);
    }

    #[test]
    #[should_panic]
    fn weighted_mean_panics_on_length_mismatch() {
        // 値配列と重み配列の長さが不一致の場合に panic することを確認する。
        let values = [1.0, 2.0];
        let weights = [1.0];
        let _ = weighted_mean(&values, &weights);
    }
}
