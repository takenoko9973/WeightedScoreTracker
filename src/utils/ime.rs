use eframe::egui;

/// egui::Response を拡張するためのトレイト
pub trait ImeFocusExtension {
    /// IME入力中（変換確定など）にEnter/Tabでフォーカスが外れてしまうのを防ぐ
    fn handle_ime_focus(&self, ui: &egui::Ui);
}

impl ImeFocusExtension for egui::Response {
    fn handle_ime_focus(&self, ui: &egui::Ui) {
        // このフレームでIMEイベントが発生しているかチェック
        let is_ime = ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Ime(_))));

        // 「フォーカスが外れた」かつ「IME操作中」かつ「Enter/Tabキー」なら、フォーカスを戻す
        if self.lost_focus()
            && is_ime
            && ui.input(|i| i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Tab))
        {
            self.request_focus();
        }
    }
}
