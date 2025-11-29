use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScoreEntry {
    pub score: i32,
    pub timestamp: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CategoryData {
    pub scores: Vec<ScoreEntry>,
    pub decay_rate: f64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AppData {
    pub categories: HashMap<String, CategoryData>,
}
