use eframe::egui;

use crate::models::chgk_game::Answer;

use super::layout::{
    CELL_HEIGHT, HEADER_FILL_COLOR, HEADER_TEXT_COLOR, QUESTION_WIDTH,
};

pub(super) fn header_cell(ui: &mut egui::Ui, text: &str, width: f32) {
    let size = [width, CELL_HEIGHT];
    cell(ui, text, size, HEADER_FILL_COLOR, HEADER_TEXT_COLOR);
}

pub(super) fn colored_cell(ui: &mut egui::Ui, text: &str, width: f32, fill_color: egui::Color32) {
    let size = [width, CELL_HEIGHT];
    let text_color = egui::Color32::WHITE;
    cell(ui, text, size, fill_color, text_color);
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

pub(super) fn answer_cell(ui: &mut egui::Ui, answer: &mut Answer, locked: bool) {
    let dark_mode = false;

    let (label, background, foreground) = match answer {
        Answer::Plus => (
            "+",
            egui::Color32::from_rgb(55, 125, 85),
            egui::Color32::WHITE,
        ),
        Answer::Minus => (
            "−",
            egui::Color32::from_rgb(170, 75, 80),
            egui::Color32::WHITE,
        ),
        Answer::Question => (
            "?",
            egui::Color32::from_rgb(220, 175, 65),
            egui::Color32::from_rgb(45, 40, 30),
        ),
        Answer::Empty => (
            "",
            if dark_mode {
                egui::Color32::from_rgb(38, 42, 50)
            } else {
                egui::Color32::from_rgb(245, 245, 245)
            },
            if dark_mode {
                egui::Color32::WHITE
            } else {
                egui::Color32::BLACK
            },
            // egui::Color32::from_rgb(245, 245, 245),
            // egui::Color32::BLACK,
        ),
    };

    let response = bordered_cell(ui, |ui| {
        ui.add_sized(
            [QUESTION_WIDTH, CELL_HEIGHT],
            egui::Button::new(egui::RichText::new(label).strong().color(foreground))
                .fill(background),
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

fn bordered_cell<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .show(ui, add_contents)
        .inner
}
