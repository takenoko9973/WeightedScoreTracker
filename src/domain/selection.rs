#[derive(Default, Debug, Clone, PartialEq)]
pub struct SelectionState {
    pub category: Option<String>,
    pub item: Option<String>,
    pub history_index: Option<usize>,
}

impl SelectionState {
    pub fn clear(&mut self) {
        self.category = None;
        self.item = None;
        self.history_index = None;
    }
}
