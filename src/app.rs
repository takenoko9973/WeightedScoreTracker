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

        // --- コントローラーロジック: イベント処理 ---
        if let Some(action) = central_panel::draw(ctx, &mut self.data, &mut self.state) {
            match action {
                // スコア追加リクエスト
                central_panel::Action::RequestAddScore(text) => {
                    if let Some(cat_name) = &self.state.current_category {
                        // ここで初めてバリデーションを行う
                        match text.parse::<i32>() {
                            Ok(score) if score >= 0 => {
                                // 成功: データを追加し、入力欄をクリア
                                self.data.add_score(cat_name, score);
                                self.state.input_score.clear();
                                save_needed = true;
                            }
                            Ok(_) => {
                                self.state.error_message =
                                    Some("スコアにマイナスの値は入力できません。".to_string())
                            }
                            Err(_) => {
                                self.state.error_message =
                                    Some("有効な整数値を入力してください。".to_string())
                            }
                        }
                    }
                }

                // 削除リクエスト
                central_panel::Action::RequestDeleteScore(idx) => {
                    // 確認ダイアログを出すためにStateを更新
                    self.state.pending_delete_index = Some(idx);
                }

                // 設定画面リクエスト
                central_panel::Action::OpenDecaySettings => {
                    if let Some(cat_name) = &self.state.current_category
                        && let Some(d) = self.data.categories.get(cat_name)
                    {
                        self.state.input_decay = d.decay_rate.to_string();
                        self.state.show_edit_decay_window = true;
                    }
                }
            }
        }

        save_needed = save_needed || modals::draw(ctx, &mut self.data, &mut self.state);

        if save_needed {
            self.save_to_file();
        }
    }
}
