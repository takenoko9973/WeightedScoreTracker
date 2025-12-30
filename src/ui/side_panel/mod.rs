pub mod category_list;

use crate::action::Action;
use crate::domain::AppData;
use crate::ui::state::UiState;
use eframe::egui;

pub struct SidePanel {}

impl SidePanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        data: &AppData,
        state: &mut UiState,
        enabled: bool,
    ) -> Option<Action> {
        let mut action = None;

        egui::SidePanel::left("side_panel")
            .resizable(true)
            .show(ctx, |ui| {
                if !enabled {
                    // UIの無効化
                    ui.disable();
                }

                // 上下に要素を先に配置
                egui::TopBottomPanel::top("header_panel").show_inside(ui, |ui| {
                    ui.heading("カテゴリ一覧");
                });
                egui::TopBottomPanel::bottom("footer_panel").show_inside(ui, |ui| {
                    if let Some(a) = self.show_footer(ui) {
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
    fn show_footer(&self, ui: &mut egui::Ui) -> Option<Action> {
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
}
