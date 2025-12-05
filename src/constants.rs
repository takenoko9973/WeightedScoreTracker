// --- ファイル保存 ---
pub const DATA_FILENAME: &str = "scores_data.json";

// --- 減衰率 (Decay Rate) ---
pub const MIN_DECAY_RATE: f64 = 0.01;
pub const MAX_DECAY_RATE: f64 = 1.00;
pub const DEFAULT_DECAY_RATE: f64 = 0.90;

// --- ウィンドウ設定 ---
pub const WINDOW_WIDTH: f32 = 1000.0;
pub const WINDOW_HEIGHT: f32 = 600.0;

// --- フォント設定 ---
pub const FONT_SCALE: f32 = 1.2;
// 優先順位順のフォントパス
pub const FONT_PATHS: &[&str] = &[
    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
    "C:\\Windows\\Fonts\\meiryo.ttc",
];

// --- 計算・プロットロジック ---
// グラフから非表示する重みの閾値
pub const PLOT_WEIGHT_THRESHOLD: f64 = 0.1;
