use eframe::egui;

use crate::models::timer::GameTimer;

const BLUE: egui::Color32 = egui::Color32::from_rgb(0, 105, 170);
const RED: egui::Color32 = egui::Color32::from_rgb(244, 67, 54);
const CARD: egui::Color32 = egui::Color32::from_rgb(248, 250, 255);
const TEXT: egui::Color32 = egui::Color32::from_rgb(55, 65, 80);

pub(super) fn show_timer(ui: &mut egui::Ui, timer: &mut GameTimer) {
    let remaining = timer.remaining();
    let is_writing = timer.is_writing();
    let is_finished = timer.is_finished();
    let is_running = timer.is_running();

    let total = if is_writing { 10.0 } else { 60.0 };
    let progress = (remaining as f32 / total).clamp(0.0, 1.0);
    let number = if is_finished { 0 } else { remaining };
    let timer_color = if is_writing {
        RED
    } else {
        BLUE
    };

    egui::Frame::new()
        .fill(CARD)
        .corner_radius(egui::CornerRadius::same(24))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(460.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("◴  ТАЙМЕР")
                        .size(20.0)
                        .strong()
                        .color(TEXT),
                );
            });

            ui.add_space(12.0);

            ui.vertical_centered(|ui| {
                let diameter = 230.0;
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(diameter, diameter), egui::Sense::click());

                let painter = ui.painter_at(rect);
                let center = rect.center();
                let radius = diameter * 0.43;

                // Основа круга.
                painter.circle_stroke(
                    center,
                    radius,
                    egui::Stroke::new(7.0, egui::Color32::from_rgb(215, 226, 235)),
                );

                // Дуга, показывающая оставшееся время.
                let segments = 100;
                let arc_segments = ((progress * segments as f32).round() as usize).max(1);
                let start_angle = -std::f32::consts::FRAC_PI_2;

                let mut points = Vec::with_capacity(arc_segments + 1);
                for i in 0..=arc_segments {
                    let fraction = i as f32 / segments as f32;
                    let angle = start_angle + std::f32::consts::TAU * fraction;
                    points.push(
                        center + egui::vec2(angle.cos(), angle.sin()) * radius,
                    );
                }

                for pair in points.windows(2) {
                    painter.line_segment(
                        [pair[0], pair[1]],
                        egui::Stroke::new(7.0, timer_color),
                    );
                }

                // Красная метка на окружности.
                let marker_angle = start_angle + std::f32::consts::TAU * progress;
                let marker = center
                    + egui::vec2(marker_angle.cos(), marker_angle.sin()) * radius;
                painter.circle_filled(marker, 9.0, RED);

                // Число в центре.
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    number.to_string(),
                    egui::FontId::proportional(76.0),
                    timer_color,
                );

                // Нажатие на круг: пауза или продолжение.
                if response.clicked() {
                    if timer.is_running() {
                        timer.pause();
                    } else if !timer.is_finished() {
                        timer.resume();
                    }
                }

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    round_mode_icon(ui, &[(0.0, -1.0)], "Обычный");
                    round_mode_icon(
                        ui,
                        &[(0.0, -1.0), (0.0, 1.0)],
                        "Дуплет",
                    );
                    round_mode_icon(
                        ui,
                        &[(-0.87, -0.5), (0.87, -0.5), (0.0, 1.0)],
                        "Блиц",
                    );

                    if timer_icon_button(ui, "Звук", |painter, center, color| {
                        draw_speaker(painter, center, color);
                    }) {
                        // Подключи здесь переключатель звука, если добавишь его в состояние приложения.
                    }

                    if timer_icon_button(ui, "Сброс", |painter, center, color| {
                        painter.text(
                            center,
                            egui::Align2::CENTER_CENTER,
                            "⟳",
                            egui::FontId::proportional(36.0),
                            color,
                        );
                    }) {
                        timer.reset();
                    }

                    if timer_icon_button(
                        ui,
                        if is_running { "Пауза" } else { "Старт / продолжить" },
                        |painter, center, color| {
                            if is_running {
                                draw_pause(painter, center, color);
                            } else {
                                draw_play(painter, center, color);
                            }
                        },
                    ) {
                        if timer.is_running() {
                            timer.pause();
                        } else if timer.is_finished() {
                            timer.start();
                        } else {
                            timer.resume();
                        }
                    }
                });

                ui.add_space(8.0);

                let status = if is_finished {
                    "Время вышло"
                } else if is_writing {
                    "Время на запись ответа"
                } else if is_running {
                    "Идёт игровой отсчёт"
                } else {
                    "Нажмите «Старт», чтобы начать"
                };

                ui.label(
                    egui::RichText::new(status)
                        .size(16.0)
                        .color(egui::Color32::from_rgb(100, 110, 125)),
                );
            });
        });
}

fn round_mode_icon(
    ui: &mut egui::Ui,
    markers: &[(f32, f32)],
    tooltip: &str,
) -> bool {
    timer_icon_button(ui, tooltip, |painter, center, color| {
        let radius = 19.0;

        painter.circle_stroke(
            center,
            radius,
            egui::Stroke::new(4.0, color),
        );

        for &(x, y) in markers {
            painter.circle_filled(
                center + egui::vec2(x * radius, y * radius),
                4.5,
                RED,
            );
        }
    })
}

fn timer_icon_button(
    ui: &mut egui::Ui,
    tooltip: &str,
    draw_icon: impl FnOnce(&egui::Painter, egui::Pos2, egui::Color32),
) -> bool {
    let size = egui::vec2(48.0, 48.0);
    let (rect, response) =
        ui.allocate_exact_size(size, egui::Sense::click());

    let color = if response.hovered() {
        egui::Color32::from_rgb(0, 135, 205)
    } else {
        BLUE
    };

    draw_icon(&ui.painter_at(rect), rect.center(), color);

    response.on_hover_text(tooltip).clicked()
}

fn draw_play(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    painter.circle_stroke(center, 16.0, egui::Stroke::new(2.5, color));

    painter.add(egui::Shape::convex_polygon(
        vec![
            center + egui::vec2(-4.0, -8.0),
            center + egui::vec2(8.0, 0.0),
            center + egui::vec2(-4.0, 8.0),
        ],
        color,
        egui::Stroke::NONE,
    ));
}

fn draw_pause(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    painter.circle_stroke(center, 16.0, egui::Stroke::new(2.5, color));

    painter.rect_filled(
        egui::Rect::from_center_size(
            center + egui::vec2(-4.0, 0.0),
            egui::vec2(4.0, 14.0),
        ),
        1.0,
        color,
    );

    painter.rect_filled(
        egui::Rect::from_center_size(
            center + egui::vec2(4.0, 0.0),
            egui::vec2(4.0, 14.0),
        ),
        1.0,
        color,
    );
}

fn draw_speaker(painter: &egui::Painter, center: egui::Pos2, color: egui::Color32) {
    let speaker = vec![
        center + egui::vec2(-15.0, -6.0),
        center + egui::vec2(-8.0, -6.0),
        center + egui::vec2(1.0, -13.0),
        center + egui::vec2(1.0, 13.0),
        center + egui::vec2(-8.0, 6.0),
        center + egui::vec2(-15.0, 6.0),
    ];

    painter.add(egui::Shape::convex_polygon(
        speaker,
        color,
        egui::Stroke::NONE,
    ));

    // Две звуковые волны справа от динамика.
    for (radius, steps) in [(9.0, 18), (14.0, 24)] {
        let mut previous = None;

        for i in 0..=steps {
            let angle = -0.9 + 1.8 * i as f32 / steps as f32;
            let point = center
                + egui::vec2(2.0 + radius * angle.cos(), radius * angle.sin());

            if let Some(from) = previous {
                painter.line_segment(
                    [from, point],
                    egui::Stroke::new(2.2, color),
                );
            }

            previous = Some(point);
        }
    }
}

