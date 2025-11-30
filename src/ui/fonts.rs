use eframe::egui;

pub fn setup_custom_fonts(ctx: &egui::Context) {
    // 現在のフォント設定を取得
    let mut fonts = egui::FontDefinitions::default();

    // 日本語フォント（可変ウェイト）を追加
    let font_paths = [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "C:\\Windows\\Fonts\\meiryo.ttc",
    ];

    let mut font_data_loaded = false;
    for path in font_paths {
        // パス確認
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "my_font".to_owned(),
                egui::FontData::from_owned(font_data)
                    .tweak(egui::FontTweak {
                        scale: 1.2,
                        ..Default::default()
                    })
                    .into(),
            );
            font_data_loaded = true;
            break;
        }
    }

    if font_data_loaded {
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "my_font".to_owned());
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "my_font".to_owned());
        ctx.set_fonts(fonts);
    } else {
        println!("日本語フォントが見つかりませんでした。");
    }
}
