pub mod central_panel;
pub mod components;
pub mod fonts;
pub mod modals;
pub mod side_panel;
pub mod state;

/// アプリケーション内で発生する操作
#[derive(Debug, Clone)]
pub enum Action {
    // --- モーダル表示リクエスト ---
    ShowAddCategoryModal,                  // カテゴリ追加
    ShowRenameCategoryModal(String),       // カテゴリ名変更 (対象カテゴリ名)
    ShowAddItemModal(String),              // 項目追加 (親カテゴリ名)
    ShowEditItemModal(String, String),     // 項目編集 (カテゴリ名, 項目名)
    ShowEditDecayModal,                    // 減衰率更新
    ShowDeleteCategoryConfirm(String),     // カテゴリ削除 (対象カテゴリ名)
    ShowDeleteItemConfirm(String, String), // 項目削除 (カテゴリ名, 項目名)
    ShowDeleteScoreConfirm(usize),         // スコア削除 (対象インデックス)

    // --- データ操作リクエスト ---
    SelectItem(String, String),        // 項目選択 (カテゴリ名, 項目名)
    AddCategory(String),               // カテゴリ追加 (名前, 減衰率)
    RenameCategory(String, String),    // カテゴリ名変更 (旧名, 新名)
    AddItem(String, String, String),   // 項目追加実行 (カテゴリ名, 項目名, 減衰率)
    AddScore(String),                  // スコア追加 (スコア)
    ExecuteDeleteCategory(String),     // カテゴリ削除実行
    ExecuteDeleteItem(String, String), // 項目削除
    ExecuteDeleteScore(usize),         // スコア削除実行
    UpdateItem(String, String, String, String, String), // 項目更新
    UpdateDecayRate(String),           // 減衰率更新 (文字列)
}
