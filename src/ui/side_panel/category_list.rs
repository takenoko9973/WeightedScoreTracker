use crate::action::Action;
use crate::domain::{
    AppData, CategoryData, ItemData, MoveDirection, SelectionState, TagData, TagId,
};
use crate::ui::state::{ItemSort, UiState};
use eframe::egui::{self, Align, Layout, Sense, UiBuilder, UiKind};
use std::collections::{HashMap, HashSet};

pub const ITEM_ROW_HEIGHT: f32 = 44.0;

fn normalized_query(query: &str) -> String {
    query.to_lowercase()
}

fn contains_case_insensitive(text: &str, query: &str) -> bool {
    query.is_empty() || text.to_lowercase().contains(&query.to_lowercase())
}

fn item_matches_query(
    item_name: &str,
    item: &ItemData,
    tags: &HashMap<TagId, TagData>,
    query: &str,
) -> bool {
    contains_case_insensitive(item_name, query)
        || contains_case_insensitive(&item.subtitle, query)
        || item
            .tag_ids
            .iter()
            .filter_map(|id| tags.get(id))
            .any(|tag| contains_case_insensitive(&tag.name, query))
}

fn matches_selected_tags(item: &ItemData, selected_tag_ids: &HashSet<TagId>) -> bool {
    selected_tag_ids.iter().all(|id| item.tag_ids.contains(id))
}

fn visible_item_names<'a>(
    cat_data: &'a CategoryData,
    manual_item_names: &[&'a str],
    category_matches: bool,
    query: &str,
    selected_tag_ids: &HashSet<TagId>,
    tags: &HashMap<TagId, TagData>,
    sort: ItemSort,
) -> Vec<&'a str> {
    let mut names: Vec<_> = match sort {
        ItemSort::Manual => manual_item_names.to_vec(),
        ItemSort::Recent | ItemSort::Name => cat_data.items.keys().map(String::as_str).collect(),
    };

    names.retain(|name| {
        let Some(item) = cat_data.items.get(*name) else {
            return false;
        };
        let search_matches = category_matches || item_matches_query(name, item, tags, query);
        search_matches && matches_selected_tags(item, selected_tag_ids)
    });

    match sort {
        ItemSort::Recent => names.sort_by(|a, b| {
            let item_a = &cat_data.items[*a];
            let item_b = &cat_data.items[*b];
            item_b
                .updated_at
                .cmp(&item_a.updated_at)
                .then_with(|| a.to_lowercase().cmp(&b.to_lowercase()))
        }),
        ItemSort::Name => names.sort_by(|a, b| {
            a.to_lowercase()
                .cmp(&b.to_lowercase())
                .then_with(|| a.cmp(b))
        }),
        ItemSort::Manual => {}
    }

    names
}

fn category_matches_query(name: &str, query: &str) -> bool {
    !query.is_empty() && contains_case_insensitive(name, query)
}

fn should_force_category_open(
    has_filter: bool,
    visible_item_count: usize,
    is_selected_category: bool,
) -> bool {
    (has_filter && visible_item_count > 0) || is_selected_category
}

fn category_menu_action(
    cat_name: &str,
    add_item_clicked: bool,
    edit_clicked: bool,
    delete_clicked: bool,
    move_up_clicked: bool,
    move_down_clicked: bool,
) -> Option<Action> {
    if add_item_clicked {
        Some(Action::ShowAddItemModal(cat_name.to_string()))
    } else if edit_clicked {
        Some(Action::ShowEditCategoryModal(cat_name.to_string()))
    } else if delete_clicked {
        Some(Action::ShowDeleteCategoryConfirm(cat_name.to_string()))
    } else if move_up_clicked {
        Some(Action::MoveCategory(
            cat_name.to_string(),
            MoveDirection::Up,
        ))
    } else if move_down_clicked {
        Some(Action::MoveCategory(
            cat_name.to_string(),
            MoveDirection::Down,
        ))
    } else {
        None
    }
}

fn item_menu_action(
    cat_name: &str,
    item_name: &str,
    edit_clicked: bool,
    delete_clicked: bool,
    move_up_clicked: bool,
    move_down_clicked: bool,
) -> Option<Action> {
    if edit_clicked {
        Some(Action::ShowEditItemModal(
            cat_name.to_string(),
            item_name.to_string(),
        ))
    } else if delete_clicked {
        Some(Action::ShowDeleteItemConfirm(
            cat_name.to_string(),
            item_name.to_string(),
        ))
    } else if move_up_clicked {
        Some(Action::MoveItem(
            cat_name.to_string(),
            item_name.to_string(),
            MoveDirection::Up,
        ))
    } else if move_down_clicked {
        Some(Action::MoveItem(
            cat_name.to_string(),
            item_name.to_string(),
            MoveDirection::Down,
        ))
    } else {
        None
    }
}

pub fn show(
    ui: &mut egui::Ui,
    data: &AppData,
    selection: &SelectionState,
    state: &UiState,
) -> Option<Action> {
    let query = normalized_query(&state.search_query);
    let mut action = None;

    egui::ScrollArea::vertical()
        .max_height(ui.available_height())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            for cat_name in data.ordered_category_names() {
                let Some(cat_data) = data.categories.get(cat_name) else {
                    continue;
                };
                let manual_item_names = data
                    .ordered_item_names(cat_name)
                    .expect("カテゴリ一覧の項目順はカテゴリ本体と一致している");
                let category_matches = category_matches_query(cat_name, &query);
                let item_names = visible_item_names(
                    cat_data,
                    &manual_item_names,
                    category_matches,
                    &query,
                    &state.selected_tag_ids,
                    &data.tags,
                    state.item_sort,
                );
                let has_filter = !query.is_empty() || !state.selected_tag_ids.is_empty();
                if has_filter && !category_matches && item_names.is_empty() {
                    continue;
                }

                if let Some(next_action) = draw_single_category(
                    ui,
                    cat_name,
                    cat_data,
                    &item_names,
                    &data.tags,
                    selection,
                    !has_filter,
                    state.item_sort == ItemSort::Manual && !has_filter,
                    should_force_category_open(
                        has_filter,
                        item_names.len(),
                        selection.category.as_deref() == Some(cat_name),
                    ),
                ) {
                    action = Some(next_action);
                }
            }
        });

    action
}

#[allow(clippy::too_many_arguments)]
fn draw_single_category(
    ui: &mut egui::Ui,
    cat_name: &str,
    cat_data: &CategoryData,
    item_names: &[&str],
    tags: &HashMap<TagId, TagData>,
    selection: &SelectionState,
    category_move_enabled: bool,
    item_move_enabled: bool,
    force_open: bool,
) -> Option<Action> {
    let mut action = None;
    let id = ui.make_persistent_id(("category", cat_name));
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
    if force_open {
        state.set_open(true);
    }

    let (_, header, _) = state
        .show_header(ui, |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(cat_name).strong())
                    .truncate()
                    .sense(Sense::hover()),
            )
            .on_hover_text(cat_name);
            ui.label(format!("{}", item_names.len()));
            if ui.small_button("＋").on_hover_text("項目を追加").clicked() {
                action = Some(Action::ShowAddItemModal(cat_name.to_string()));
            }
            ui.menu_button("⋯", |ui| {
                let add = ui.button("＋ 項目を追加").clicked();
                let edit = ui.button("✏ 名前を変更").clicked();
                let delete = ui.button("🗑 カテゴリを削除").clicked();
                ui.separator();
                let move_up = ui
                    .add_enabled(category_move_enabled, egui::Button::new("↑ 上へ"))
                    .clicked();
                let move_down = ui
                    .add_enabled(category_move_enabled, egui::Button::new("↓ 下へ"))
                    .clicked();
                if let Some(next_action) =
                    category_menu_action(cat_name, add, edit, delete, move_up, move_down)
                {
                    action = Some(next_action);
                    ui.close_kind(UiKind::Menu);
                }
            });
        })
        .body(|ui| {
            for item_name in item_names {
                let Some(item) = cat_data.items.get(*item_name) else {
                    continue;
                };
                if let Some(next_action) = draw_single_item(
                    ui,
                    cat_name,
                    item_name,
                    item,
                    tags,
                    selection,
                    item_move_enabled,
                ) {
                    action = Some(next_action);
                }
            }
        });

    header.response.context_menu(|ui| {
        let add = ui.button("＋ 項目を追加").clicked();
        let edit = ui.button("✏ 名前を変更").clicked();
        let delete = ui.button("🗑 カテゴリを削除").clicked();
        ui.separator();
        let move_up = ui
            .add_enabled(category_move_enabled, egui::Button::new("↑ 上へ"))
            .clicked();
        let move_down = ui
            .add_enabled(category_move_enabled, egui::Button::new("↓ 下へ"))
            .clicked();
        if let Some(next_action) =
            category_menu_action(cat_name, add, edit, delete, move_up, move_down)
        {
            action = Some(next_action);
            ui.close_kind(UiKind::Menu);
        }
    });

    action
}

fn draw_single_item(
    ui: &mut egui::Ui,
    cat_name: &str,
    item_name: &str,
    item: &ItemData,
    tags: &HashMap<TagId, TagData>,
    selection: &SelectionState,
    item_move_enabled: bool,
) -> Option<Action> {
    let is_selected = selection.category.as_deref() == Some(cat_name)
        && selection.item.as_deref() == Some(item_name);
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), ITEM_ROW_HEIGHT),
        Sense::click(),
    );
    if is_selected {
        ui.painter()
            .rect_filled(rect, 3.0, ui.visuals().selection.bg_fill);
    } else if response.hovered() {
        ui.painter()
            .rect_filled(rect, 3.0, ui.visuals().widgets.hovered.bg_fill);
    }

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(4.0, 2.0)))
            .layout(Layout::left_to_right(Align::Center)),
        |ui| {
            let tags_width = (ui.available_width() * 0.45).max(70.0);
            let info_width = (ui.available_width() - tags_width - 6.0).max(30.0);
            ui.allocate_ui_with_layout(
                egui::vec2(info_width, ITEM_ROW_HEIGHT - 4.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.add_sized(
                        [info_width, 20.0],
                        egui::Label::new(egui::RichText::new(item_name).strong()).truncate(),
                    );
                    let subtitle = if item.subtitle.is_empty() {
                        " "
                    } else {
                        &item.subtitle
                    };
                    ui.add_sized(
                        [info_width, 18.0],
                        egui::Label::new(egui::RichText::new(subtitle).weak()).truncate(),
                    );
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(tags_width, ITEM_ROW_HEIGHT - 4.0),
                Layout::left_to_right(Align::Center),
                |ui| draw_item_tags(ui, item, tags),
            );
        },
    );

    let tooltip = item_tooltip(item_name, item, tags);
    let response = response.on_hover_text(tooltip);
    let mut action = None;
    response.context_menu(|ui| {
        let edit = ui.button("✏ 項目を編集...").clicked();
        let delete = ui.button("🗑 項目を削除").clicked();
        ui.separator();
        let move_up = ui
            .add_enabled(item_move_enabled, egui::Button::new("↑ 上へ"))
            .clicked();
        let move_down = ui
            .add_enabled(item_move_enabled, egui::Button::new("↓ 下へ"))
            .clicked();
        if let Some(next_action) =
            item_menu_action(cat_name, item_name, edit, delete, move_up, move_down)
        {
            ui.close_kind(UiKind::Menu);
            action = Some(next_action);
        }
    });

    if action.is_some() {
        action
    } else if response.clicked() {
        Some(Action::SelectItem(
            cat_name.to_string(),
            item_name.to_string(),
        ))
    } else {
        None
    }
}

fn draw_item_tags(ui: &mut egui::Ui, item: &ItemData, tags: &HashMap<TagId, TagData>) {
    let available_width = ui.available_width();
    let mut used_width = 0.0;
    let mut shown = 0;
    let total = item.tag_ids.len();

    ui.spacing_mut().item_spacing.x = 3.0;
    for (index, tag_id) in item.tag_ids.iter().enumerate() {
        let Some(tag) = tags.get(tag_id) else {
            continue;
        };
        let tag_width = 20.0 + tag.name.chars().count() as f32 * 7.0;
        let hidden_after = total.saturating_sub(index + 1);
        let hidden_width = if hidden_after > 0 { 28.0 } else { 0.0 };
        if shown > 0 && used_width + tag_width + hidden_width > available_width {
            break;
        }
        ui.colored_label(
            egui::Color32::from_rgb(tag.color[0], tag.color[1], tag.color[2]),
            "●",
        );
        ui.add(egui::Label::new(&tag.name).truncate());
        used_width += tag_width;
        shown += 1;
    }
    if shown < total {
        ui.label(format!("+{}", total - shown));
    }
}

fn item_tooltip(item_name: &str, item: &ItemData, tags: &HashMap<TagId, TagData>) -> String {
    let tag_names: Vec<_> = item
        .tag_ids
        .iter()
        .filter_map(|id| tags.get(id).map(|tag| tag.name.as_str()))
        .collect();
    let subtitle = if item.subtitle.is_empty() {
        "補足なし"
    } else {
        &item.subtitle
    };
    if tag_names.is_empty() {
        format!("{}\n{}", item_name, subtitle)
    } else {
        format!(
            "{}\n{}\nタグ: {}",
            item_name,
            subtitle,
            tag_names.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn item(name: &str, subtitle: &str, tag_ids: Vec<TagId>) -> (String, ItemData) {
        (
            name.to_string(),
            ItemData {
                scores: Vec::new(),
                decay_rate: 0.9,
                subtitle: subtitle.to_string(),
                tag_ids,
                updated_at: Utc::now(),
            },
        )
    }

    #[test]
    fn search_matches_title_subtitle_and_tag_name_case_insensitively() {
        let tags = HashMap::from([(
            1,
            TagData {
                id: 1,
                name: "Urgent".to_string(),
                color: [255, 0, 0],
            },
        )]);
        let (_, data) = item("Readme", "Weekly note", vec![1]);
        assert!(item_matches_query("Readme", &data, &tags, "read"));
        assert!(item_matches_query("Readme", &data, &tags, "WEEKLY"));
        assert!(item_matches_query("Readme", &data, &tags, "urgent"));
    }

    #[test]
    fn selected_tags_are_an_and_filter() {
        let item = ItemData {
            scores: Vec::new(),
            decay_rate: 0.9,
            subtitle: String::new(),
            tag_ids: vec![1, 2],
            updated_at: Utc::now(),
        };
        assert!(matches_selected_tags(&item, &HashSet::from([1, 2])));
        assert!(!matches_selected_tags(&item, &HashSet::from([1, 3])));
    }

    #[test]
    fn category_match_returns_all_items_that_pass_tag_filter() {
        let category = CategoryData {
            items: HashMap::from([item("A", "", vec![1]), item("B", "", vec![2])]),
            item_order: vec!["A".to_string(), "B".to_string()],
            created_at: Utc::now(),
        };
        let tags = HashMap::from([
            (
                1,
                TagData {
                    id: 1,
                    name: "One".to_string(),
                    color: [1, 1, 1],
                },
            ),
            (
                2,
                TagData {
                    id: 2,
                    name: "Two".to_string(),
                    color: [2, 2, 2],
                },
            ),
        ]);
        let names = visible_item_names(
            &category,
            &["A", "B"],
            true,
            "category",
            &HashSet::from([2]),
            &tags,
            ItemSort::Manual,
        );
        assert_eq!(names, vec!["B"]);
    }

    #[test]
    fn action_mapping_preserves_item_location_and_move_direction() {
        assert!(matches!(
            item_menu_action("Cat", "Item", false, false, true, false),
            Some(Action::MoveItem(cat, item, MoveDirection::Up)) if cat == "Cat" && item == "Item"
        ));
        assert!(matches!(
            category_menu_action("Cat", false, false, false, false, true),
            Some(Action::MoveCategory(name, MoveDirection::Down)) if name == "Cat"
        ));
    }

    #[test]
    fn filtered_matches_and_selected_categories_are_forced_open() {
        assert!(should_force_category_open(true, 1, false));
        assert!(should_force_category_open(false, 0, true));
        assert!(!should_force_category_open(true, 0, false));
    }
}
