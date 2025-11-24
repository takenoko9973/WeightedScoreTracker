import json
import os
from dataclasses import asdict, dataclass, field

from .config import DATA_FILENAME, DEFAULT_DECAY_RATE


@dataclass
class CategoryData:
    """1つのカテゴリ（項目）のデータを表現するデータクラス"""

    scores: list[int] = field(default_factory=list)
    decay_rate: float = DEFAULT_DECAY_RATE


class DataManager:
    """データの永続化とデータクラスの管理を担当するクラス"""

    def __init__(self, filename: str = DATA_FILENAME):
        self.filename: str = filename
        # 辞書の値として dataclass を保持する
        self.data: dict[str, CategoryData] = self.load_data()

    def load_data(self) -> dict[str, CategoryData]:
        """JSONを読み込み、CategoryDataオブジェクトの辞書に変換する"""
        if not os.path.exists(self.filename):
            return {}

        try:
            with open(self.filename, "r", encoding="utf-8") as f:
                raw_data = json.load(f)

            result: dict[str, CategoryData] = {}
            should_save = False

            for key, val in raw_data.items():
                # マイグレーション: 古いリスト形式 ([10, 20]) の場合
                if isinstance(val, list):
                    result[key] = CategoryData(
                        scores=val, decay_rate=DEFAULT_DECAY_RATE
                    )
                    should_save = True

                # 通常形式 ({"scores": [...], "decay_rate": ...}) の場合
                elif isinstance(val, dict):
                    # 安全に値を取り出し、 dataclass に変換
                    scores = val.get("scores", [])
                    decay = val.get("decay_rate", DEFAULT_DECAY_RATE)
                    result[key] = CategoryData(scores=scores, decay_rate=decay)

            if should_save:
                print("旧データ形式を変換しました。")
                self.save_data_direct(result)

            return result

        except (json.JSONDecodeError, IOError):
            return {}

    def save_data(self) -> None:
        self.save_data_direct(self.data)

    def save_data_direct(self, data_to_save: dict[str, CategoryData]) -> None:
        """CategoryDataの辞書を標準の辞書に変換してJSON保存"""
        try:
            # dataclass -> dict 変換 (json.dump用)
            json_ready_data = {k: asdict(v) for k, v in data_to_save.items()}

            with open(self.filename, "w", encoding="utf-8") as f:
                json.dump(json_ready_data, f, ensure_ascii=False, indent=4)
        except IOError as e:
            print(f"Save Error: {e}")

    def add_category(
        self, category_name: str, decay_rate: float = DEFAULT_DECAY_RATE
    ) -> bool:
        if category_name not in self.data:
            # 新しいデータクラスのインスタンスを作成
            self.data[category_name] = CategoryData(scores=[], decay_rate=decay_rate)
            self.save_data()
            return True
        return False

    def update_decay_rate(self, category: str, new_rate: float) -> None:
        if category in self.data:
            # ドット記法でアクセス可能
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
        """指定カテゴリのスコアリストを取得"""
        if category and category in self.data:
            return self.data[category].scores
        return []
