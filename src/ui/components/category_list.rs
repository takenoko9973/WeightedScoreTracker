use crate::models::{AppData, CategoryData};
use crate::ui::Action;
use crate::ui::state::SelectionState;
use eframe::egui::{self, UiKind};

/// カテゴリリスト描画のエントリーポイント
pub fn show(ui: &mut egui::Ui, data: &AppData, selection: &SelectionState) -> Option<Action> {
    let mut action = None;

    egui::ScrollArea::vertical()
        .max_height(ui.available_height())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // カテゴリを日付順（新しい順）にソート
            let mut categories: Vec<_> = data.categories.iter().collect();
            categories.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

            // 各カテゴリを描画
            for (cat_name, cat_data) in categories {
                if let Some(act) = draw_single_category(ui, cat_name, cat_data, selection) {
                    action = Some(act);
                }
            }
        });

    action
}

/// 1つのカテゴリ（折りたたみヘッダー）の描画
fn draw_single_category(
    ui: &mut egui::Ui,
    cat_name: &str,
    cat_data: &CategoryData,
    selection: &SelectionState,
) -> Option<Action> {
    let mut action = None;

    let header_response = egui::CollapsingHeader::new(cat_name)
        .id_salt(cat_name)
        .default_open(true)
        .show(ui, |ui| {
            if let Some(act) = draw_category_contents(ui, cat_name, cat_data, selection) {
                action = Some(act);
            }
        });

    // カテゴリに対する右クリックメニュー
    header_response.header_response.context_menu(|ui| {
        if ui.button("✏ 名前を変更").clicked() {
            action = Some(Action::ShowRenameCategoryModal(cat_name.to_string()));
            ui.close_kind(UiKind::Menu);
        }
        if ui.button("🗑 このカテゴリを削除").clicked() {
            action = Some(Action::ShowDeleteCategoryConfirm(cat_name.to_string()));
            ui.close_kind(UiKind::Menu);
        }
    });

    action
}

/// カテゴリの中身（項目リストと追加ボタン）の描画
fn draw_category_contents(
    ui: &mut egui::Ui,
    cat_name: &str,
    cat_data: &CategoryData,
    selection: &SelectionState,
) -> Option<Action> {
    let mut action = None;

    // 項目を日付順にソート
    let mut items: Vec<_> = cat_data.items.iter().collect();
    items.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    // 各項目を描画
    items
        .iter()
        .filter_map(|(item_name, _)| draw_single_item(ui, cat_name, item_name, selection))
        .for_each(|act| action = Some(act));

    ui.add_space(5.0);

    // 項目追加ボタン
    if ui.button("＋ 項目").clicked() {
        action = Some(Action::ShowAddItemModal(cat_name.to_string()));
    }

    ui.add_space(5.0);

    action
}

/// 1項目描画
fn draw_single_item(
    ui: &mut egui::Ui,
    cat_name: &str,
    item_name: &str,
    selection: &SelectionState,
) -> Option<Action> {
    let mut action = None;

    // 選択状態の判定パスが state.selection... に変わっている点に注意
    let is_selected = selection.current_category.as_deref() == Some(cat_name)
        && selection.current_item.as_deref() == Some(item_name);

    let response = ui.selectable_label(is_selected, item_name);

    // 左クリック: 選択
    if response.clicked() {
        action = Some(Action::SelectItem(
            cat_name.to_string(),
            item_name.to_string(),
        ));
    }

    // 右クリック: 削除メニュー
    response.context_menu(|ui| {
        if ui.button("🗑 この項目を削除").clicked() {
            action = Some(Action::ShowDeleteItemConfirm(
                cat_name.to_string(),
                item_name.to_string(),
            ));
            ui.close_kind(UiKind::Menu);
        }
    });

    action
}
