use eframe::egui;

use crate::models::app::AppData;
use crate::ui::Action;
use crate::ui::components::category_list;
use crate::ui::state::UiState;

/// サイドパネル描画のエントリーポイント
pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            // 上下に要素を先に配置
            egui::TopBottomPanel::top("header_panel").show_inside(ui, |ui| {
                ui.heading("カテゴリ一覧");
            });
            egui::TopBottomPanel::bottom("footer_panel").show_inside(ui, |ui| {
                if let Some(a) = draw_footer(ui) {
                    action = Some(a);
                }
            });

            // メインのリストエリア
            egui::CentralPanel::default().show_inside(ui, |ui| {
                if let Some(a) = category_list::show(ui, data, &state.selection) {
                    action = Some(a);
                }
            });
        });

    action
}

/// フッター描画
fn draw_footer(ui: &mut egui::Ui) -> Option<Action> {
    let mut action = None;

    ui.add_space(3.0);

    let btn_size = egui::vec2(ui.available_width(), 30.0);
    if ui
        .add_sized(btn_size, egui::Button::new("＋ カテゴリ追加"))
        .clicked()
    {
        action = Some(Action::ShowAddCategoryModal);
    }

    action
}
