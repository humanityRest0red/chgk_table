use eframe::egui;
use crate::models::chgk_game::Answer;
use super::theme::{CELL_HEIGHT, QUESTION_WIDTH, palette};


pub(super) fn header_cell(ui: &mut egui::Ui, text: &str, width: f32) {
    let p = palette(ui.visuals().dark_mode);
    let size = [width, CELL_HEIGHT];
    cell(ui, text, size, p.header, p.header_text);
}

pub(super) fn colored_cell(ui: &mut egui::Ui, text: &str, width: f32, fill_color: egui::Color32) {
    let size = [width, CELL_HEIGHT];
    bordered_cell(ui, |ui| {
        ui.add_sized(
            size,
            egui::Button::new(
                egui::RichText::new(text).strong().color(egui::Color32::WHITE),
            )
            .fill(fill_color),
        );
    });
}


pub(super) fn answer_cell(ui: &mut egui::Ui, answer: &mut Answer, locked: bool) {
    let p = palette(ui.visuals().dark_mode);

    let (label, background, foreground) = match answer {
        Answer::Plus => ("+", p.success, egui::Color32::WHITE),
        Answer::Minus => ("−", p.danger, egui::Color32::WHITE),
        // Answer::Plus => ("+", egui::Color32::WHITE, p.success),
        // Answer::Minus => ("−", egui::Color32::WHITE, p.danger),
        Answer::Question => ("?", p.question, egui::Color32::from_rgb(40, 40, 40)),
        Answer::Empty => ("", p.card, p.text),
    };

    // Для пустой ячейки делаем легкую обводку, чтобы видеть сетку
    let response = bordered_cell(ui, |ui| {
        ui.add_sized(
            [QUESTION_WIDTH, CELL_HEIGHT],
            egui::Button::new(
                egui::RichText::new(label)
                .strong()
                .color(foreground)
                .size(25.0)
            )
                .fill(background)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(4))
        )
    });

    let response = if locked {
        response
            .on_hover_cursor(egui::CursorIcon::NotAllowed)
            .on_hover_text("Ввод результатов заблокирован")
    } else {
        response
    };

    if response.clicked() && !locked {
        *answer = match answer {
            Answer::Empty => Answer::Plus,
            Answer::Plus => Answer::Minus,
            Answer::Minus => Answer::Question,
            Answer::Question => Answer::Empty,
        };
    }
}

fn cell(
    ui: &mut egui::Ui,
    text: &str,
    size: [f32; 2],
    fill_color: egui::Color32,
    text_color: egui::Color32,
) {
    bordered_cell(ui, |ui| {
        ui.add_sized(
            size,
            egui::Button::new(egui::RichText::new(text).strong().color(text_color))
                .fill(fill_color),
        );
    });
}

fn bordered_cell<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let p = palette(ui.visuals().dark_mode);
    egui::Frame::new()
        .fill(p.card) // Фон ячейки всегда цвет карточки
        .stroke(egui::Stroke::new(1.0, p.border)) // Тонкая граница
        .corner_radius(egui::CornerRadius::same(4))
        .show(ui, add_contents)
        .inner
}
