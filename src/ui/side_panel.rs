use crate::app::UiState;
use crate::models::AppData;
use eframe::egui;

// 項目欄
pub fn draw(ctx: &egui::Context, data: &mut AppData, state: &mut UiState) {
    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            let footer_height = 80.0;

            ui.label("カテゴリ一覧");
            ui.separator();

            // フッターを除いた残りの配置可能な高さ
            let available_height_for_list = ui.available_height() - footer_height;
            egui::ScrollArea::vertical()
                .max_height(available_height_for_list)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // カテゴリリスト
                    let mut categories: Vec<_> = data.categories.keys().cloned().collect();
                    categories.sort();

                    for cat in &categories {
                        let is_selected = state.current_category.as_ref() == Some(cat);
                        let category_buttom = egui::Button::new(cat).selected(is_selected);
                        if ui
                            .add_sized(egui::vec2(ui.available_width(), 20.0), category_buttom)
                            .clicked()
                        {
                            state.current_category = Some(cat.clone());
                            state.input_score.clear();
                        }
                    }
                });

            // フッターを除いた残りの空白分
            let space_height = ui.available_height() - footer_height;
            if space_height > 0.0 {
                ui.allocate_space(egui::vec2(0.0, space_height));
            }

            ui.separator();

            // === 追加ボタン
            let btn_size = egui::vec2(ui.available_width(), 30.0);
            if ui
                .add_sized(btn_size, egui::Button::new("＋ 項目追加"))
                .clicked()
            {
                state.input_category.clear();
                state.input_decay = "0.95".to_string();
                state.show_add_category_window = true;
            }

            // === カテゴリ削除ボタン
            let is_selected = state.current_category.is_some(); // 選択確認
            let delete_btn_response = ui
                .add_enabled_ui(is_selected, |ui| {
                    ui.add_sized(btn_size, egui::Button::new("🗑 項目削除"))
                })
                .inner;

            if delete_btn_response.clicked()
                && let Some(current) = &state.current_category
            {
                // 確認用変数をセット
                state.pending_delete_category = Some(current.clone());
            }
        });
}
