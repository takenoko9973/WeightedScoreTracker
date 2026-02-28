use crate::domain::TrackerModel;

use super::{AppError, DataStore};

pub struct TrackerService<S: DataStore> {
    model: TrackerModel,
    store: S,
}

impl<S: DataStore> TrackerService<S> {
    pub fn new(store: S) -> Result<Self, AppError> {
        let data = store.load()?.unwrap_or_default();
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
        self.model.add_category(name)?;
        self.persist()
    }

    pub fn rename_category(&mut self, old_name: &str, new_name: String) -> Result<(), AppError> {
        self.model.rename_category(old_name, new_name)?;
        self.persist()
    }

    pub fn add_item(
        &mut self,
        category: &str,
        item_name: String,
        decay_input: &str,
    ) -> Result<(), AppError> {
        let decay_rate = parse_f64(decay_input, "有効な数値を入力してください。")?;
        self.model.add_item(category, item_name, decay_rate)?;
        self.persist()
    }

    pub fn add_score_to_selection(&mut self, score_input: &str) -> Result<(), AppError> {
        let (cat, item) = self
            .selected_item_pair()
            .ok_or_else(|| AppError::Domain("項目が選択されていません。".into()))?;

        let score = parse_i64(score_input, "スコアには整数値を入力してください。")?;
        self.model.add_score(&cat, &item, score)?;
        self.persist()
    }

    pub fn update_item(
        &mut self,
        old_loc: (&str, &str),
        new_loc: (&str, &str),
        decay_input: &str,
    ) -> Result<(), AppError> {
        let decay = parse_f64(decay_input, "有効な数値を入力してください。")?;
        self.model.update_item(old_loc, new_loc, decay)?;
        self.persist()
    }

    pub fn update_decay_for_selection(&mut self, decay_input: &str) -> Result<(), AppError> {
        let (cat, item) = self
            .selected_item_pair()
            .ok_or_else(|| AppError::Domain("項目が選択されていません。".into()))?;

        let decay = parse_f64(decay_input, "有効な数値を入力してください。")?;
        self.model.update_decay(&cat, &item, decay)?;
        self.persist()
    }

    pub fn delete_category(&mut self, category_name: &str) -> Result<(), AppError> {
        self.model.remove_category(category_name)?;
        self.persist()
    }

    pub fn delete_item(&mut self, category: &str, item: &str) -> Result<(), AppError> {
        self.model.remove_item(category, item)?;
        self.persist()
    }

    pub fn delete_score_from_selection(&mut self, index: usize) -> Result<(), AppError> {
        let (cat, item) = self
            .selected_item_pair()
            .ok_or_else(|| AppError::Domain("項目が選択されていません。".into()))?;

        self.model.remove_score(&cat, &item, index)?;
        self.persist()
    }

    fn selected_item_pair(&self) -> Option<(String, String)> {
        let category = self.model.selection.category.clone()?;
        let item = self.model.selection.item.clone()?;
        Some((category, item))
    }

    fn persist(&self) -> Result<(), AppError> {
        self.store.save(&self.model.data)
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
