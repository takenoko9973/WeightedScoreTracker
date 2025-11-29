use std::iter::zip;

use crate::app::UiState;
use crate::logic::calculate_stats;
use crate::models::{AppData, CategoryData, ScoreEntry};
use chrono::{DateTime, Local};
use eframe::egui::{self};
use egui_plot::{Bar, BarChart, Legend, Plot};

pub fn draw(ctx: &egui::Context, data: &mut AppData, state: &mut UiState) -> bool {
    let mut save_needed = false;

    egui::CentralPanel::default().show(ctx, |ui| {
        // カテゴリ未選択
        let Some(cat_name) = &state.current_category else {
            ui.centered_and_justified(|ui| {
                ui.label("左のリストから項目を選択するか、追加してください");
            });
            return;
        };

        // データ取得エラー
        let Some(category_data) = data.categories.get_mut(cat_name) else {
            ui.label("データ読み込みエラー");
            return;
        };

        // ===========================================

        // ヘッダー
        draw_header(ui, category_data, state);
        ui.separator();

        // グラフ
        draw_graph(ui, category_data, state);
        ui.add_space(10.0);

        // 入力と履歴
        ui.columns(2, |columns| {
            // 左カラム: 入力
            if draw_input_column(&mut columns[0], category_data, state) {
                save_needed = true;
            }
            // 右カラム: 履歴
            draw_history_column(&mut columns[1], category_data, state)
        });
    });

    save_needed
}

/// ヘッダー（統計情報と設定ボタン）の描画
fn draw_header(ui: &mut egui::Ui, data: &CategoryData, state: &mut UiState) {
    ui.horizontal(|ui| {
        let (avg, count, _) = calculate_stats(&data.scores, data.decay_rate);

        ui.label(
            egui::RichText::new(format!("現在の加重平均: {:.2}", avg))
                .size(16.0)
                .strong(),
        );
        ui.label(format!("(データ数: {})", count));

        // 右寄せ配置 (右から左に順番に設置)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("設定変更").clicked() {
                state.input_decay = data.decay_rate.to_string();
                state.show_edit_decay_window = true;
            }
            ui.label(format!("減衰率: {:.2}", data.decay_rate));
        });
    });
}

/// グラフ（Plot）の描画
fn draw_graph(ui: &mut egui::Ui, data: &CategoryData, state: &mut UiState) {
    let (avg, _, weights) = calculate_stats(&data.scores, data.decay_rate);
    let base_color = egui::Color32::from_rgb(65, 105, 225);

    let mut boundaries = Vec::new(); // クリック判定用のバー範囲記録
    let mut current_x = 0.0; // 棒グラフの合計横幅記録用

    let bars = zip(data.scores.iter(), weights.iter())
        .enumerate()
        .map(|(i, (entry, &weight))| {
            let width = weight; // 重みがそのまま横幅となる

            let center_x = current_x + (width / 2.0);
            let bar = Bar::new(center_x, entry.score as f64)
                .width(width)
                .name(format!("{}回目", i + 1));

            let is_selected = state.selected_history_index == Some(i);
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
        .legend(Legend::default())
        .x_axis_formatter(|_, _| String::new())
        .show_x(false)
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
            state.selected_history_index = None;
        } else {
            state.selected_history_index = Some(idx);
        }
    }
}

/// 入力カラムの描画
fn draw_input_column(ui: &mut egui::Ui, data: &mut CategoryData, state: &mut UiState) -> bool {
    let mut saved = false;

    ui.vertical(|ui| {
        ui.label("【スコア入力】");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            // 記録ボタン
            let is_clicked = ui.button("記録").clicked();

            // 入力欄
            let response = ui.add(
                egui::TextEdit::singleline(&mut state.input_score).desired_width(f32::INFINITY),
            );
            // 入力欄でのenter入力
            let is_enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if is_enter || is_clicked {
                let score_validation_result = match state.input_score.parse::<i32>() {
                    // 整数にならなかった場合
                    Err(_) => Err("有効な整数値を入力してください。".to_string()),
                    // 数字だが、負の数だった場合
                    Ok(score) if score < 0 => {
                        Err("スコアにマイナスの値は入力できません。".to_string())
                    }
                    // 正常な整数の場合
                    Ok(score) => Ok(score),
                };

                match score_validation_result {
                    Ok(score) => {
                        data.scores.push(ScoreEntry {
                            score,
                            timestamp: Local::now(),
                        });
                        state.input_score.clear();

                        if is_enter {
                            response.request_focus();
                        }
                        saved = true;
                    }
                    Err(msg) => state.error_message = Some(msg),
                }
            }
        });
    });

    saved
}

/// 履歴カラムの描画
fn draw_history_column(ui: &mut egui::Ui, data: &mut CategoryData, state: &mut UiState) {
    ui.vertical(|ui| {
        ui.label("【履歴】");

        egui::ScrollArea::vertical()
            .id_salt("history")
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                let total = data.scores.len();

                // 新しい順(rev)に表示
                for (i, entry) in data.scores.iter().rev().enumerate() {
                    let original_idx = total - 1 - i;

                    // 日時フォーマット
                    let local_time: DateTime<Local> = entry.timestamp;
                    let time_str = local_time.format("%Y-%m-%d %H:%M").to_string();

                    ui.horizontal(|ui| {
                        if ui.button("🗑").clicked() {
                            // 削除待ちのデータインデックスをセット
                            state.pending_delete_index = Some(original_idx);
                        }

                        let is_selected = state.selected_history_index == Some(original_idx);
                        let label_text =
                            format!("[{}] {}回目: {}", time_str, original_idx + 1, entry.score);

                        let response = ui.selectable_label(is_selected, label_text);
                        if response.clicked() {
                            // 履歴をクリックしても選択状態にする
                            state.selected_history_index = Some(original_idx);
                        }
                        if is_selected {
                            // 選択されたら自動スクロールで表示させる
                            response.scroll_to_me(Some(egui::Align::Center));
                        }
                    });
                }
            });
    });
}
