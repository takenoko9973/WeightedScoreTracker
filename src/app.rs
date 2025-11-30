use crate::models::AppData;
use crate::persistence::{load_data, save_data};
use crate::ui::Action;
use crate::ui::{central_panel, modals, side_panel};
use eframe::egui;

#[derive(Default)]
pub struct UiState {
    pub current_category: Option<String>,
    pub input_score: String,
    pub input_category: String,
    pub input_decay: String,
    pub show_add_category_window: bool,
    pub show_edit_decay_window: bool,
    pub selected_history_index: Option<usize>, // バークリック挙動
    pub pending_delete_category: Option<String>, // カテゴリ削除
    pub pending_delete_index: Option<usize>,   // スコア削除確認
    pub error_message: Option<String>,         // エラー用
}

// アプリケーション状態保存
pub struct ScoreTracker {
    data: AppData,
    state: UiState,
}

impl ScoreTracker {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let data = load_data().unwrap_or_default();
        Self {
            data,
            state: UiState::default(),
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            // モーダル表示系
            Action::ShowAddCategoryModal => self.open_add_category_modal(),
            Action::ShowEditDecayModal => self.open_edit_decay_modal(),
            Action::ShowDeleteCategoryConfirm(name) => self.confirm_delete_category(name),
            Action::ShowDeleteScoreConfirm(idx) => self.confirm_delete_score(idx),

            // データ操作系
            Action::SelectCategory(name) => self.select_category(name),
            Action::AddCategory(name, decay) => self.add_category(name, decay),
            Action::AddScore(text) => self.add_score(text),
            Action::UpdateDecayRate(rate) => self.update_decay_rate(rate),
            Action::ExecuteDeleteCategory(name) => self.execute_delete_category(name),
            Action::ExecuteDeleteScore(idx) => self.execute_delete_score(idx),
        };
    }

    // ======================================
    // 共通処理
    // ======================================
    fn save_to_file(&mut self) {
        if let Err(e) = save_data(&self.data) {
            self.state.error_message = Some(format!("保存に失敗しました: {}", e));
        }
    }

    fn validate_decay_rate(&self, rate_str: &str) -> Result<f64, String> {
        match rate_str.parse::<f64>() {
            Ok(decay_rate) if 0.0 < decay_rate && decay_rate <= 1.0 => Ok(decay_rate),
            Ok(_) => Err("減衰率は 0 より大きく、1 以下の数値を入力してください。".to_string()),
            Err(_) => Err("有効な数値を入力してください。".to_string()),
        }
    }

    // ======================================
    // モーダル表示
    // ======================================

    /// カテゴリ登録
    fn open_add_category_modal(&mut self) {
        self.state.input_category.clear();
        self.state.input_decay = "0.95".to_string();
        self.state.show_add_category_window = true;
    }

    /// 減衰率変更
    fn open_edit_decay_modal(&mut self) {
        if let Some(cat) = &self.state.current_category
            && let Some(d) = self.data.categories.get(cat)
        {
            self.state.input_decay = d.decay_rate.to_string();
            self.state.show_edit_decay_window = true;
        }
    }

    /// カテゴリ削除
    fn confirm_delete_category(&mut self, name: String) {
        self.state.pending_delete_category = Some(name);
    }

    /// スコア削除
    fn confirm_delete_score(&mut self, idx: usize) {
        self.state.pending_delete_index = Some(idx);
    }

    // ======================================
    // データ操作
    // ======================================

    /// カテゴリ選択
    fn select_category(&mut self, name: String) {
        self.state.current_category = Some(name);

        // カテゴリが変わったら入力欄と選択状態をリセットする
        self.state.input_score.clear();
        self.state.selected_history_index = None;
    }

    /// カテゴリ登録
    fn add_category(&mut self, name: String, decay_str: String) {
        if name.is_empty() {
            self.state.error_message = Some("項目名を入力してください。".to_string());
            return;
        }

        match self.validate_decay_rate(&decay_str) {
            Ok(rate) => {
                self.data.add_category(name, rate);
                self.save_to_file();
                self.state.show_add_category_window = false;
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// スコア追加
    fn add_score(&mut self, text: String) {
        let Some(cat_name) = &self.state.current_category else {
            return;
        };

        match text.parse::<i32>() {
            Ok(score) if score >= 0 => {
                self.data.add_score(cat_name, score);
                self.state.input_score.clear();
                self.save_to_file();
            }
            Ok(_) => {
                self.state.error_message =
                    Some("スコアにマイナスの値は入力できません。".to_string());
            }
            Err(_) => {
                self.state.error_message = Some("有効な整数値を入力してください。".to_string());
            }
        }
    }

    /// 減衰率変更
    fn update_decay_rate(&mut self, rate_str: String) {
        let Some(cat) = &self.state.current_category else {
            return;
        };

        match self.validate_decay_rate(&rate_str) {
            Ok(rate) => {
                self.data.update_decay_rate(cat, rate);
                self.save_to_file();
                self.state.show_edit_decay_window = false; // ウィンドウ非表示処理
            }
            Err(msg) => self.state.error_message = Some(msg),
        }
    }

    /// カテゴリ削除
    fn execute_delete_category(&mut self, name: String) {
        self.data.remove_category(&name);
        if self.state.current_category.as_ref() == Some(&name) {
            self.state.current_category = None;
            self.state.input_score.clear();
        }
        self.save_to_file();
    }

    /// スコア削除
    fn execute_delete_score(&mut self, idx: usize) {
        let Some(cat) = &self.state.current_category else {
            return;
        };

        self.data.remove_score(cat, idx);
        self.state.selected_history_index = None;
        self.save_to_file();
    }
}

impl eframe::App for ScoreTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let action = side_panel::draw(ctx, &self.data, &mut self.state)
            .or_else(|| central_panel::draw(ctx, &self.data, &mut self.state))
            .or_else(|| modals::draw(ctx, &self.data, &mut self.state));

        if let Some(act) = action {
            self.handle_action(act);
        }
    }
}
