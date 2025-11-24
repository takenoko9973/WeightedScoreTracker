from PySide6.QtCore import Qt, Signal
from PySide6.QtWidgets import (
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QListWidget,
    QListWidgetItem,
    QPushButton,
    QVBoxLayout,
    QWidget,
)


class CategoryWidget(QWidget):
    """左側のカテゴリ管理パネル"""

    # イベント通知用シグナル
    category_selected = Signal(str)  # カテゴリが選ばれた (名前 or 空文字)
    add_requested = Signal()  # 追加ボタンが押された
    delete_requested = Signal()  # 削除ボタンが押された

    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.addWidget(QLabel("【項目一覧】"))

        self.list_widget = QListWidget()
        self.list_widget.currentItemChanged.connect(self._on_change)
        layout.addWidget(self.list_widget)

        self.btn_add = QPushButton("項目追加")
        self.btn_add.clicked.connect(self.add_requested.emit)
        layout.addWidget(self.btn_add)

        self.btn_del = QPushButton("項目削除")
        self.btn_del.clicked.connect(self.delete_requested.emit)
        layout.addWidget(self.btn_del)

    def update_list(self, categories: list[str], current: str | None):
        """リストの中身を更新する"""
        self.list_widget.blockSignals(True)  # 無駄なシグナル発火を防ぐ
        self.list_widget.clear()
        for cat in categories:
            self.list_widget.addItem(cat)

        if current:
            items = self.list_widget.findItems(current, Qt.MatchFlag.MatchExactly)
            if items:
                self.list_widget.setCurrentItem(items[0])
            else:
                # 指定されたカテゴリが見つからない場合は選択解除
                self.list_widget.clearSelection()
                self.list_widget.setCurrentRow(-1)
        else:
            # 現在の選択がない場合は選択解除
            self.list_widget.clearSelection()
            self.list_widget.setCurrentRow(-1)

        self.list_widget.blockSignals(False)

    def _on_change(
        self, current: QListWidgetItem | None, previous: QListWidgetItem | None
    ):
        text = current.text() if current else ""
        self.category_selected.emit(text)

    def get_current_text(self) -> str | None:
        item = self.list_widget.currentItem()
        return item.text() if item else None


class ControlHeaderWidget(QWidget):
    """右上の統計情報と設定ボタン"""

    edit_decay_requested = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        self.stats_label = QLabel("項目を選択してください")
        self.stats_label.setStyleSheet(
            "font-size: 16px; font-weight: bold; color: #333;"
        )
        layout.addWidget(self.stats_label)

        layout.addStretch()

        self.decay_label = QLabel("減衰率: -")
        layout.addWidget(self.decay_label)

        self.btn_edit = QPushButton("設定変更")
        self.btn_edit.setFixedWidth(80)
        self.btn_edit.clicked.connect(self.edit_decay_requested.emit)
        self.btn_edit.setEnabled(False)
        layout.addWidget(self.btn_edit)

    def update_info(self, avg: float, count: int, decay: float, has_selection: bool):
        if not has_selection:
            self.stats_label.setText("項目を選択してください")
            self.decay_label.setText("減衰率: -")
            self.btn_edit.setEnabled(False)
        else:
            self.stats_label.setText(f"現在の加重平均: {avg:.2f} (データ数: {count})")
            self.decay_label.setText(f"減衰率: {decay}")
            self.btn_edit.setEnabled(True)


class InputWidget(QWidget):
    """右下のスコア入力エリア"""

    score_added = Signal(str)  # 入力されたテキストを送信

    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.addWidget(QLabel("【スコア入力】"))

        self.input_field = QLineEdit()
        self.input_field.setPlaceholderText("スコア (整数)")
        self.input_field.returnPressed.connect(self._on_submit)
        self.input_field.setFixedHeight(40)
        self.input_field.setStyleSheet("font-size: 14px;")
        layout.addWidget(self.input_field)

        self.btn_submit = QPushButton("記録を追加")
        self.btn_submit.setFixedHeight(40)
        self.btn_submit.setStyleSheet("font-weight: bold; background-color: #e1f5fe;")
        self.btn_submit.clicked.connect(self._on_submit)
        layout.addWidget(self.btn_submit)
        layout.addStretch()

    def _on_submit(self):
        text = self.input_field.text()
        if text:
            self.score_added.emit(text)
            self.input_field.clear()

    def set_enabled(self, enabled: bool):
        self.input_field.setEnabled(enabled)
        self.btn_submit.setEnabled(enabled)


class HistoryWidget(QWidget):
    """右下の履歴リストエリア"""

    delete_requested = Signal(int)  # 削除したい行番号(0始まり)を送信

    def __init__(self, parent=None):
        super().__init__(parent)
        layout = QVBoxLayout(self)
        layout.addWidget(QLabel("【履歴 (上が最新)】"))

        self.list_widget = QListWidget()
        self.list_widget.setSelectionMode(QListWidget.SelectionMode.SingleSelection)
        layout.addWidget(self.list_widget)

        self.btn_del = QPushButton("選択した履歴を削除")
        self.btn_del.clicked.connect(self._on_delete)
        layout.addWidget(self.btn_del)

    def update_history(self, scores: list[int]):
        self.list_widget.clear()
        # 新しい順に表示
        total = len(scores)
        for i, score in enumerate(reversed(scores)):
            original_idx = total - i
            self.list_widget.addItem(f"{original_idx}回目:  {score}")

    def _on_delete(self):
        items = self.list_widget.selectedItems()
        if items:
            row = self.list_widget.row(items[0])
            self.delete_requested.emit(row)
