// リリースビルドでコンソール非表示
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod constants;
mod logic;
mod models;
mod persistence;
mod ui;
mod utils;

use app::WeightedScoreTracker;
use constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui;
use ui::fonts::setup_custom_fonts;

fn main() -> eframe::Result<()> {
    // ウィンドウ設定
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
        ..Default::default()
    };

    eframe::run_native(
        "Weighted Score Tracker",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(WeightedScoreTracker::new(cc)))
        }),
    )
}
