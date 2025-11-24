from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QHBoxLayout,
    QInputDialog,
    QLabel,
    QLineEdit,
    QListWidget,
    QListWidgetItem,
    QMainWindow,
    QMessageBox,
    QPushButton,
    QSplitter,
    QVBoxLayout,
    QWidget,
)

from .calculator import ScoreCalculator
from .config import DEFAULT_DECAY_RATE, MAX_DECAY, MIN_DECAY, WINDOW_SIZE, WINDOW_TITLE
from .data_model import DataManager
from .plot_widget import MplCanvas


class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle(WINDOW_TITLE)
        self.resize(*WINDOW_SIZE)

        self.manager = DataManager()
        self.current_category: str | None = None

        self._setup_ui()
        self._refresh_category_list()

    def _setup_ui(self):
        main_widget = QWidget()
        self.setCentralWidget(main_widget)
        main_layout = QHBoxLayout(main_widget)

        # Left Panel
        left_panel = QWidget()
        left_layout = QVBoxLayout(left_panel)
        left_layout.addWidget(QLabel("【項目一覧】"))
        self.category_list = QListWidget()
        self.category_list.currentItemChanged.connect(self.on_category_selected)
        left_layout.addWidget(self.category_list)

        btn_add_cat = QPushButton("項目追加")
        btn_add_cat.clicked.connect(self.add_category_dialog)
        left_layout.addWidget(btn_add_cat)

        btn_del_cat = QPushButton("項目削除")
        btn_del_cat.clicked.connect(self.delete_category_action)
        left_layout.addWidget(btn_del_cat)

        # Right Panel
        right_panel = QWidget()
        right_layout = QVBoxLayout(right_panel)

        # Header
        header_layout = QHBoxLayout()
        self.stats_label = QLabel("項目を選択してください")
        self.stats_label.setStyleSheet(
            "font-size: 16px; font-weight: bold; color: #333;"
        )
        header_layout.addWidget(self.stats_label)
        header_layout.addStretch()
        self.decay_label = QLabel("減衰率: -")
        header_layout.addWidget(self.decay_label)
        self.btn_edit_decay = QPushButton("設定変更")
        self.btn_edit_decay.setFixedWidth(80)
        self.btn_edit_decay.clicked.connect(self.edit_decay_rate)
        self.btn_edit_decay.setEnabled(False)
        header_layout.addWidget(self.btn_edit_decay)
        right_layout.addLayout(header_layout)

        # Canvas
        self.canvas = MplCanvas(self, width=5, height=4, dpi=100)
        right_layout.addWidget(self.canvas, stretch=2)

        # Bottom Area
        bottom_container = QWidget()
        bottom_layout = QHBoxLayout(bottom_container)
        bottom_layout.setContentsMargins(0, 0, 0, 0)

        # Input
        input_group = QWidget()
        input_v_layout = QVBoxLayout(input_group)
        input_v_layout.addWidget(QLabel("【スコア入力】"))
        self.score_input = QLineEdit()
        self.score_input.setPlaceholderText("スコア (整数)")
        self.score_input.returnPressed.connect(self.add_score_action)
        self.score_input.setFixedHeight(40)
        self.score_input.setStyleSheet("font-size: 14px;")
        input_v_layout.addWidget(self.score_input)

        btn_add_score = QPushButton("記録を追加")
        btn_add_score.setFixedHeight(40)
        btn_add_score.setStyleSheet("font-weight: bold; background-color: #e1f5fe;")
        btn_add_score.clicked.connect(self.add_score_action)
        input_v_layout.addWidget(btn_add_score)
        input_v_layout.addStretch()

        # History
        history_group = QWidget()
        history_v_layout = QVBoxLayout(history_group)
        history_v_layout.addWidget(QLabel("【履歴 (上が最新)】"))
        self.history_list = QListWidget()
        self.history_list.setSelectionMode(QListWidget.SelectionMode.SingleSelection)
        history_v_layout.addWidget(self.history_list)
        btn_del_score = QPushButton("選択した履歴を削除")
        btn_del_score.clicked.connect(self.delete_score_action)
        history_v_layout.addWidget(btn_del_score)

        bottom_layout.addWidget(input_group, stretch=1)
        bottom_layout.addWidget(history_group, stretch=2)
        right_layout.addWidget(bottom_container, stretch=1)

        splitter = QSplitter(Qt.Orientation.Horizontal)
        splitter.addWidget(left_panel)
        splitter.addWidget(right_panel)
        splitter.setStretchFactor(1, 3)
        main_layout.addWidget(splitter)

    def _refresh_category_list(self):
        current = self.category_list.currentItem()
        current_text = current.text() if current else None

        self.category_list.clear()
        for cat in self.manager.data.keys():
            self.category_list.addItem(cat)

        if current_text:
            items = self.category_list.findItems(
                current_text, Qt.MatchFlag.MatchExactly
            )
            if items:
                self.category_list.setCurrentItem(items[0])

    def add_category_dialog(self):
        name, ok1 = QInputDialog.getText(self, "新規項目", "項目名を入力:")
        if not ok1 or not name:
            return

        rate, ok2 = QInputDialog.getDouble(
            self,
            "重み設定",
            f"'{name}' の減衰率 ({MIN_DECAY} - {MAX_DECAY}):",
            DEFAULT_DECAY_RATE,
            MIN_DECAY,
            MAX_DECAY,
            2,
        )
        if ok2:
            if self.manager.add_category(name, rate):
                self._refresh_category_list()
            else:
                QMessageBox.warning(self, "エラー", "その項目名は既に存在します。")

    def edit_decay_rate(self):
        if not self.current_category:
            return
        current_rate = self.manager.get_decay_rate(self.current_category)
        rate, ok = QInputDialog.getDouble(
            self,
            "重み設定変更",
            "減衰率の変更:",
            current_rate,
            MIN_DECAY,
            MAX_DECAY,
            2,
        )
        if ok:
            self.manager.update_decay_rate(self.current_category, rate)
            self.update_display()

    def delete_category_action(self):
        item = self.category_list.currentItem()
        if item:
            ret = QMessageBox.question(
                self,
                "確認",
                f"'{item.text()}' を削除しますか？",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
            )
            if ret == QMessageBox.StandardButton.Yes:
                self.manager.delete_category(item.text())
                self._refresh_category_list()
                self.on_category_selected(None, None)

    def on_category_selected(
        self, current: QListWidgetItem | None, previous: QListWidgetItem | None
    ):
        if not current:
            self.current_category = None
            self.stats_label.setText("項目を選択してください")
            self.decay_label.setText("減衰率: -")
            self.btn_edit_decay.setEnabled(False)
            self.score_input.setEnabled(False)
            self.canvas.clear_plot()
            self.history_list.clear()
            return

        self.current_category = current.text()
        self.btn_edit_decay.setEnabled(True)
        self.score_input.setEnabled(True)
        self.update_display()

    def add_score_action(self):
        if not self.current_category:
            return
        text = self.score_input.text()
        if not text:
            return
        try:
            score = int(text)
            self.manager.add_score(self.current_category, score)
            self.score_input.clear()
            self.update_display()
        except ValueError:
            QMessageBox.warning(self, "エラー", "整数を入力してください。")

    def delete_score_action(self):
        if not self.current_category:
            return
        selected_items = self.history_list.selectedItems()
        if not selected_items:
            return

        row = self.history_list.row(selected_items[0])
        scores = self.manager.get_scores(self.current_category)
        target_index = (len(scores) - 1) - row

        val = scores[target_index]
        ret = QMessageBox.question(
            self,
            "確認",
            f"履歴「{target_index + 1}回目: {val}」を削除しますか？",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
        )
        if ret == QMessageBox.StandardButton.Yes:
            self.manager.delete_score_at(self.current_category, target_index)
            self.update_display()

    def update_display(self):
        """画面表示の更新処理"""
        if not self.current_category:
            return

        scores = self.manager.get_scores(self.current_category)
        decay_rate = self.manager.get_decay_rate(self.current_category)

        avg, weights = ScoreCalculator.calculate_stats(scores, decay_rate)

        self.stats_label.setText(f"現在の加重平均: {avg:.2f} (データ数: {len(scores)})")
        self.decay_label.setText(f"減衰率: {decay_rate}")

        self.history_list.clear()
        for i, score in enumerate(reversed(scores)):
            original_idx = len(scores) - i
            self.history_list.addItem(f"{original_idx}回目:  {score}")

        self.canvas.update_plot(self.current_category, avg, scores, weights, decay_rate)
