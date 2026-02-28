use std::fs;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::application::{AppError, DataStore};
use crate::domain::AppData;

pub struct JsonFileStore {
    path: PathBuf,
}

impl JsonFileStore {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }

    fn temp_path(&self) -> PathBuf {
        let mut tmp = self.path.clone();
        let extension = self
            .path
            .extension()
            .map(|ext| format!("{}.tmp", ext.to_string_lossy()))
            .unwrap_or_else(|| "tmp".to_string());

        tmp.set_extension(extension);
        tmp
    }

    fn path_exists(path: &Path) -> bool {
        path.exists()
    }
}

impl DataStore for JsonFileStore {
    fn load(&self) -> Result<Option<AppData>, AppError> {
        if !Self::path_exists(&self.path) {
            return Ok(None);
        }

        let file = fs::File::open(&self.path)
            .map_err(|e| AppError::Persistence(format!("ファイルを開けません: {}", e)))?;
        let reader = BufReader::new(file);

        let data = serde_json::from_reader(reader)
            .map_err(|e| AppError::Persistence(format!("JSON読込に失敗しました: {}", e)))?;

        Ok(Some(data))
    }

    fn save(&self, data: &AppData) -> Result<(), AppError> {
        let tmp_path = self.temp_path();

        let file = fs::File::create(&tmp_path)
            .map_err(|e| AppError::Persistence(format!("一時ファイル作成に失敗しました: {}", e)))?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer_pretty(&mut writer, data)
            .map_err(|e| AppError::Persistence(format!("JSON保存に失敗しました: {}", e)))?;
        writer
            .flush()
            .map_err(|e| AppError::Persistence(format!("一時ファイル書込に失敗しました: {}", e)))?;

        fs::rename(&tmp_path, &self.path).map_err(|e| {
            AppError::Persistence(format!(
                "ファイルの置換に失敗しました ({} -> {}): {}",
                tmp_path.display(),
                self.path.display(),
                e
            ))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn json_store_roundtrip() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();

        let path = std::env::temp_dir().join(format!("weighted-score-{}.json", unique));
        let store = JsonFileStore::new(&path);

        let mut data = AppData::default();
        data.add_category("test".to_string())
            .expect("failed to add category for test");

        store.save(&data).expect("failed to save test data");

        let loaded = store
            .load()
            .expect("failed to load test data")
            .expect("expected data");
        assert!(loaded.categories.contains_key("test"));

        let _ = fs::remove_file(path);
    }
}
