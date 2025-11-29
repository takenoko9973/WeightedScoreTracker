from score_app.data_model import ScoreEntry


class ScoreCalculator:
    """スコアの統計計算を行う純粋なロジッククラス"""

    @staticmethod
    def calculate_stats(
        score_entries: list[ScoreEntry], decay_rate: float
    ) -> tuple[float, list[float]]:
        """
        加重平均と、各スコアに対応する重みのリストを計算して返す

        Args:
            scores: スコアのリスト（古い順）
            decay_rate: 減衰率 (0 < r <= 1)

        Returns:
            (加重平均, 重みのリスト)
        """
        scores = [entry.score for entry in score_entries]

        if not scores:
            return 0.0, []

        n = len(scores)
        weights: list[float] = []

        # 重みの計算: 古い順(index=0)から新しい順(index=n-1)への重み付け
        for i in range(n):
            exponent = (n - 1) - i
            weights.append(decay_rate**exponent)

        # 加重平均の計算
        weighted_sum = sum(s * w for s, w in zip(scores, weights))
        total_weight = sum(weights)
        weighted_avg = weighted_sum / total_weight if total_weight > 0 else 0.0

        return weighted_avg, weights
