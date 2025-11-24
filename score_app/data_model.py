import json
import logging
import os
from dataclasses import asdict, dataclass, field

from .config import DATA_FILENAME, DEFAULT_DECAY_RATE

# ロガーの設定
logger = logging.getLogger(__name__)
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(levelname)s - %(message)s"
)


@dataclass
class CategoryData:
    scores: list[int] = field(default_factory=list)
    decay_rate: float = DEFAULT_DECAY_RATE


class DataManager:
    """データの永続化とデータクラスの管理を担当するクラス"""

    def __init__(self, filename: str = DATA_FILENAME):
        self.filename: str = filename
        self.data: dict[str, CategoryData] = self.load_data()

    def load_data(self) -> dict[str, CategoryData]:
        if not os.path.exists(self.filename):
            logger.info(f"データファイルが見つかりません: {self.filename}")
            return {}

        try:
            with open(self.filename, "r", encoding="utf-8") as f:
                raw_data = json.load(f)

            result: dict[str, CategoryData] = {}
            should_save = False

            for key, val in raw_data.items():
                if isinstance(val, list):
                    result[key] = CategoryData(
                        scores=val, decay_rate=DEFAULT_DECAY_RATE
                    )
                    should_save = True
                elif isinstance(val, dict):
                    scores = val.get("scores", [])
                    decay = val.get("decay_rate", DEFAULT_DECAY_RATE)
                    result[key] = CategoryData(scores=scores, decay_rate=decay)

            if should_save:
                logger.info("旧データ形式を変換しました。")
                self.save_data_direct(result)

            logger.info(f"データを読み込みました: {len(result)}件のカテゴリ")
            return result

        except (json.JSONDecodeError, IOError) as e:
            logger.error(f"データ読み込みエラー: {e}")
            return {}

    def save_data(self) -> None:
        self.save_data_direct(self.data)

    def save_data_direct(self, data_to_save: dict[str, CategoryData]) -> None:
        try:
            json_ready_data = {k: asdict(v) for k, v in data_to_save.items()}
            with open(self.filename, "w", encoding="utf-8") as f:
                json.dump(json_ready_data, f, ensure_ascii=False, indent=4)
        except IOError as e:
            logger.error(f"データ保存エラー: {e}")

    # --- 以下、前回のメソッド定義と同じため省略しませんが、変更はありません ---
    def add_category(
        self, category_name: str, decay_rate: float = DEFAULT_DECAY_RATE
    ) -> bool:
        if category_name not in self.data:
            self.data[category_name] = CategoryData(scores=[], decay_rate=decay_rate)
            self.save_data()
            return True
        return False

    def update_decay_rate(self, category: str, new_rate: float) -> None:
        if category in self.data:
            self.data[category].decay_rate = float(new_rate)
            self.save_data()

    def get_decay_rate(self, category: str) -> float:
        if category in self.data:
            return self.data[category].decay_rate
        return DEFAULT_DECAY_RATE

    def add_score(self, category: str, score: int) -> None:
        if category in self.data:
            self.data[category].scores.append(int(score))
            self.save_data()

    def delete_score_at(self, category: str, index: int) -> None:
        if category in self.data:
            scores = self.data[category].scores
            if 0 <= index < len(scores):
                del scores[index]
                self.save_data()

    def delete_category(self, category: str) -> None:
        if category in self.data:
            del self.data[category]
            self.save_data()

    def get_scores(self, category: str | None) -> list[int]:
        if category and category in self.data:
            return self.data[category].scores
        return []
