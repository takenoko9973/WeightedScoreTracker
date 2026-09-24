// リリースビルドでコンソール非表示
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod action;
mod app;
mod application;
mod constants;
mod domain;
mod infrastructure;
mod logic;
mod ui;
mod utils;

use app::WeightedScoreTracker;
use constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use eframe::egui;
use ui::fonts::setup_custom_fonts;

fn main() -> eframe::Result<()> {
    // ウィンドウ設定
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
        ..Default::default()
    };

    // この環境では Vulkan/DX12 のウィンドウ移動がカクつくため、検証済みの GL に固定する。
    let eframe::egui_wgpu::WgpuSetup::CreateNew(ref mut setup) = options.wgpu_options.wgpu_setup
    else {
        unreachable!("既定の wgpu 設定は新規インスタンスを作成する");
    };
    setup.instance_descriptor.backends = eframe::wgpu::Backends::GL;

    eframe::run_native(
        "Weighted Score Tracker",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(WeightedScoreTracker::new(cc)))
        }),
    )
}
