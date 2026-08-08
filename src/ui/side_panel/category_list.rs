use crate::action::Action;
use crate::domain::{
    AppData, CategoryData, ItemData, MoveDirection, SelectionState, TagData, TagId,
};
use crate::ui::state::{ItemSort, UiState};
use eframe::egui::{self, Align, Layout, Sense, UiBuilder, UiKind};
use std::collections::{HashMap, HashSet};

pub const ITEM_ROW_HEIGHT: f32 = 44.0;
const ITEM_MEMO_FONT_SIZE: f32 = 12.0;
const ITEM_TEXT_SPACING: f32 = 2.0;
const TAG_MARKER_WIDTH: f32 = 16.0;
const TAG_ITEM_SPACING: f32 = 3.0;
const HIDDEN_TAGS_WIDTH: f32 = 28.0;
const TAG_REGION_MARKER_COUNT: usize = 4;
const ITEM_INFO_MIN_WIDTH: f32 = 96.0;
// タグが増えても項目情報を圧迫しすぎないよう、丸4個と +N が収まる幅を上限にする。
const TAG_REGION_MAX_WIDTH: f32 = TAG_REGION_MARKER_COUNT as f32 * TAG_MARKER_WIDTH
    + TAG_REGION_MARKER_COUNT as f32 * TAG_ITEM_SPACING
    + HIDDEN_TAGS_WIDTH;

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
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
                if ui.small_button("＋").on_hover_text("項目を追加").clicked() {
                    action = Some(Action::ShowAddItemModal(cat_name.to_string()));
                }
                ui.label(item_names.len().to_string());

                let name_width = ui.available_width().max(0.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(name_width, ui.spacing().interact_size.y),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(cat_name).strong())
                                .truncate()
                                .sense(Sense::hover())
                                .halign(Align::Min),
                        )
                        .on_hover_text(cat_name);
                    },
                );
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
            let content_width = ui.available_width().max(0.0);
            let has_tags = item.tag_ids.iter().any(|id| tags.contains_key(id));
            let split_width = (content_width - ui.spacing().item_spacing.x).max(0.0);
            let tag_available_width = (split_width - ITEM_INFO_MIN_WIDTH).max(0.0);
            let tags_width = tag_region_width(item, tags, tag_available_width);
            let info_width = if has_tags {
                (split_width - tags_width).max(0.0)
            } else {
                content_width
            };
            ui.allocate_ui_with_layout(
                egui::vec2(info_width, ITEM_ROW_HEIGHT - 4.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.y = ITEM_TEXT_SPACING;
                    ui.add(
                        egui::Label::new(egui::RichText::new(item_name).strong())
                            .truncate()
                            .halign(Align::Min),
                    );
                    let subtitle = if item.subtitle.is_empty() {
                        " "
                    } else {
                        &item.subtitle
                    };
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(subtitle)
                                .weak()
                                .size(ITEM_MEMO_FONT_SIZE),
                        )
                        .truncate()
                        .halign(Align::Min),
                    );
                },
            );
            if has_tags {
                ui.allocate_ui_with_layout(
                    egui::vec2(tags_width, ITEM_ROW_HEIGHT - 4.0),
                    Layout::left_to_right(Align::Center),
                    |ui| draw_item_tags(ui, item, tags),
                );
            }
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

fn tag_region_width(item: &ItemData, tags: &HashMap<TagId, TagData>, available_width: f32) -> f32 {
    let tag_count = item
        .tag_ids
        .iter()
        .filter(|id| tags.contains_key(id))
        .count();
    let marker_width = tag_count as f32 * TAG_MARKER_WIDTH;
    let spacing_width = tag_count.saturating_sub(1) as f32 * TAG_ITEM_SPACING;
    (marker_width + spacing_width)
        .min(TAG_REGION_MAX_WIDTH)
        .min(available_width.max(0.0))
}

fn visible_tag_count(tag_count: usize, available_width: f32) -> usize {
    let all_markers_width =
        tag_count as f32 * TAG_MARKER_WIDTH + tag_count.saturating_sub(1) as f32 * TAG_ITEM_SPACING;
    if all_markers_width <= available_width {
        return tag_count;
    }

    let mut used_width = 0.0;
    let mut shown = 0;
    for index in 0..tag_count {
        let hidden_after = tag_count.saturating_sub(index + 1);
        let marker_spacing = if shown > 0 { TAG_ITEM_SPACING } else { 0.0 };
        let hidden_indicator_width = if hidden_after > 0 {
            TAG_ITEM_SPACING + HIDDEN_TAGS_WIDTH
        } else {
            0.0
        };
        let required_width = marker_spacing + TAG_MARKER_WIDTH + hidden_indicator_width;
        if used_width + required_width > available_width {
            break;
        }

        used_width += marker_spacing + TAG_MARKER_WIDTH;
        shown += 1;
    }

    shown
}

fn draw_item_tags(ui: &mut egui::Ui, item: &ItemData, tags: &HashMap<TagId, TagData>) {
    let tag_data: Vec<_> = item
        .tag_ids
        .iter()
        .filter_map(|tag_id| tags.get(tag_id))
        .collect();
    let available_width = ui.available_width();
    let total = tag_data.len();
    let shown = visible_tag_count(total, available_width);

    ui.spacing_mut().item_spacing.x = TAG_ITEM_SPACING;
    let row_height = ui.spacing().interact_size.y;
    for tag in tag_data.iter().take(shown) {
        ui.add_sized(
            [TAG_MARKER_WIDTH, row_height],
            egui::Label::new(egui::RichText::new("●").color(egui::Color32::from_rgb(
                tag.color[0],
                tag.color[1],
                tag.color[2],
            ))),
        )
        .on_hover_text(&tag.name);
    }
    if shown < total {
        let hidden_names = tag_data
            .iter()
            .skip(shown)
            .map(|tag| tag.name.as_str())
            .collect::<Vec<_>>();
        ui.add_sized(
            [HIDDEN_TAGS_WIDTH.min(ui.available_width()), row_height],
            egui::Label::new(format!("+{}", total - shown)).truncate(),
        )
        .on_hover_text(format!("タグ: {}", hidden_names.join(", ")));
    }
}

fn item_tooltip(item_name: &str, item: &ItemData, tags: &HashMap<TagId, TagData>) -> String {
    let tag_names: Vec<_> = item
        .tag_ids
        .iter()
        .filter_map(|id| tags.get(id).map(|tag| tag.name.as_str()))
        .collect();
    let subtitle = if item.subtitle.is_empty() {
        "メモなし"
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
    fn tag_region_width_is_bounded_without_becoming_negative() {
        let tags: HashMap<TagId, TagData> = (1..=6)
            .map(|id| {
                (
                    id,
                    TagData {
                        id,
                        name: format!("Tag {id}"),
                        color: [0, 0, 0],
                    },
                )
            })
            .collect();
        let (_, no_tags) = item("No tags", "", Vec::new());
        let (_, one_tag) = item("One tag", "", vec![1]);
        let (_, four_tags) = item("Four tags", "", vec![1, 2, 3, 4]);
        let (_, many_tags) = item("Many tags", "", vec![1, 2, 3, 4, 5, 6]);

        assert_eq!(tag_region_width(&no_tags, &tags, 200.0), 0.0);
        assert_eq!(tag_region_width(&one_tag, &tags, 200.0), TAG_MARKER_WIDTH);
        assert_eq!(
            tag_region_width(&four_tags, &tags, 200.0),
            4.0 * TAG_MARKER_WIDTH + 3.0 * TAG_ITEM_SPACING
        );
        assert_eq!(
            tag_region_width(&many_tags, &tags, 200.0),
            TAG_REGION_MAX_WIDTH
        );
        assert_eq!(tag_region_width(&many_tags, &tags, 40.0), 40.0);
        assert_eq!(tag_region_width(&many_tags, &tags, -1.0), 0.0);
    }

    #[test]
    fn tag_display_reserves_space_for_the_hidden_count() {
        assert_eq!(visible_tag_count(6, TAG_REGION_MAX_WIDTH), 4);
        assert_eq!(visible_tag_count(6, 85.0), 3);
        assert_eq!(visible_tag_count(6, HIDDEN_TAGS_WIDTH), 0);
        assert_eq!(visible_tag_count(4, TAG_REGION_MAX_WIDTH), 4);
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
