pub mod category_list;

use crate::action::Action;
use crate::domain::{TagData, TrackerModel};
use crate::ui::state::{ItemSort, UiState};
use eframe::egui::{self, Align, Layout};

const SIDE_PANEL_WIDTH: f32 = 320.0;
const SIDE_PANEL_MIN_WIDTH: f32 = 220.0;
const SIDE_PANEL_MAX_WIDTH: f32 = 420.0;

pub struct SidePanel {}

impl SidePanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        model: &TrackerModel,
        state: &mut UiState,
        enabled: bool,
    ) -> Option<Action> {
        let mut action = None;

        egui::SidePanel::left("side_panel")
            .resizable(true)
            .default_width(SIDE_PANEL_WIDTH)
            .min_width(SIDE_PANEL_MIN_WIDTH)
            .max_width(SIDE_PANEL_MAX_WIDTH)
            .show(ctx, |ui| {
                if !enabled {
                    ui.disable();
                }

                egui::TopBottomPanel::top("header_panel").show_inside(ui, |ui| {
                    if let Some(header_action) = show_header(ui, &model.data.tags, state) {
                        action = Some(header_action);
                    }
                });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    if let Some(list_action) =
                        category_list::show(ui, &model.data, &model.selection, state)
                    {
                        action = Some(list_action);
                    }
                });
            });

        action
    }
}

fn show_header(
    ui: &mut egui::Ui,
    tags: &std::collections::HashMap<crate::domain::TagId, TagData>,
    state: &mut UiState,
) -> Option<Action> {
    let mut action = None;
    let button_width = 28.0;

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if ui
            .add_sized([button_width, 26.0], egui::Button::new("＋"))
            .on_hover_text("カテゴリを追加")
            .clicked()
        {
            action = Some(Action::ShowAddCategoryModal);
        }

        ui.menu_button("⋯", |ui| {
            if ui.button("タグを管理").clicked() {
                action = Some(Action::ShowTagManagerModal);
                ui.close_kind(egui::UiKind::Menu);
            }
        });

        let filter_label = if state.selected_tag_ids.is_empty() {
            "タグ".to_string()
        } else {
            format!("タグ {}", state.selected_tag_ids.len())
        };
        ui.menu_button(filter_label, |ui| {
            ui.set_min_width(150.0);
            let mut sorted_tags: Vec<_> = tags.values().collect();
            sorted_tags.sort_by(|a, b| a.name.cmp(&b.name));
            for tag in sorted_tags {
                let mut selected = state.selected_tag_ids.contains(&tag.id);
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(tag.color[0], tag.color[1], tag.color[2]),
                        "●",
                    );
                    changed = ui.checkbox(&mut selected, &tag.name).changed();
                });
                if changed {
                    if selected {
                        state.selected_tag_ids.insert(tag.id);
                    } else {
                        state.selected_tag_ids.remove(&tag.id);
                    }
                }
            }
            if !state.selected_tag_ids.is_empty() && ui.button("選択を解除").clicked() {
                state.selected_tag_ids.clear();
                ui.close_kind(egui::UiKind::Menu);
            }
        });

        ui.menu_button("⇅", |ui| {
            ui.set_min_width(110.0);
            ui.selectable_value(&mut state.item_sort, ItemSort::Recent, "最近更新");
            ui.selectable_value(&mut state.item_sort, ItemSort::Name, "名前");
            ui.selectable_value(&mut state.item_sort, ItemSort::Manual, "手動");
        });

        let search_width = ui.available_width();
        ui.add_sized(
            [search_width, 26.0],
            egui::TextEdit::singleline(&mut state.search_query).hint_text("項目・メモ・タグを検索"),
        );
    });

    action
}
