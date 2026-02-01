use chrono::{DateTime, Utc};
pub fn default_created_at() -> DateTime<Utc> {
    Utc::now()
}

mod app_data;
mod category;
mod item;
mod selection;

pub use app_data::AppData;
pub use category::CategoryData;
pub use item::{ItemData, ScoreEntry};
pub use selection::SelectionState;
