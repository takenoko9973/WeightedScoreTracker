/// モーダルの定義
#[derive(Default, Clone, Debug)]
pub enum ModalType {
    #[default]
    None,
    // カテゴリ追加画面の状態
    AddCategory {
        input_name: String,
    },
    // カテゴリ名変更画面の状態
    RenameCategory {
        target: String,
        input_new_name: String,
    },
    // 項目追加画面の状態
    AddItem {
        target_category: String,
        input_name: String,
        input_decay: String,
    },
    // 項目編集
    EditItem {
        target_cat: String,  // 元のカテゴリ
        target_item: String, // 元の項目名
        input_cat: String,   // カテゴリ選択用（移動先）
        input_item: String,  // 名前入力用
        input_decay: String, // 減衰率入力用
    },
    // 減衰率変更画面の状態
    EditDecay {
        input_decay: String,
    },
    // 削除確認ダイアログ
    ConfirmDeleteCategory {
        target: String,
    },
    ConfirmDeleteItem {
        target_cat: String,
        target_item: String,
    },
    ConfirmDeleteScore {
        index: usize,
    },
}
