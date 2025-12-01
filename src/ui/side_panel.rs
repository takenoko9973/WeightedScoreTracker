use eframe::egui;

use crate::ui::Action;
use crate::ui::components::category_list;
use crate::{models::AppData, ui::state::UiState};

/// サイドパネル描画のエントリーポイント
pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("カテゴリ一覧");

            ui.separator();

            // メインのリストエリア
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                // フッターエリア
                if let Some(act) = draw_footer(ui) {
                    action = Some(act);
                }

                ui.separator();

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        if let Some(act) = category_list::show(ui, data, &state.selection) {
                            action = Some(act);
                        }
                    },
                );
            });
        });

    action
}

/// フッター（カテゴリ追加ボタン）の描画
fn draw_footer(ui: &mut egui::Ui) -> Option<Action> {
    // 逆順で登録
    let mut action = None;

    ui.add_space(5.0);

    let btn_size = egui::vec2(ui.available_width(), 30.0);
    if ui
        .add_sized(btn_size, egui::Button::new("＋ カテゴリ追加"))
        .clicked()
    {
        action = Some(Action::ShowAddCategoryModal);
    }

    action
}
