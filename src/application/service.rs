use crate::domain::{TagId, TrackerModel};

use super::{AppError, DataStore};

pub struct TrackerService<S: DataStore> {
    model: TrackerModel,
    store: S,
}

impl<S: DataStore> TrackerService<S> {
    pub fn new(store: S) -> Result<Self, AppError> {
        let mut data = store.load()?.unwrap_or_default();
        data.normalize();
        Ok(Self {
            model: TrackerModel::new(data),
            store,
        })
    }

    pub fn empty(store: S) -> Self {
        Self {
            model: TrackerModel::new(Default::default()),
            store,
        }
    }

    pub fn model(&self) -> &TrackerModel {
        &self.model
    }

    pub fn select_item(&mut self, category: String, item: String) {
        self.model.select_item(category, item);
    }

    pub fn add_category(&mut self, name: String) -> Result<(), AppError> {
        self.mutate(|model| Ok(model.add_category(name)?))
    }

    pub fn rename_category(&mut self, old_name: &str, new_name: String) -> Result<(), AppError> {
        self.mutate(|model| Ok(model.rename_category(old_name, new_name)?))
    }

    pub fn add_item(
        &mut self,
        category: &str,
        item_name: String,
        subtitle: String,
        decay_input: &str,
        tag_ids: Vec<TagId>,
    ) -> Result<(), AppError> {
        let decay_rate = parse_f64(decay_input, "有効な数値を入力してください。")?;
        self.mutate(|model| Ok(model.add_item(category, item_name, subtitle, decay_rate, tag_ids)?))
    }

    pub fn add_score_to_selection(&mut self, score_input: &str) -> Result<(), AppError> {
        let (cat, item) = self
            .selected_item_pair()
            .ok_or_else(|| AppError::Domain("項目が選択されていません。".into()))?;

        let score = parse_i64(score_input, "スコアには整数値を入力してください。")?;
        self.mutate(|model| Ok(model.add_score(&cat, &item, score)?))
    }

    pub fn update_item(
        &mut self,
        old_loc: (&str, &str),
        new_loc: (&str, &str),
        subtitle: String,
        decay_input: &str,
        tag_ids: Vec<TagId>,
    ) -> Result<(), AppError> {
        let decay = parse_f64(decay_input, "有効な数値を入力してください。")?;
        self.mutate(|model| Ok(model.update_item(old_loc, new_loc, subtitle, decay, tag_ids)?))
    }

    pub fn delete_category(&mut self, category_name: &str) -> Result<(), AppError> {
        self.mutate(|model| Ok(model.remove_category(category_name)?))
    }

    pub fn delete_item(&mut self, category: &str, item: &str) -> Result<(), AppError> {
        self.mutate(|model| Ok(model.remove_item(category, item)?))
    }

    pub fn delete_score_from_selection(&mut self, index: usize) -> Result<(), AppError> {
        let (cat, item) = self
            .selected_item_pair()
            .ok_or_else(|| AppError::Domain("項目が選択されていません。".into()))?;

        self.mutate(|model| Ok(model.remove_score(&cat, &item, index)?))
    }

    pub fn create_tag(&mut self, name: String, color: [u8; 3]) -> Result<TagId, AppError> {
        self.mutate(|model| Ok(model.create_tag(name, color)?))
    }

    pub fn update_tag(&mut self, id: TagId, name: String, color: [u8; 3]) -> Result<(), AppError> {
        self.mutate(|model| Ok(model.update_tag(id, name, color)?))
    }

    pub fn delete_tag(&mut self, id: TagId) -> Result<(), AppError> {
        self.mutate(|model| Ok(model.delete_tag(id)?))
    }

    fn selected_item_pair(&self) -> Option<(String, String)> {
        let category = self.model.selection.category.clone()?;
        let item = self.model.selection.item.clone()?;
        Some((category, item))
    }

    fn mutate<T>(
        &mut self,
        change: impl FnOnce(&mut TrackerModel) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let mut candidate = self.model.clone();
        let result = change(&mut candidate)?;
        self.store.save(&candidate.data)?;
        self.model = candidate;
        Ok(result)
    }
}

fn parse_f64(input: &str, message: &str) -> Result<f64, AppError> {
    input
        .parse::<f64>()
        .map_err(|_| AppError::Input(message.to_string()))
}

fn parse_i64(input: &str, message: &str) -> Result<i64, AppError> {
    input
        .parse::<i64>()
        .map_err(|_| AppError::Input(message.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AppData;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct MockStore {
        loaded: Option<AppData>,
        save_calls: Rc<RefCell<usize>>,
        fail_on_load: bool,
        fail_on_save: bool,
    }

    impl MockStore {
        fn new(loaded: Option<AppData>) -> Self {
            Self {
                loaded,
                save_calls: Rc::new(RefCell::new(0)),
                fail_on_load: false,
                fail_on_save: false,
            }
        }
    }

    impl DataStore for MockStore {
        fn load(&self) -> Result<Option<AppData>, AppError> {
            if self.fail_on_load {
                return Err(AppError::Persistence("load failed".to_string()));
            }
            Ok(self.loaded.clone())
        }

        fn save(&self, _data: &AppData) -> Result<(), AppError> {
            *self.save_calls.borrow_mut() += 1;
            if self.fail_on_save {
                return Err(AppError::Persistence("save failed".to_string()));
            }
            Ok(())
        }
    }

    fn seeded_data() -> AppData {
        let mut data = AppData::default();
        data.add_category("Cat".to_string()).unwrap();
        data.add_item("Cat", "Item".to_string(), String::new(), 0.9, vec![])
            .unwrap();
        data
    }

    #[test]
    fn new_loads_existing_data_from_store() {
        // ストアに保存済みのデータがサービス初期化時に正しく読み込まれることを確認する。
        let store = MockStore::new(Some(seeded_data()));
        let service = TrackerService::new(store).unwrap();

        assert!(service.model().data.categories.contains_key("Cat"));
        assert!(service.model().get_item("Cat", "Item").is_ok());
    }

    #[test]
    fn add_item_returns_input_error_when_decay_is_invalid_number() {
        // 減衰率の入力が数値でない場合に入力エラーが返ることを確認する。
        let store = MockStore::new(Some(seeded_data()));
        let mut service = TrackerService::new(store).unwrap();

        let err = service
            .add_item(
                "Cat",
                "New".to_string(),
                String::new(),
                "not-a-number",
                vec![],
            )
            .unwrap_err();
        assert!(matches!(err, AppError::Input(_)));
    }

    #[test]
    fn add_score_to_selection_requires_selected_item() {
        // 項目未選択の状態でスコア追加するとドメインエラーになることを確認する。
        let store = MockStore::new(Some(seeded_data()));
        let mut service = TrackerService::new(store).unwrap();

        let err = service.add_score_to_selection("10").unwrap_err();
        assert!(matches!(err, AppError::Domain(_)));
    }

    #[test]
    fn add_score_to_selection_persists_on_success() {
        // スコア追加成功時にモデル更新と永続化処理が実行されることを確認する。
        let store = MockStore::new(Some(seeded_data()));
        let save_calls = Rc::clone(&store.save_calls);
        let mut service = TrackerService::new(store).unwrap();
        service.select_item("Cat".to_string(), "Item".to_string());

        service.add_score_to_selection("10").unwrap();

        assert_eq!(
            service
                .model()
                .get_item("Cat", "Item")
                .unwrap()
                .scores
                .len(),
            1
        );
        assert_eq!(*save_calls.borrow(), 1);
    }

    #[test]
    fn persistence_error_is_propagated() {
        // 永続化処理で発生したエラーがサービス層から呼び出し元へ伝播することを確認する。
        let mut store = MockStore::new(None);
        store.fail_on_save = true;
        let mut service = TrackerService::new(store).unwrap();

        let err = service.add_category("Cat".to_string()).unwrap_err();
        assert!(matches!(err, AppError::Persistence(_)));
        assert!(!service.model().data.categories.contains_key("Cat"));
    }

    #[test]
    fn failed_save_after_adding_score_preserves_timestamps_and_model_state() {
        let mut store = MockStore::new(Some(seeded_data()));
        store.fail_on_save = true;
        let save_calls = Rc::clone(&store.save_calls);
        let mut service = TrackerService::new(store).unwrap();
        service.select_item("Cat".to_string(), "Item".to_string());

        let previous_data = serde_json::to_value(&service.model().data).unwrap();
        let previous_selection = service.model().selection.clone();
        let previous_item_updated_at = service.model().get_item("Cat", "Item").unwrap().updated_at;
        let previous_category_updated_at = service.model().data.categories["Cat"].updated_at;

        let err = service.add_score_to_selection("10").unwrap_err();

        assert!(matches!(err, AppError::Persistence(_)));
        assert_eq!(
            serde_json::to_value(&service.model().data).unwrap(),
            previous_data
        );
        assert_eq!(service.model().selection, previous_selection);
        assert_eq!(
            service.model().get_item("Cat", "Item").unwrap().updated_at,
            previous_item_updated_at
        );
        assert_eq!(
            service.model().data.categories["Cat"].updated_at,
            previous_category_updated_at
        );
        assert_eq!(*save_calls.borrow(), 1);
    }

    #[test]
    fn failed_save_after_deleting_selected_item_preserves_selection_and_data() {
        let mut store = MockStore::new(Some(seeded_data()));
        store.fail_on_save = true;
        let mut service = TrackerService::new(store).unwrap();
        service.select_item("Cat".to_string(), "Item".to_string());
        service.model.selection.history_index = Some(0);

        let previous_data = serde_json::to_value(&service.model().data).unwrap();
        let previous_selection = service.model().selection.clone();

        let err = service.delete_item("Cat", "Item").unwrap_err();

        assert!(matches!(err, AppError::Persistence(_)));
        assert_eq!(
            serde_json::to_value(&service.model().data).unwrap(),
            previous_data
        );
        assert_eq!(service.model().selection, previous_selection);
        assert!(service.model().get_item("Cat", "Item").is_ok());
    }
}
