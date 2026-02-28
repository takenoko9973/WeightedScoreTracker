use crate::domain::AppData;

use super::AppError;

/// データ系の操作を行うためのインターフェース
pub trait DataStore {
    fn load(&self) -> Result<Option<AppData>, AppError>;
    fn save(&self, data: &AppData) -> Result<(), AppError>;
}
