use std::iter::zip;

use crate::logic::{calculate_plot_params, calculate_stats};
use crate::models::{AppData, ItemData, ScoreEntry};
use crate::ui::Action;
use crate::ui::state::UiState;
use eframe::egui::{self};
use egui_plot::{Bar, BarChart, Corner, Legend, Plot};

pub fn draw(ctx: &egui::Context, data: &AppData, state: &mut UiState) -> Option<Action> {
    egui::CentralPanel::default()
        .show(ctx, |ui| {
            // カテゴリ未選択
            let (Some(cat_name), Some(item_name)) = (
                &state.selection.current_category,
                &state.selection.current_item,
            ) else {
                ui.centered_and_justified(|ui| {
                    ui.label("左のリストから項目を選択するか、追加してください");
                });
                return None;
            };

            // データ取得: カテゴリ -> 項目
            let Some(cat_data) = data.categories.get(cat_name) else {
                ui.label("カテゴリデータ読み込みエラー");
                return None;
            };
            let Some(item_data) = cat_data.items.get(item_name) else {
                ui.label("項目データ読み込みエラー");
                return None;
            };

            // ===========================================

            // ヘッダー
            let header_action = draw_header(ui, item_data);
            ui.separator();

            // グラフ
            draw_graph(ui, item_data, state);
            ui.add_space(10.0);

            // 入力と履歴
            let (input_action, history_action) = ui.columns(2, |columns| {
                (
                    // 左カラム: 入力
                    draw_input_section(&mut columns[0], state),
                    // 右カラム: 履歴
                    draw_history_section(&mut columns[1], item_data, state),
                )
            });

            header_action.or(input_action).or(history_action)
        })
        .inner
}

/// ヘッダー（統計情報と設定ボタン）の描画
fn draw_header(ui: &mut egui::Ui, item_data: &ItemData) -> Option<Action> {
    let (avg, count, _) = calculate_stats(&item_data.scores, item_data.decay_rate);
    let mut action = None;

    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("現在の加重平均: {:.2}", avg))
                .size(16.0)
                .strong(),
        );
        ui.label(format!("(データ数: {})", count));

        // 右寄せ配置 (右から左に順番に設置)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("設定変更").clicked() {
                action = Some(Action::ShowEditDecayModal);
            }
            ui.label(format!("減衰率: {:.2}", item_data.decay_rate));
        });
    });

    action
}

/// グラフの描画
fn draw_graph(ui: &mut egui::Ui, item_data: &ItemData, state: &mut UiState) {
    let (avg, _, weights) = calculate_stats(&item_data.scores, item_data.decay_rate);
    let base_color = egui::Color32::from_rgb(65, 105, 225);

    let params = calculate_plot_params(&item_data.scores, &weights);
    let bar_base = params.bar_base;

    let mut boundaries = Vec::new(); // クリック判定用のバー範囲記録
    let mut current_x = 0.0; // 棒グラフの合計横幅記録用

    let bars = zip(item_data.scores.iter(), weights.iter())
        .enumerate()
        .map(|(i, (entry, &weight))| {
            let width = weight; // 重みがそのまま横幅となる

            let center_x = current_x + (width / 2.0);
            let bar = Bar::new(center_x, (entry.score as f64 - bar_base).max(0.0))
                .base_offset(bar_base)
                .width(width)
                .name(format!("{}回目", i + 1));

            let is_selected = state.selection.selected_history_index == Some(i);
            let fill_color = if is_selected {
                base_color // 選択時は濃く
            } else {
                base_color.gamma_multiply(0.4) // 通常は薄く
            };

            boundaries.push(current_x + weight);
            current_x += width;

            bar.fill(fill_color)
                .stroke(egui::Stroke::new(1.0, base_color))
        })
        .collect();

    let total_width = current_x;

    // 棒グラフデータ作成
    let plot_height = ui.available_height() * 0.6; // 画面の6割を使う
    let plot = Plot::new("score_plot")
        .height(plot_height)
        .legend(Legend::default().position(Corner::RightBottom))
        .x_axis_formatter(|_, _| String::new())
        .show_x(false)
        .allow_axis_zoom_drag(false)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false);

    let plot_response = plot.show(ui, |plot_ui| {
        // 棒グラフの描画
        plot_ui.bar_chart(
            BarChart::new("スコア", bars)
                .color(base_color)
                .highlight(false)
                .element_formatter(Box::new(|bar, _chart| {
                    format!("{}\nスコア: {:.1}", bar.name, bar.value)
                })),
        );

        // 平均線の描画
        let avg_line_data = vec![[0.0, avg], [total_width, avg]];
        plot_ui.line(
            egui_plot::Line::new("荷重平均", egui_plot::PlotPoints::new(avg_line_data))
                .color(egui::Color32::ORANGE)
                .style(egui_plot::LineStyle::Dashed { length: 10.0 })
                .highlight(false)
                .allow_hover(false),
        );

        // --- クリック検出ロジック ---
        // グラフがクリックされたか？
        if plot_ui.response().clicked() {
            // マウスカーソルの座標（Plot空間）を取得
            if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                // X座標がどのバーの範囲内にあるか探す
                // boundaries[i] は i番目のバーの「右端」の座標
                if pointer_pos.x >= 0.0 && pointer_pos.x <= total_width {
                    let clicked_index = boundaries.iter().position(|&end_x| end_x > pointer_pos.x);
                    if let Some(idx) = clicked_index {
                        return Some(idx); // クリックされたインデックスを返す
                    }
                }
            }
            // 範囲外クリックなら選択解除（Noneを返す）
            return Some(usize::MAX); // 特殊値: 解除用
        }
        None
    });

    // InnerResponse経由でクリック結果を受け取る
    if let Some(idx) = plot_response.inner {
        if idx == usize::MAX {
            state.selection.selected_history_index = None;
        } else {
            state.selection.selected_history_index = Some(idx);
        }
    }
}

/// 入力カラムの描画
fn draw_input_section(ui: &mut egui::Ui, state: &mut UiState) -> Option<Action> {
    let mut action = None;

    ui.vertical(|ui| {
        ui.label("【スコア入力】");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            // 記録ボタン
            let is_clicked = ui.button("記録").clicked();

            // 入力欄
            let response = ui.add(
                egui::TextEdit::singleline(&mut state.selection.input_score)
                    .desired_width(f32::INFINITY),
            );
            // 入力欄でのenter入力
            let is_enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if is_enter || is_clicked {
                action = Some(Action::AddScore(state.selection.input_score.clone()));
                if is_enter {
                    response.request_focus();
                }
            }
        });
    });

    action
}

// 履歴カラムの描画
fn draw_history_section(
    ui: &mut egui::Ui,
    item_data: &ItemData,
    state: &mut UiState,
) -> Option<Action> {
    let mut action = None;

    ui.vertical(|ui| {
        ui.label("【履歴】");

        egui::ScrollArea::vertical()
            .id_salt("history")
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                let total = item_data.scores.len();
                action = item_data
                    .scores
                    .iter()
                    .rev()
                    .enumerate()
                    .filter_map(|(i, entry)| draw_history_row(ui, entry, i, total, state))
                    .last();
            });
    });

    action
}

fn draw_history_row(
    ui: &mut egui::Ui,
    entry: &ScoreEntry,
    rev_index: usize,
    total: usize,
    state: &mut UiState,
) -> Option<Action> {
    let mut action = None;
    let original_idx = total - 1 - rev_index;

    ui.horizontal(|ui| {
        // 削除ボタン
        if ui.button("🗑").clicked() {
            action = Some(Action::ShowDeleteScoreConfirm(original_idx));
        }

        // ラベル作成
        let local_time = entry.timestamp.with_timezone(&chrono::Local);
        let time_str = local_time.format("%Y-%m-%d %H:%M").to_string();
        let label_text = format!("[{}] {}回目: {}", time_str, original_idx + 1, entry.score);

        // 選択可能ラベルの描画
        let is_selected = state.selection.selected_history_index == Some(original_idx);
        let response = ui.selectable_label(is_selected, label_text);

        // クリック時の処理 (State更新)
        if response.clicked() {
            state.selection.selected_history_index = Some(original_idx);
        }

        // 自動スクロール
        if is_selected {
            response.scroll_to_me(Some(egui::Align::Center));
        }
    });

    action
}
