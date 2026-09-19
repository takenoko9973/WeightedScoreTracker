pub mod add_category;
pub mod add_item;
pub mod confirm;
pub mod edit_category;
pub mod edit_item;
pub mod error;
pub mod tag_manager;

use crate::action::Action;
use crate::domain::{TagData, TagId};
use crate::ui::modals::error::ErrorModal;
use crate::ui::state::UiState;
use eframe::egui;

fn tag_chip(ui: &mut egui::Ui, tag: &TagData, selected: bool) -> egui::Response {
    let color = egui::Color32::from_rgb(tag.color[0], tag.color[1], tag.color[2]);
    ui.add(
        egui::Button::new((egui::RichText::new("●").color(color), tag.name.as_str()))
            .selected(selected)
            .corner_radius(ui.spacing().interact_size.y / 2.0)
            .truncate(),
    )
    .on_hover_text(&tag.name)
}

fn sort_tags(tags: &mut [TagData]) {
    tags.sort_by(|a, b| a.name.cmp(&b.name));
}

fn toggle_tag(selected_tag_ids: &mut Vec<TagId>, id: TagId, selected: bool) {
    if selected {
        if !selected_tag_ids.contains(&id) {
            selected_tag_ids.push(id);
        }
    } else {
        selected_tag_ids.retain(|tag_id| *tag_id != id);
    }
}

fn show_tag_chips(
    ui: &mut egui::Ui,
    available_tags: &[TagData],
    selected_tag_ids: &mut Vec<TagId>,
) {
    ui.horizontal_wrapped(|ui| {
        for tag in available_tags {
            let selected = selected_tag_ids.contains(&tag.id);
            if tag_chip(ui, tag, selected).clicked() {
                toggle_tag(selected_tag_ids, tag.id, !selected);
            }
        }
    });
}

/// モーダル管理
pub struct ModalLayer {
    active_modal: Option<Box<dyn Modal>>,
}

impl ModalLayer {
    pub fn new() -> Self {
        Self { active_modal: None }
    }

    /// モーダルを開く
    pub fn open<M: Modal + 'static>(&mut self, modal: M) {
        self.active_modal = Some(Box::new(modal));
    }

    /// 描画処理
    pub fn show(&mut self, ctx: &egui::Context, state: &mut UiState) -> Option<Action> {
        let mut action = None;
        let mut should_close = false;

        // エラーモーダル
        if let Some(msg) = &state.error_message {
            let mut error_modal = ErrorModal::new(msg.to_string());

            match error_modal.show(ctx) {
                ModalResult::KeepOpen => {}
                ModalResult::Close => {
                    state.error_message = None;
                }
                ModalResult::Dispatch(_) => {}
            }
        }

        // 通常モーダル
        if let Some(modal) = &mut self.active_modal {
            match modal.show(ctx) {
                ModalResult::KeepOpen => {}
                ModalResult::Close => should_close = true,
                ModalResult::Dispatch(act) => {
                    action = Some(act);
                    should_close = true;
                }
            }
        }

        if should_close {
            self.active_modal = None;
        }

        action
    }

    /// モーダルが開いているかどうか
    pub fn is_open(&self) -> bool {
        self.active_modal.is_some()
    }
}

/// モーダルの実行結果
pub enum ModalResult {
    KeepOpen,         // 開いたまま
    Close,            // 閉じる（キャンセルなど）
    Dispatch(Action), // アクションを実行して閉じる
}

/// モーダル用インターフェース
pub trait Modal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult;
}
