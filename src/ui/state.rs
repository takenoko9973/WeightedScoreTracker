use std::collections::HashSet;

use crate::domain::TagId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemSort {
    #[default]
    Recent,
    Name,
    Manual,
}

#[derive(Default)]
pub struct UiState {
    /// エラーメッセージ
    pub error_message: Option<String>,
    pub search_query: String,
    pub selected_tag_ids: HashSet<TagId>,
    pub item_sort: ItemSort,
}
