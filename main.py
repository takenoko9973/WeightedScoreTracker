import sys

import matplotlib
from matplotlib import rcParams
from PySide6.QtWidgets import QApplication

from score_app import MainWindow

# --- 初期設定 ---
# Matplotlibの日本語フォント設定
rcParams["font.family"] = "sans-serif"
rcParams["font.sans-serif"] = [
    "Meiryo",
    "Yu Gothic",
    "Hiragino Sans",
    "Takao",
    "IPAexGothic",
    "IPAPGothic",
    "Noto Sans CJK JP",
]

# バックエンド指定
matplotlib.use("QtAgg")


def main():
    app = QApplication(sys.argv)

    # メインウィンドウの作成と表示
    window = MainWindow()
    window.show()

    sys.exit(app.exec())


if __name__ == "__main__":
    main()
