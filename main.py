import sys

import matplotlib
from matplotlib import rcParams
from PySide6.QtWidgets import QApplication

from score_app import MainWindow

# Configからフォント設定をインポート
from score_app.config import FONT_FAMILY, FONT_LIST

# --- 初期設定 ---
rcParams["font.family"] = FONT_FAMILY
rcParams["font.sans-serif"] = FONT_LIST
matplotlib.use("QtAgg")


def main():
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
