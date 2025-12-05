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
