use chrono::{DateTime, Utc};
pub fn default_created_at() -> DateTime<Utc> {
    Utc::now()
}

mod app_data;
mod category;
mod error;
mod item;
mod model;
mod selection;

pub use app_data::{AppData, MoveDirection, TagData, TagId};
pub use category::CategoryData;
pub use error::DomainError;
pub use item::{ItemData, ScoreEntry};
pub use model::TrackerModel;
pub use selection::SelectionState;
