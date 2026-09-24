use eframe::egui;

use crate::action::Action;

pub struct ScoreInput;

impl ScoreInput {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, input_text: &mut String) -> Option<Action> {
        let mut action = None;

        ui.vertical(|ui| {
            ui.label("スコア入力");
            ui.horizontal(|ui| {
                let res = ui.add(egui::TextEdit::singleline(input_text).return_key(None));

                let is_clicked_button = ui.button("追加").clicked();
                let is_enter = res.has_focus()
                    && ui.input(|input| {
                        input.events.iter().any(|event| {
                            matches!(
                                event,
                                egui::Event::Key {
                                    key: egui::Key::Enter,
                                    pressed: true,
                                    repeat: false,
                                    ..
                                }
                            )
                        })
                    });

                if (is_clicked_button || is_enter) && !input_text.is_empty() {
                    action = Some(Action::AddScore(input_text.clone()));
                }
            });
        });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_press(key: egui::Key) -> egui::Event {
        key_event(key, true, false)
    }

    fn key_repeat(key: egui::Key) -> egui::Event {
        key_event(key, true, true)
    }

    fn key_release(key: egui::Key) -> egui::Event {
        key_event(key, false, false)
    }

    fn key_event(key: egui::Key, pressed: bool, repeat: bool) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat,
            modifiers: egui::Modifiers::NONE,
        }
    }

    fn run_frame(
        ctx: &egui::Context,
        score_input: &mut ScoreInput,
        input_text: &mut String,
        events: Vec<egui::Event>,
    ) -> Option<Action> {
        let mut raw_input = egui::RawInput::default();
        raw_input.events = events;

        let mut action = None;
        let output = ctx.run_ui(raw_input, |ui| {
            action = score_input.show(ui, input_text);
        });
        output.drop_without_applying_deltas();
        action
    }

    #[test]
    fn enter_adds_score_once_and_keeps_input_focus() {
        let ctx = egui::Context::default();
        let mut score_input = ScoreInput::new();
        let mut input_text = String::new();

        assert!(run_frame(&ctx, &mut score_input, &mut input_text, vec![]).is_none());
        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_press(egui::Key::Tab)]
            )
            .is_none()
        );
        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![egui::Event::Text("100".to_string())]
            )
            .is_none()
        );
        assert_eq!(input_text, "100");
        let focused_input = ctx.memory(|memory| memory.focused());
        assert!(focused_input.is_some());

        assert!(matches!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_press(egui::Key::Enter)]
            ),
            Some(Action::AddScore(score)) if score == "100"
        ));
        assert_eq!(ctx.memory(|memory| memory.focused()), focused_input);

        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_repeat(egui::Key::Enter)]
            )
            .is_none()
        );
        assert_eq!(ctx.memory(|memory| memory.focused()), focused_input);
        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_release(egui::Key::Enter)]
            )
            .is_none()
        );

        input_text.clear();
        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![egui::Event::Text("200".to_string())]
            )
            .is_none()
        );
        assert_eq!(input_text, "200");
        assert!(matches!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_press(egui::Key::Enter)]
            ),
            Some(Action::AddScore(score)) if score == "200"
        ));
        assert_eq!(ctx.memory(|memory| memory.focused()), focused_input);
        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_release(egui::Key::Enter)]
            )
            .is_none()
        );

        input_text.clear();
        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_press(egui::Key::Tab)]
            )
            .is_none()
        );
        let focused_other = ctx.memory(|memory| memory.focused());
        assert!(focused_other.is_some());
        assert_ne!(focused_other, focused_input);
        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_press(egui::Key::Enter)]
            )
            .is_none()
        );
        assert_eq!(ctx.memory(|memory| memory.focused()), focused_other);
    }

    #[test]
    fn enter_with_empty_input_does_not_add_score_or_change_focus() {
        let ctx = egui::Context::default();
        let mut score_input = ScoreInput::new();
        let mut input_text = String::new();

        assert!(run_frame(&ctx, &mut score_input, &mut input_text, vec![]).is_none());
        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_press(egui::Key::Tab)]
            )
            .is_none()
        );
        let focused_input = ctx.memory(|memory| memory.focused());
        assert!(focused_input.is_some());

        assert!(
            run_frame(
                &ctx,
                &mut score_input,
                &mut input_text,
                vec![key_press(egui::Key::Enter)]
            )
            .is_none()
        );
        assert_eq!(ctx.memory(|memory| memory.focused()), focused_input);
    }
}
