use crate::action::Action;
use crate::domain::ScoreEntry;
use crate::utils::comma_display::CommaDisplay;
use eframe::egui;

pub struct HistoryList<'a> {
    score_entries: &'a [ScoreEntry],
}

impl<'a> HistoryList<'a> {
    pub fn new(score_entries: &'a [ScoreEntry]) -> Self {
        Self { score_entries }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        selected_index: &mut Option<usize>,
        scroll_req_index: &mut Option<usize>,
    ) -> Option<Action> {
        let mut action = None;

        ui.vertical(|ui| {
            ui.label("履歴");
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("history")
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    if self.score_entries.is_empty() {
                        ui.label("履歴はありません");
                        return;
                    }

                    let total = self.score_entries.len();
                    action = self
                        .score_entries
                        .iter()
                        .rev()
                        .enumerate()
                        .filter_map(|(idx, entry)| {
                            let score_index = total - 1 - idx;

                            let is_selected = Some(score_index) == *selected_index;
                            let should_scroll = Some(score_index) == *scroll_req_index;

                            // 行オブジェクトを作って描画
                            let action = HistoryRow::new(score_index, entry, is_selected).show(
                                ui,
                                selected_index,
                                should_scroll,
                            );

                            // スクロール状態解除
                            if should_scroll {
                                *scroll_req_index = None;
                            }

                            action
                        })
                        .last();
                });
        });

        action
    }
}

struct HistoryRow<'a> {
    index: usize,
    entry: &'a ScoreEntry,
    is_selected: bool,
}

impl<'a> HistoryRow<'a> {
    pub fn new(index: usize, entry: &'a ScoreEntry, is_selected: bool) -> Self {
        Self {
            index,
            entry,
            is_selected,
        }
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        selected_index: &mut Option<usize>,
        should_scroll: bool,
    ) -> Option<Action> {
        let mut action = None;

        ui.horizontal(|ui| {
            // 削除ボタン
            let del_button = ui.button("🗑").on_hover_text("削除");
            if del_button.clicked() {
                action = Some(Action::ShowDeleteScoreConfirm(self.index));
            }

            // 日付フォーマット
            let local_time = self.entry.timestamp.with_timezone(&chrono::Local);
            let date_str = local_time.format("%Y-%m-%d %H:%M").to_string();
            let label_text = format!(
                "[{}] {}回目: {}",
                date_str,
                self.index + 1,
                self.entry.score.to_comma()
            );

            // ラベル
            let response_label = ui.selectable_label(self.is_selected, label_text);
            if response_label.clicked() {
                *selected_index = Some(self.index);
            }

            // 自動スクロール
            if should_scroll {
                response_label.scroll_to_me(Some(egui::Align::Center));
            }
        });

        action
    }
}
