from matplotlib.backends.backend_qtagg import FigureCanvasQTAgg
from matplotlib.figure import Figure
from matplotlib.ticker import MaxNLocator, ScalarFormatter

from .config import COLOR_AVG_LINE, COLOR_BAR, COLOR_BAR_EDGE


class MplCanvas(FigureCanvasQTAgg):
    def __init__(
        self, parent=None, width: float = 5, height: float = 4, dpi: int = 100
    ):
        self.fig = Figure(figsize=(width, height), dpi=dpi)
        self.axes = self.fig.add_subplot(111)
        self.fig.subplots_adjust(bottom=0.2)
        super().__init__(self.fig)

    def update_plot(
        self,
        category_name: str | None,
        avg: float,
        scores: list[int],
        weights: list[float],
        decay_rate: float,
    ) -> None:
        self.axes.clear()

        if not scores:
            self.draw()
            return

        x_positions = [0.0]
        for w in weights[:-1]:
            x_positions.append(x_positions[-1] + w)
        total_width = sum(weights)

        # Configの色を使用
        self.axes.bar(
            x_positions,
            scores,
            width=weights,
            align="edge",
            color=COLOR_BAR,
            alpha=0.7,
            edgecolor=COLOR_BAR_EDGE,
        )

        self.axes.plot(
            [0, total_width],
            [avg, avg],
            color=COLOR_AVG_LINE,
            linestyle="--",
            linewidth=2,
            label=f"加重平均 ({avg:.2f})",
        )

        title_text = f"{category_name} の推移" if category_name else "データなし"
        self.axes.set_title(f"{title_text} (重み={decay_rate})", fontsize=12)
        self.axes.set_ylabel("スコア", fontsize=10)
        self.axes.set_xlabel("回数 (右端が最新)", fontsize=10)
        self.axes.set_xlim(0, total_width)

        tick_positions = [x + w / 2 for x, w in zip(x_positions, weights)]
        tick_labels = [str(i + 1) for i in range(len(scores))]

        n = len(scores)
        if n > 30:
            step = n // 15
            visible_indices = set(range(0, n, step))
            visible_indices.add(n - 1)

            final_positions = []
            final_labels = []
            for i in sorted(list(visible_indices)):
                final_positions.append(tick_positions[i])
                final_labels.append(tick_labels[i])

            self.axes.set_xticks(final_positions)
            self.axes.set_xticklabels(final_labels, fontsize=8, rotation=90)
        else:
            self.axes.set_xticks(tick_positions)
            self.axes.set_xticklabels(tick_labels, fontsize=8, rotation=90)

        y_formatter = ScalarFormatter(useOffset=False, useMathText=False)
        y_formatter.set_scientific(False)
        self.axes.yaxis.set_major_formatter(y_formatter)
        self.axes.yaxis.set_major_locator(MaxNLocator(integer=True))

        self.axes.grid(True, axis="y", linestyle=":", alpha=0.6)
        self.axes.legend()

        self.fig.tight_layout()
        self.draw()

    def clear_plot(self) -> None:
        self.axes.clear()
        self.draw()
