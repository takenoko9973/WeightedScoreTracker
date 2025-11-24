# ファイル設定
DATA_FILENAME: str = "scores_data.json"

# 計算設定
DEFAULT_DECAY_RATE: float = 0.95
MIN_DECAY: float = 0.01
MAX_DECAY: float = 1.00

# グラフ描画設定 (色やフォント)
COLOR_BAR: str = "royalblue"
COLOR_BAR_EDGE: str = "blue"
COLOR_AVG_LINE: str = "orange"
FONT_FAMILY: str = "sans-serif"
FONT_LIST: list[str] = [
    "Meiryo",
    "Yu Gothic",
    "Hiragino Sans",
    "Takao",
    "IPAexGothic",
    "IPAPGothic",
    "Noto Sans CJK JP",
]

# UI設定
WINDOW_TITLE: str = "Weighted Score Tracker (Modern Type Hints)"
WINDOW_SIZE: tuple[int, int] = (1100, 700)
