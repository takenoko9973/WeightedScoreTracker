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
    use crate::application::TrackerService;
    use crate::domain::AppData;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{unique}.json"))
    }

    struct TempJsonFile {
        path: PathBuf,
    }

    impl TempJsonFile {
        fn new(name: &str) -> Self {
            Self {
                path: unique_path(name),
            }
        }
    }

    impl Drop for TempJsonFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
            let _ = fs::remove_file(self.path.with_extension("json.tmp"));
        }
    }

    #[test]
    fn json_store_roundtrip() {
        // JSON ファイルへ保存したデータを再読み込みして内容が保持されることを確認する。
        let path = unique_path("weighted-score");
        let store = JsonFileStore::new(&path);

        let mut data = AppData::default();
        data.add_category("test".to_string())
            .expect("failed to add category for test");
        let tag_id = data
            .create_tag("練習中".to_string(), [255, 200, 0])
            .expect("failed to add tag for test");
        data.add_item(
            "test",
            "item".to_string(),
            "毎週更新".to_string(),
            0.9,
            vec![tag_id],
        )
        .expect("failed to add item for test");

        store.save(&data).expect("failed to save test data");

        let loaded = store
            .load()
            .expect("failed to load test data")
            .expect("expected data");
        assert!(loaded.categories.contains_key("test"));
        assert_eq!(loaded.category_order, vec!["test"]);
        assert_eq!(loaded.tags.get(&tag_id).unwrap().color, [255, 200, 0]);
        let item = loaded.get_item("test", "item").unwrap();
        assert_eq!(item.subtitle, "毎週更新");
        assert_eq!(item.tag_ids, vec![tag_id]);
        assert_eq!(loaded.ordered_item_names("test").unwrap(), vec!["item"]);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn load_returns_none_when_file_does_not_exist() {
        // 保存ファイルが存在しない場合に load が None を返すことを確認する。
        let path = unique_path("weighted-score-missing");
        let store = JsonFileStore::new(path);
        let loaded = store.load().unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn load_returns_error_for_invalid_json() {
        // 不正な JSON 形式のファイルを読み込むと永続化エラーになることを確認する。
        let path = unique_path("weighted-score-invalid");
        fs::write(&path, "{ not-json }").unwrap();

        let store = JsonFileStore::new(&path);
        let result = store.load();
        assert!(matches!(result, Err(AppError::Persistence(_))));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn updated_item_tags_survive_service_reload() {
        // サービス経由で項目タグを入れ替えた後、JSON再読込でも補足情報とタグ参照が保持されることを確認する。
        let temp_file = TempJsonFile::new("weighted-score-tag-update");
        let path = &temp_file.path;

        let mut data = AppData::default();
        data.add_category("test".to_string())
            .expect("failed to add category for test");
        let first = data
            .create_tag("first".to_string(), [255, 0, 0])
            .expect("failed to add first tag for test");
        let second = data
            .create_tag("second".to_string(), [0, 0, 255])
            .expect("failed to add second tag for test");
        data.add_item(
            "test",
            "item".to_string(),
            "before".to_string(),
            0.9,
            vec![first],
        )
        .expect("failed to add item for test");

        JsonFileStore::new(path)
            .save(&data)
            .expect("failed to seed test data");

        let mut service =
            TrackerService::new(JsonFileStore::new(path)).expect("failed to load seeded test data");
        service
            .update_item(
                ("test", "item"),
                ("test", "item"),
                "after".to_string(),
                "0.9",
                vec![second],
            )
            .expect("failed to update item for test");
        drop(service);

        let service = TrackerService::new(JsonFileStore::new(path))
            .expect("failed to reload updated test data");
        let item = service
            .model()
            .get_item("test", "item")
            .expect("expected updated item");
        assert_eq!(item.subtitle, "after");
        assert_eq!(item.tag_ids, vec![second]);
    }
}
