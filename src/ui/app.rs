use eframe::egui;

use crate::models::chgk_game::{Game, Player};
use crate::models::timer::GameTimer;
use crate::ui::save_game::my_toasts;
use crate::utils::excel_saver::export_to_excel;

use super::chgk_app::ChgkApp;
// use super::timer_panel::show_timer;
use super::cells::{answer_cell, colored_cell, header_cell};
use super::layout::{
    BURGUNDY, CELL_HEIGHT, QUESTION_WIDTH, RATING_WIDTH, ROUND_FILL_COLOR, ROUND_TOTAL_WIDTH,
    TEAM_WIDTH, TOTAL_WIDTH,
};
use super::toolbar::{toolbar_button, LOCKED_COLOR, TOOLBAR_BLUE, UNLOCKED_COLOR};

impl Default for ChgkApp {
    fn default() -> Self {
        Self {
            round_count: 3,
            questions_per_round: 12,
            team_count: 3,
            game: None,
            results_locked: false,
            rounds_vertical: true,
            dark_mode: false,
            timer: GameTimer::default(),
            toasts: my_toasts(),
        }
    }
}

impl ChgkApp {
    // // --- МЕТОДЫ СОХРАНЕНИЯ И ЗАГРУЗКИ ---
    // fn save_game(&self) {
    //     if let Some(game) = &self.game {
    //         if let Ok(json) = serde_json::to_string_pretty(game) {
    //             if let Err(e) = std::fs::write("game_save.json", json) {
    //                 eprintln!("Ошибка сохранения: {}", e);
    //             }
    //         }
    //     }
    // }

    // fn load_game(&mut self, ctx: &egui::Context) {
    //     if let Ok(data) = std::fs::read_to_string("game_save.json") {
    //         if let Ok(game) = serde_json::from_str::<Game>(&data) {
    //             self.game = Some(game);
    //             if let Some(_) = &self.game {
    //                 let size = self.game_window_size();
    //                 ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
    //             }
    //         }
    //     }
    // }
    // -----------------------------------

    fn show_setup(&mut self, ui: &mut egui::Ui) {
        let card_color = if self.dark_mode {
            egui::Color32::from_rgb(38, 42, 50)
        } else {
            egui::Color32::WHITE
        };

        let text_color = if self.dark_mode {
            egui::Color32::from_rgb(235, 238, 242)
        } else {
            egui::Color32::from_rgb(45, 55, 70)
        };

        let accent = egui::Color32::from_rgb(0, 105, 170);

        let size = egui::vec2(400.0, 400.0);
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));

        ui.vertical_centered(|ui| {
            egui::Frame::new()
                .fill(card_color)
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(220, 226, 234),
                ))
                .corner_radius(egui::CornerRadius::same(18))
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    // ui.set_height(440.0);
                    // ui.set_width(300.0);

                    ui.add_space(24.0);

                    egui::Grid::new("game_setup_grid")
                        .num_columns(2)
                        .spacing([20.0, 16.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Количество туров").color(text_color));
                            ui.add_sized(
                                [50.0, 34.0],
                                egui::DragValue::new(&mut self.round_count).range(1..=30),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Вопросов в туре").color(text_color));
                            ui.add_sized(
                                [50.0, 34.0],
                                egui::DragValue::new(&mut self.questions_per_round).range(1..=100),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Количество команд").color(text_color));
                            ui.add_sized(
                                [50.0, 34.0],
                                egui::DragValue::new(&mut self.team_count).range(1..=50),
                            );
                            ui.end_row();
                        });

                    ui.add_space(24.0);

                    if ui
                        .add_sized(
                            [240.0, 44.0],
                            egui::Button::new(
                                egui::RichText::new("Создать таблицу")
                                    .size(18.0)
                                    .strong()
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(accent)
                            .corner_radius(egui::CornerRadius::same(10)),
                        )
                        .clicked()
                    {
                        let game =
                            Game::new(self.round_count, self.questions_per_round, self.team_count);

                        self.results_locked = false;
                        self.rounds_vertical = false;
                        self.timer = GameTimer::default();
                        self.game = Some(game);

                        let size = self.game_window_size();
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                    }

                    ui.add_space(10.0);

                    ui.vertical_centered(|ui| {
                        let theme_label = if self.dark_mode {
                            "☀  Светлая тема"
                        } else {
                            "☾  Тёмная тема"
                        };

                        if ui.button(theme_label).clicked() {
                            self.dark_mode = !self.dark_mode;

                            ui.ctx().set_theme(if self.dark_mode {
                                egui::Theme::Dark
                            } else {
                                egui::Theme::Light
                            });
                        }
                    });
                });
        });
    }

    fn show_game(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut go_to_settings = false;
        let mut sort_teams = false;
        let mut save_requested = false;
        let mut load_requested = false;
        let mut export_requested = false;

        ui.horizontal_wrapped(|ui| {
            let (lock_label, lock_color) = if self.results_locked {
                ("Разблокировать ввод", LOCKED_COLOR)
            } else {
                ("Заблокировать ввод", UNLOCKED_COLOR)
            };

            if toolbar_button(ui, lock_label, 190.0, lock_color).clicked() {
                self.results_locked = !self.results_locked;
            }

            let mode_button = if self.rounds_vertical {
                "Все туры в одну линию"
            } else {
                "Туры отдельнымы блоками"
            };

            if toolbar_button(ui, mode_button, 210.0, TOOLBAR_BLUE).clicked() {
                self.rounds_vertical = !self.rounds_vertical;

                if let Some(_) = &self.game {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                        self.game_window_size(),
                    ));
                }
            }

            if toolbar_button(ui, "Сортировать", 130.0, TOOLBAR_BLUE).clicked() {
                sort_teams = true;
            }

            // --- ДОБАВЛЕННЫЕ КНОПКИ СОХРАНЕНИЯ И ЗАГРУЗКИ ---
            if toolbar_button(ui, "Сохранить", 110.0, TOOLBAR_BLUE).clicked() {
                save_requested = true;
            }

            if toolbar_button(ui, "Загрузить", 110.0, TOOLBAR_BLUE).clicked() {
                load_requested = true;
            }
            // --------------------------------------------

            if toolbar_button(ui, "Экспорт Excel", 130.0, TOOLBAR_BLUE).clicked() {
                export_requested = true;
            }

            if toolbar_button(ui, "Настройки", 110.0, TOOLBAR_BLUE).clicked() {
                go_to_settings = true;
            }
        });

        if save_requested {
            self.save_game();
        }
        if load_requested {
            self.load_game(ctx);
        }
        self.toasts.show(ui);

        if export_requested {
            export_to_excel(self.game.clone());
        }

        if go_to_settings {
            self.game = None;
            self.results_locked = false;
            self.rounds_vertical = false;
            self.timer = GameTimer::default();
            return;
        }

        ui.add_space(8.0);
        ui.separator();

        let locked = self.results_locked;
        let vertical = self.rounds_vertical;

        let Some(game) = self.game.as_mut() else {
            return;
        };

        if sort_teams {
            game.sort_game_teams();
        }

        if vertical {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    draw_rounds_vertical(ui, game, locked);
                });
        } else {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    draw_all_rounds_horizontal(ui, game, locked);
                });
        }
    }
}

fn draw_score_grid(
    ui: &mut egui::Ui,
    grid_id: &str,
    game: &mut Game,
    locked: bool,
    rounds_to_show: &[usize],
    start_q_num: usize,
) {
    egui::Grid::new(grid_id)
        .striped(true)
        .spacing([1.0, 1.0])
        .show(ui, |ui| {
            header_cell(ui, "Команда", TEAM_WIDTH);
            header_cell(ui, "Итог", TOTAL_WIDTH);
            header_cell(ui, "R", RATING_WIDTH);

            let mut current_q = start_q_num;
            for &r_idx in rounds_to_show {
                let round_len = game.rounds[r_idx].questions.len();
                for _ in 0..round_len {
                    header_cell(ui, &current_q.to_string(), QUESTION_WIDTH);
                    current_q += 1;
                }
                header_cell(ui, &format!("Тур {}", r_idx + 1), ROUND_TOTAL_WIDTH);
            }
            ui.end_row();

            for t_idx in 0..game.teams.len() {
                draw_team_base_info(ui, t_idx, game);
                for &r_idx in rounds_to_show {
                    draw_round_data_for_team(ui, game, r_idx, t_idx, locked);
                }
                ui.end_row();
            }

            ui.label(
                egui::RichText::new("R вопроса")
                    .strong()
                    .color(egui::Color32::WHITE),
            );
            ui.label("");
            ui.label("");

            for &r_idx in rounds_to_show {
                let question_ratings = game.question_ratings();
                for rating in &question_ratings[r_idx] {
                    colored_cell(ui, &rating.to_string(), QUESTION_WIDTH, ROUND_FILL_COLOR);
                }
                ui.label("");
            }
            ui.end_row();
        });
}

fn draw_team_base_info(ui: &mut egui::Ui, team_index: usize, game: &mut Game) {
    ui.add_sized(
        [TEAM_WIDTH, CELL_HEIGHT],
        egui::TextEdit::singleline(&mut game.teams[team_index].name),
    );

    let score_text = game.team_score_text(team_index);
    let rating = game.team_rating(team_index);

    colored_cell(ui, &score_text, TOTAL_WIDTH, BURGUNDY);
    colored_cell(ui, &rating.to_string(), RATING_WIDTH, BURGUNDY);
}

fn draw_round_data_for_team(
    ui: &mut egui::Ui,
    game: &mut Game,
    round_index: usize,
    team_index: usize,
    locked: bool,
) {
    for q_idx in 0..game.questions_per_round() {
        let answer = game.answer_mut(round_index, q_idx, team_index);
        answer_cell(ui, answer, locked);
    }

    let score_text = game.round_score_text(team_index, round_index);
    colored_cell(ui, &score_text, ROUND_TOTAL_WIDTH, ROUND_FILL_COLOR);
}

fn draw_all_rounds_horizontal(ui: &mut egui::Ui, game: &mut Game, locked: bool) {
    let all_rounds: Vec<usize> = (0..game.rounds.len()).collect();
    draw_score_grid(ui, "h_all", game, locked, &all_rounds, 1);
}

fn draw_rounds_vertical(ui: &mut egui::Ui, game: &mut Game, locked: bool) {
    let mut q_num = 1;
    for r_idx in 0..game.rounds.len() {
        ui.heading(format!("Тур {}", r_idx + 1));
        draw_score_grid(ui, &format!("v_{r_idx}"), game, locked, &[r_idx], q_num);
        q_num += game.rounds[r_idx].questions.len();
        ui.add_space(12.0);
    }
}

impl eframe::App for ChgkApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let background = if self.dark_mode {
            egui::Color32::from_rgb(30, 33, 38)
        } else {
            egui::Color32::from_rgb(248, 250, 255)
        };

        ui.painter().rect_filled(ui.max_rect(), 0.0, background);
        ui.visuals_mut().panel_fill = background;

        let ctx = ui.ctx().clone();

        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
            1200.0, 0.0,
        )));

        let mut style = (*ctx.style_of(egui::Theme::Light)).clone();

        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::proportional(32.0)),
            (egui::TextStyle::Body, egui::FontId::proportional(24.0)),
            (egui::TextStyle::Button, egui::FontId::proportional(24.0)),
            (egui::TextStyle::Small, egui::FontId::proportional(22.0)),
            (egui::TextStyle::Monospace, egui::FontId::monospace(20.0)),
        ]
        .into();

        ctx.set_style_of(egui::Theme::Light, style.clone());
        ctx.set_style_of(egui::Theme::Dark, style);

        ctx.set_theme(if self.dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });

        if self.game.is_some() {
            self.show_game(ui, &ctx);
        } else {
            ui.vertical_centered(|ui| {
                self.show_setup(ui);
            });
        }

        self.show_rosters_window(&ctx);
    }
}

impl ChgkApp {
    fn show_rosters_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Настройка составов").show(ctx, |ui| {
            if let Some(game) = &mut self.game {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for t_idx in 0..game.teams.len() {
                        // Используем collapsing, чтобы список игроков каждой команды не занимал весь экран
                        ui.collapsing(
                            format!(
                                "{} ({} чел.)",
                                game.teams[t_idx].name,
                                game.teams[t_idx].roster.len()
                            ),
                            |ui| {
                                egui::Grid::new(format!("grid_team_{t_idx}"))
                                    .num_columns(4) // Добавляем колонку для удаления конкретного игрока
                                    .spacing([10.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Фамилия");
                                        ui.label("Имя");
                                        ui.label("Отчество");
                                        ui.label(""); // Колонка для кнопки удаления
                                        ui.end_row();

                                        for p_idx in 0..game.teams[t_idx].roster.len() {
                                            let player = &mut game.teams[t_idx].roster[p_idx];
                                            ui.add(
                                                egui::TextEdit::singleline(&mut player.last_name)
                                                    .desired_width(100.0),
                                            );
                                            ui.add(
                                                egui::TextEdit::singleline(&mut player.first_name)
                                                    .desired_width(100.0),
                                            );
                                            ui.add(
                                                egui::TextEdit::singleline(&mut player.middle_name)
                                                    .desired_width(100.0),
                                            );

                                            if ui.button("❌").clicked() {
                                                // Помечаем игрока на удаление или удаляем сразу (см. ниже)
                                            }
                                            ui.end_row();
                                        }
                                    });

                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    if ui.button("➕ Добавить игрока").clicked() {
                                        if game.teams[t_idx].roster.len() < 9 {
                                            game.teams[t_idx].roster.push(Player::default());
                                        }
                                    }
                                });
                            },
                        );
                        ui.separator();
                    }
                });
            } else {
                ui.label("Сначала создайте или загрузите игру");
            }
        });
    }
}

// {
//         // let available_height = ui.available_height();

//         if self.game.is_some() {
//             self.show_game(ui, &ctx);
//         } else {
//             // let menu_height = 280.0;
//             // let top_space = ((available_height - menu_height) / 2.0).max(0.0);
//             // ui.add_space(top_space);

//             ui.vertical_centered(|ui| {
//                 self.show_setup(ui);
//             });
//         }

//         // let available = ui.available_size();
//         // let timer_height = 300.0;
//         // let content_height = (available.y - timer_height).max(0.0);

//         // ui.allocate_ui_with_layout(
//         //     egui::vec2(available.x, content_height),
//         //     egui::Layout::top_down(egui::Align::Min),
//         //     |ui| {
//         //         if self.game.is_some() {
//         //             self.show_game(ui, &ctx);
//         //         } else {
//         //             self.show_setup(ui);
//         //         }
//         //     },
//         // );

//         // let need_timer = false;
//         // if need_timer {
//         //     ui.separator();

//         //     ui.allocate_ui_with_layout(
//         //         egui::vec2(available.x, timer_height),
//         //         egui::Layout::top_down(egui::Align::Center),
//         //         |ui| {
//         //             show_timer(ui, &mut self.timer);
//         //         },
//         //     );
//         // }
//     }
