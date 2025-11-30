pub mod central_panel;
pub mod fonts;
pub mod modals;
pub mod side_panel;

/// アプリケーション内で発生する操作
#[derive(Debug, Clone)]
pub enum Action {
    // --- モーダル表示リクエスト ---
    ShowAddCategoryModal,
    ShowEditDecayModal,
    ShowDeleteCategoryConfirm(String), // 対象カテゴリ名
    ShowDeleteScoreConfirm(usize),     // 対象インデックス

    // --- データ操作リクエスト ---
    SelectCategory(String),        // カテゴリ選択
    AddScore(String),              // スコア追加 (文字列のまま渡す)
    ExecuteDeleteScore(usize),     // スコア削除実行
    AddCategory(String, String),   // カテゴリ追加 (名前, 減衰率)
    ExecuteDeleteCategory(String), // カテゴリ削除実行
    UpdateDecayRate(String),       // 減衰率更新 (文字列)
}
