use crate::models::AppData;
use crate::persistence::{load_data, save_data};
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
    pub pending_delete_category: Option<String>, // カテゴリ削除
    pub pending_delete_index: Option<usize>,     // スコア削除確認
    pub error_message: Option<String>,           // エラー用
    pub selected_history_index: Option<usize>,   // バークリック挙動
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

    fn save_to_file(&self) {
        save_data(&self.data);
    }
}

impl eframe::App for ScoreTracker {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut save_needed = false;

        // 各パネルを描画し、保存が必要ならフラグを立てる
        side_panel::draw(ctx, &mut self.data, &mut self.state);
        save_needed = save_needed || central_panel::draw(ctx, &mut self.data, &mut self.state);
        save_needed = save_needed || modals::draw(ctx, &mut self.data, &mut self.state);

        if save_needed {
            self.save_to_file();
        }
    }
}
