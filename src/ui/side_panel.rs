use crate::models::AppData;
use crate::ui::Action;
use crate::{app::UiState, models::CategoryData};
use eframe::egui::{self, UiKind};

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
                        if let Some(act) = draw_category_list(ui, data, state) {
                            action = Some(act);
                        }
                    },
                );
            });
        });

    action
}

/// カテゴリリスト全体の描画
fn draw_category_list(ui: &mut egui::Ui, data: &AppData, state: &UiState) -> Option<Action> {
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
                if let Some(act) = draw_single_category(ui, cat_name, cat_data, state) {
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
    state: &UiState,
) -> Option<Action> {
    let mut action = None;

    let header_response = egui::CollapsingHeader::new(cat_name)
        .id_salt(cat_name)
        .default_open(true)
        .show(ui, |ui| {
            if let Some(act) = draw_category_contents(ui, cat_name, cat_data, state) {
                action = Some(act);
            }
        });

    header_response.header_response.context_menu(|ui| {
        if ui.button("✏ 名前を変更").clicked() {
            action = Some(Action::ShowRenameCategoryModal(cat_name.to_string()));
            ui.close_kind(UiKind::Menu);
        }
        if ui.button("🗑 このカテゴリを削除").clicked() {
            action = Some(Action::ShowDeleteCategoryConfirm(cat_name.to_string()));
            ui.close_kind(egui::UiKind::Menu);
        }
    });

    action
}

/// カテゴリの中身（項目リストと追加ボタン）の描画
fn draw_category_contents(
    ui: &mut egui::Ui,
    cat_name: &str,
    cat_data: &CategoryData,
    state: &UiState,
) -> Option<Action> {
    let mut action = None;

    // 項目を日付順にソート
    let mut items: Vec<_> = cat_data.items.iter().collect();
    items.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

    // 各項目を描画
    for (item_name, _) in items {
        if let Some(act) = draw_single_item(ui, cat_name, item_name, state) {
            action = Some(act);
        }
    }

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
    state: &UiState,
) -> Option<Action> {
    let mut action = None;

    let is_selected = state.selection.current_category.as_deref() == Some(cat_name)
        && state.selection.current_item.as_deref() == Some(item_name);

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
