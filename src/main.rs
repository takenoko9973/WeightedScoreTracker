// リリースビルドでコンソール非表示
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod logic;
mod models;
mod persistence;
mod ui;
mod utils;

use app::ScoreTracker;
use eframe::egui;
use ui::fonts::setup_custom_fonts;

fn main() -> eframe::Result<()> {
    // ウィンドウ設定
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Weighted Score Tracker",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(ScoreTracker::new(cc)))
        }),
    )
}
