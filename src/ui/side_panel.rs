use crate::app::UiState;
use crate::models::AppData;
use crate::ui::Action;
use eframe::egui;

// 項目欄
pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("カテゴリ一覧");
            ui.separator();

            // 下から上に配置
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.add_space(5.0);

                let btn_size = egui::vec2(ui.available_width(), 30.0);

                // === カテゴリ削除ボタン
                let is_selected = state.current_category.is_some(); // 選択確認
                let delete_clicked = ui
                    .add_enabled_ui(is_selected, |ui| {
                        ui.add_sized(btn_size, egui::Button::new("🗑 項目削除"))
                    })
                    .inner
                    .clicked();
                if delete_clicked && let Some(current) = &state.current_category {
                    action = Some(Action::ShowDeleteCategoryConfirm(current.clone()));
                }
                ui.add_space(5.0);

                // === 追加ボタン
                let register_clicked = ui
                    .add_sized(btn_size, egui::Button::new("＋ 項目追加"))
                    .clicked();
                if register_clicked {
                    action = Some(Action::ShowAddCategoryModal);
                }

                ui.separator();

                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    // フッターを除いた残りの配置可能な高さ
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());

                            // カテゴリリスト (作成順でソート)
                            let mut categories: Vec<_> = data.categories.iter().collect();
                            categories.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

                            for (cat, _) in categories {
                                let is_selected = state.current_category.as_ref() == Some(cat);
                                if ui
                                    .add_sized(
                                        egui::vec2(ui.available_width(), 20.0),
                                        egui::Button::new(cat).selected(is_selected),
                                    )
                                    .clicked()
                                {
                                    state.current_category = Some(cat.clone());
                                    state.input_score.clear();
                                    state.selected_history_index = None; // 履歴選択状態解除
                                }
                            }
                        });
                })
            });
        });

    action
}
