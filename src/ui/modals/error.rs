use crate::ui::modals::{Modal, ModalResult};
use eframe::egui;

pub struct ErrorModal {
    error_msg: String,
}

impl ErrorModal {
    pub fn new(error_msg: String) -> Self {
        Self { error_msg }
    }
}

impl Modal for ErrorModal {
    fn show(&mut self, ctx: &egui::Context) -> ModalResult {
        let mut result = ModalResult::KeepOpen;
        let mut open = true;

        egui::Window::new("エラー")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(&self.error_msg);

                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    if ui.button("閉じる").clicked() {
                        result = ModalResult::Close;
                    }
                });
            });

        if !open {
            result = ModalResult::Close;
        }

        result
    }
}
