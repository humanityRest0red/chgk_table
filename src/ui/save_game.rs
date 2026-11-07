use eframe::egui;
use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use serde_json;
use std::path::Path;

use egui::{Color32, Frame, Margin, Response, Ui};

const SAVE_TOAST: u32 = 1;

use super::chgk_app::ChgkApp;
use crate::models::chgk_game::Game;

impl ChgkApp {
    pub(super) fn save_game(&mut self) {
        let result = if let Some(game) = &self.game {
            let path = Path::new("data/chgk_save.json");

            match path.parent().map(std::fs::create_dir_all) {
                Some(Ok(())) | None => serde_json::to_string_pretty(game)
                    .map_err(|e| format!("Ошибка сериализации: {e}"))
                    .and_then(|json| {
                        std::fs::write(path, json).map_err(|e| format!("Ошибка сохранения: {e}"))
                    }),
                Some(Err(e)) => Err(format!("Ошибка создания папки: {e}")),
            }
        } else {
            Err("Нет игры для сохранения".to_string())
        };

        match result {
            Ok(()) => self.add_toast("Файл сохранён", ToastKind::Success),
            Err(error) => self.add_toast(error, ToastKind::Error),
        }
    }

    pub(super) fn load_game(&mut self, ctx: &egui::Context) {
        let path = Path::new("data/chgk_save.json");

        let result = std::fs::read_to_string(path)
            .map_err(|e| format!("Ошибка загрузки: {e}"))
            .and_then(|data| {
                serde_json::from_str::<Game>(&data).map_err(|e| format!("Ошибка чтения файла: {e}"))
            });

        match result {
            Ok(game) => {
                self.game = Some(game);
                self.add_toast("Файл загружен", ToastKind::Success);

                let size = self.game_window_size();
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
            Err(error) => self.add_toast(error, ToastKind::Error),
        }
    }

    fn add_toast(&mut self, text: impl Into<String>, kind: ToastKind) {
        self.toasts.add(Toast {
            text: eframe::egui::WidgetText::Text(text.into()),
            kind,
            options: ToastOptions::default()
                .duration_in_seconds(3.0)
                .show_progress(true),
            ..Default::default()
        });
    }
}

pub(crate) fn my_toasts() -> Toasts {
    Toasts::new()
        .custom_contents(SAVE_TOAST, save_toast_contents)
        .anchor(egui::Align2::RIGHT_BOTTOM, (-10.0, -10.0))
}

fn save_toast_contents(ui: &mut Ui, toast: &mut Toast) -> Response {
    Frame::new()
        .fill(Color32::from_rgb(45, 45, 55))
        .inner_margin(Margin::same(12))
        .corner_radius(6)
        .show(ui, |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            ui.label(toast.text.clone());
        })
        .response
}
