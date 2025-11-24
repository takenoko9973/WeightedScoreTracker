import json
import os


class DataManager:
    """データの永続化、データ構造の管理、統計計算を行うモデルクラス"""

    def __init__(self, filename="scores_data.json"):
        self.filename = filename
        self.default_decay = 0.95
        self.data = self.load_data()

    def load_data(self):
        """データを読み込み、必要に応じて新形式へマイグレーションする"""
        if not os.path.exists(self.filename):
            return {}

        try:
            with open(self.filename, "r", encoding="utf-8") as f:
                raw_data = json.load(f)

            # マイグレーション処理: リスト形式 -> 辞書形式
            migrated = False
            new_data = {}
            for key, val in raw_data.items():
                if isinstance(val, list):
                    new_data[key] = {"scores": val, "decay_rate": self.default_decay}
                    migrated = True
                else:
                    new_data[key] = val

            if migrated:
                print("旧データ形式を検知しました。新形式に変換して保存します。")
                self.save_data_direct(new_data)

            return new_data
        except (json.JSONDecodeError, IOError):
            return {}

    def save_data(self):
        self.save_data_direct(self.data)

    def save_data_direct(self, data_to_save):
        try:
            with open(self.filename, "w", encoding="utf-8") as f:
                json.dump(data_to_save, f, ensure_ascii=False, indent=4)
        except IOError as e:
            print(f"Save Error: {e}")

    def add_category(self, category_name, decay_rate=0.95):
        if category_name not in self.data:
            self.data[category_name] = {"scores": [], "decay_rate": decay_rate}
            self.save_data()
            return True
        return False

    def update_decay_rate(self, category, new_rate):
        if category in self.data:
            self.data[category]["decay_rate"] = float(new_rate)
            self.save_data()

    def get_decay_rate(self, category):
        if category in self.data:
            return self.data[category].get("decay_rate", self.default_decay)
        return self.default_decay

    def add_score(self, category, score):
        if category in self.data:
            self.data[category]["scores"].append(int(score))
            self.save_data()

    def delete_score_at(self, category, index):
        """指定したインデックス(0始まり)のスコアを削除"""
        if category in self.data:
            scores = self.data[category]["scores"]
            if 0 <= index < len(scores):
                del scores[index]
                self.save_data()

    def delete_category(self, category):
        if category in self.data:
            del self.data[category]
            self.save_data()

    def calculate_stats(self, category):
        """
        描画に必要な統計情報を計算して返す
        Return: (加重平均, スコアリスト, 重みリスト, 減衰率)
        """
        if category not in self.data:
            return 0.0, [], [], self.default_decay

        entry = self.data[category]
        scores = entry["scores"]
        decay_rate = entry.get("decay_rate", self.default_decay)

        if not scores:
            return 0.0, [], [], decay_rate

        n = len(scores)
        weights = []

        # 重みの計算: 古い順(index=0)から新しい順(index=n-1)への重み付け
        for i in range(n):
            exponent = (n - 1) - i
            weights.append(decay_rate**exponent)

        # 加重平均の計算
        weighted_sum = sum(s * w for s, w in zip(scores, weights))
        total_weight = sum(weights)
        weighted_avg = weighted_sum / total_weight if total_weight > 0 else 0

        return weighted_avg, scores, weights, decay_rate
