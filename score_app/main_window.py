from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QHBoxLayout,
    QInputDialog,
    QMainWindow,
    QMessageBox,
    QSplitter,
    QVBoxLayout,
    QWidget,
)

from .calculator import ScoreCalculator
from .components import CategoryWidget, ControlHeaderWidget, HistoryWidget, InputWidget
from .config import DEFAULT_DECAY_RATE, MAX_DECAY, MIN_DECAY, WINDOW_SIZE, WINDOW_TITLE
from .data_model import DataManager, ScoreEntry
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

        # 1. 左パネル (コンポーネント化)
        self.category_widget = CategoryWidget()
        # シグナル接続
        self.category_widget.category_selected.connect(self.on_category_selected)
        self.category_widget.add_requested.connect(self.add_category_dialog)
        self.category_widget.delete_requested.connect(self.delete_category_action)

        # 2. 右パネル
        right_panel = QWidget()
        right_layout = QVBoxLayout(right_panel)

        # 2-A. ヘッダー (コンポーネント化)
        self.header_widget = ControlHeaderWidget()
        self.header_widget.edit_decay_requested.connect(self.edit_decay_rate)
        right_layout.addWidget(self.header_widget)

        # 2-B. グラフ (既存)
        self.canvas = MplCanvas(self, width=5, height=4, dpi=100)
        right_layout.addWidget(self.canvas, stretch=2)

        # 2-C. 下部エリア (入力 + 履歴)
        bottom_container = QWidget()
        bottom_layout = QHBoxLayout(bottom_container)
        bottom_layout.setContentsMargins(0, 0, 0, 0)

        # 入力ウィジェット
        self.input_widget = InputWidget()
        self.input_widget.score_added.connect(self.add_score_action)
        self.input_widget.set_enabled(False)  # 初期状態は無効

        # 履歴ウィジェット
        self.history_widget = HistoryWidget()
        self.history_widget.delete_requested.connect(self.delete_score_action)

        bottom_layout.addWidget(self.input_widget, stretch=1)
        bottom_layout.addWidget(self.history_widget, stretch=2)
        right_layout.addWidget(bottom_container, stretch=1)

        # スプリッターで左右配置
        splitter = QSplitter(Qt.Orientation.Horizontal)
        splitter.addWidget(self.category_widget)
        splitter.addWidget(right_panel)
        splitter.setStretchFactor(1, 3)
        main_layout.addWidget(splitter)

    # --- Controller Logic ---

    def _refresh_category_list(self):
        # 現在の選択状態を維持しつつリスト更新
        cats = list(self.manager.data.keys())
        current = self.current_category
        self.category_widget.update_list(cats, current)

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
        # コンポーネントから現在選択されているテキストを取得
        name = self.category_widget.get_current_text()
        if name:
            ret = QMessageBox.question(
                self,
                "確認",
                f"'{name}' を削除しますか？",
                QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
            )
            if ret == QMessageBox.StandardButton.Yes:
                self.manager.delete_category(name)
                # 削除後は選択を解除
                self.current_category = None
                self._refresh_category_list()
                self.on_category_selected("")

    def on_category_selected(self, category_name: str):
        if not category_name:
            self.current_category = None
            self.header_widget.update_info(0, 0, 0, has_selection=False)
            self.input_widget.set_enabled(False)
            self.canvas.clear_plot()
            self.history_widget.update_history([])
            return

        self.current_category = category_name
        self.input_widget.set_enabled(True)
        self.update_display()

    def add_score_action(self, text: str):
        if not self.current_category:
            return

        try:
            score = int(text)

            if score < 0:
                QMessageBox.warning(
                    self, "エラー", "スコアにマイナスの値は入力できません。"
                )
                return

            self.manager.add_score(self.current_category, score)
            self.update_display()
        except ValueError:
            QMessageBox.warning(self, "エラー", "整数を入力してください。")

    def delete_score_action(self, row_index: int):
        if not self.current_category:
            return

        score_entries = self.manager.get_scores(self.current_category)
        # 表示は逆順なので、モデルのインデックスに変換
        target_index = (len(score_entries) - 1) - row_index

        val = score_entries[target_index].score
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
        """画面全体の更新"""
        if not self.current_category:
            return

        score_entries: list[ScoreEntry] = self.manager.get_scores(self.current_category)
        decay_rate = self.manager.get_decay_rate(self.current_category)

        # 計算
        avg, weights = ScoreCalculator.calculate_stats(score_entries, decay_rate)

        self.header_widget.update_info(avg, len(score_entries), decay_rate, has_selection=True)
        self.history_widget.update_history(score_entries)

        scores_for_plot = [entry.score for entry in score_entries]
        self.canvas.update_plot(self.current_category, avg, scores_for_plot, weights, decay_rate)
