use eframe::egui;

use crate::models::chgk_db::ChgkDatabase;
use crate::models::chgk_game::Game;
use crate::models::timer::GameTimer;
use crate::ui::save_game::my_toasts;
use crate::utils::excel_saver::{export_to_excel, save_region_screenshot};

use super::cells::{answer_cell, colored_cell, header_cell};
use super::chgk_app::ChgkApp;
use super::players::DATABASE_PATH;
use super::timer_panel::show_timer;
use super::theme::{
    CELL_HEIGHT, QUESTION_WIDTH, RATING_WIDTH, ROUND_TOTAL_WIDTH, TEAM_WIDTH, TOTAL_WIDTH, WIDTH_PADDING, HEIGHT_PADDING,
    palette, apply_theme,
};
use super::toolbar::{toolbar_button, toolbar_button_colored};

impl Default for ChgkApp {
    fn default() -> Self {
        Self {
            round_count: 3,
            questions_per_round: 12,
            team_count: 3,
            game: None,
            in_setup: false,
            results_locked: false,
            rounds_vertical: true,
            dark_mode: false,
            timer: GameTimer::default(),
            timer_open: true,
            toasts: my_toasts(),
            chgk_db: None,
            roster_states: Vec::new(),
            rosters_open: false,
            rosters_overview_open: false,
            selected_team: 0,
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
        let p = palette(self.dark_mode);
        let has_game = self.game.is_some();

        let size = egui::vec2(400.0, 460.0);
        ui.ctx()
            .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));

        ui.vertical_centered(|ui| {
            egui::Frame::new()
                .fill(p.card)
                .stroke(egui::Stroke::new(1.0, p.border))
                .corner_radius(egui::CornerRadius::same(18))
                .inner_margin(egui::Margin::same(24))
                .show(ui, |ui| {
                    if has_game {
                        ui.label(
                            egui::RichText::new("Игра идёт — таблица сохранится")
                                .size(16.0)
                                .color(p.text),
                        );
                        ui.add_space(8.0);
                    }

                    egui::Grid::new("game_setup_grid")
                        .num_columns(2)
                        .spacing([20.0, 16.0])
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Количество туров").color(p.text));
                            ui.add_sized(
                                [50.0, 34.0],
                                egui::DragValue::new(&mut self.round_count).range(1..=30),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Вопросов в туре").color(p.text));
                            ui.add_sized(
                                [50.0, 34.0],
                                egui::DragValue::new(&mut self.questions_per_round).range(1..=100),
                            );
                            ui.end_row();

                            ui.label(egui::RichText::new("Количество команд").color(p.text));
                            ui.add_sized(
                                [50.0, 34.0],
                                egui::DragValue::new(&mut self.team_count).range(1..=50),
                            );
                            ui.end_row();
                        });

                    ui.add_space(20.0);

                    // Главная кнопка: создать новую или применить к существующей.
                    let label = if has_game {
                        "Применить изменения"
                    } else {
                        "Создать таблицу"
                    };

                    if ui
                        .add_sized(
                            [240.0, 44.0],
                            egui::Button::new(
                                egui::RichText::new(label)
                                    .size(18.0)
                                    .strong()
                                    .color(p.accent_text),
                            )
                            .fill(p.accent)
                            .corner_radius(egui::CornerRadius::same(10)),
                        )
                        .clicked()
                    {
                        if has_game {
                            self.apply_setup();
                            self.in_setup = false; // вернулись в таблицу
                        } else {
                            let game = Game::new(
                                self.round_count,
                                self.questions_per_round,
                                self.team_count,
                            );
                            self.results_locked = false;
                            self.rounds_vertical = true;
                            self.timer = GameTimer::default();
                            self.game = Some(game);
                        }

                        if self.chgk_db.is_none() {
                            match ChgkDatabase::load(std::path::Path::new(DATABASE_PATH)) {
                                Ok(db) => self.chgk_db = Some(db),
                                Err(e) => {
                                    eprintln!("Не удалось загрузить базу игроков: {e}")
                                }
                            }
                        }

                        if self.game.is_some() {
                            let size = self.game_window_size();
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                        }
                    }

                    if has_game {
                        ui.add_space(10.0);

                        if ui.button("←  Вернуться к игре").clicked() {
                            self.in_setup = false;
                        }

                        if ui.button("Начать новую игру").clicked() {
                            self.game = None;
                            self.results_locked = false;
                            self.rounds_vertical = true;
                            self.timer = GameTimer::default();
                            // остаёмся в настройках — там форма создания
                        }
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

                        let label = if self.timer_open {
                            "⏱ Скрыть таймер"
                        } else {
                            "⏱ Таймер"
                        };
                        if ui.button(label).clicked() {
                            self.toggle_timer();
                        }
                    });
                });
        });
    }

    fn apply_setup(&mut self) {
        let old = self.game.take();
        let mut new_game = Game::new(self.round_count, self.questions_per_round, self.team_count);

        if let Some(mut old) = old {
            let teams = self.team_count.min(old.teams.len());

            // Переносим имена команд.
            for t in 0..teams {
                new_game.teams[t].name = old.teams[t].name.clone();
                new_game.teams[t].id = old.teams[t].id.clone();
            }

            // Переносим ответы по индексам — всё, что влезло в новую сетку.
            let rounds = self.round_count.min(old.rounds.len());
            for r in 0..rounds {
                let questions = self.questions_per_round.min(old.rounds[r].questions.len());
                for q in 0..questions {
                    for t in 0..teams {
                        let value = old.answer_mut(r, q, t).clone();
                        *new_game.answer_mut(r, q, t) = value;
                    }
                }
            }
        }

        self.game = Some(new_game);

        // Составы: состояния соответствуют командам по индексу.
        // Лишние (команд стало меньше) — обрезаем,
        // недостающие (команд стало больше) — добавляем по умолчанию.
        let team_total = self.game.as_ref().map_or(0, |g| g.teams.len());
        self.roster_states.truncate(team_total);
        while self.roster_states.len() < team_total {
            self.roster_states.push(Default::default());
        }
    }

    fn show_game(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let mut go_to_settings = false;
        let mut sort_teams = false;
        let mut save_requested = false;
        let mut load_requested = false;
        let mut export_requested = false;

        ui.horizontal_wrapped(|ui| {
            let p = palette(self.dark_mode);

            let (lock_label, lock_fill) = if self.results_locked {
                ("Разблокировать ввод", p.success)
            } else {
                ("Заблокировать ввод", p.danger)
            };

            if toolbar_button_colored(ui, lock_label, 190.0, lock_fill).clicked() {
                self.results_locked = !self.results_locked;
            }

            let mode_button = if self.rounds_vertical {
                "Все туры в одну линию"
            } else {
                "Туры отдельными блоками"
            };

            if toolbar_button(ui, mode_button, 210.0).clicked() {
                self.rounds_vertical = !self.rounds_vertical;

                if let Some(_) = &self.game {
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                        self.game_window_size(),
                    ));
                }
            }

            if toolbar_button(ui, "Сортировать", 130.0).clicked() {
                sort_teams = true;
            }

            if toolbar_button(ui, "Сохранить", 110.0).clicked() {
                save_requested = true;
            }

            if toolbar_button(ui, "Загрузить", 110.0).clicked() {
                load_requested = true;
            }

            if toolbar_button(ui, "Составы", 110.0).clicked() {
                self.rosters_open = !self.rosters_open;
            }

            if toolbar_button(ui, "Обзор составов", 150.0).clicked() {
                self.rosters_overview_open = !self.rosters_overview_open;
            }

            if toolbar_button(ui, "Сохранить эксель, скрин", 130.0).clicked()
            {
                export_requested = true;
            }

            if toolbar_button(ui, "Настройки", 110.0).clicked() {
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

        if go_to_settings {
            self.in_setup = true;

            // Синхронизируем параметры с текущей игрой ОДИН раз,
            // в момент перехода в настройки.
            if let Some(game) = &self.game {
                self.round_count = game.rounds.len().max(1);
                self.questions_per_round = game.questions_per_round().max(1);
                self.team_count = game.teams.len().max(1);
            }

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

        let table_rect;

        if vertical {
            let response = egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    draw_rounds_vertical(ui, game, locked, self.chgk_db.as_ref());
                });
            table_rect = response.inner_rect;
        } else {
            let response = egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    draw_all_rounds_horizontal(ui, game, locked, self.chgk_db.as_ref());
                });
            table_rect = response.inner_rect;
        }

        let p = palette(self.dark_mode);

        let add_button = egui::Button::new(
            egui::RichText::new("➕ Добавить команду")
                .size(18.0)
                .color(p.header_text),
        )
        .fill(p.header)
        .corner_radius(egui::CornerRadius::same(8));

        if ui.add_sized([220.0, 40.0], add_button).clicked() {
            game.add_team();
        }

        // Экспорт — здесь, когда rect уже известен
        if export_requested {
            export_to_excel(self.game.clone());

            if let Err(e) = save_region_screenshot(table_rect, ctx) {
                eprintln!("Ошибка при сохранении скриншота: {}", e);
            }
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
    db: Option<&ChgkDatabase>,
) {
    // Отложенное удаление: строка помечается во время отрисовки,
    // команда удаляется после цикла, чтобы не ломать индексы.
    let mut delete_team: Option<usize> = None;

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

            let mut t_idx = 0;
            while t_idx < game.teams.len() {
                draw_team_base_info(ui, t_idx, game, db, grid_id, &mut delete_team);
                for &r_idx in rounds_to_show {
                    draw_round_data_for_team(ui, game, r_idx, t_idx, locked);
                }
                ui.end_row();
                t_idx += 1;
            }

            let p = palette(ui.visuals().dark_mode);
            header_cell(ui, "R вопроса", TEAM_WIDTH);
            // ui.label(egui::RichText::new("R вопроса").strong().color(p.text));
            ui.label("");
            ui.label("");

            for &r_idx in rounds_to_show {
                let question_ratings = game.question_ratings();
                for rating in &question_ratings[r_idx] {
                    colored_cell(ui, &rating.to_string(), QUESTION_WIDTH, p.total);
                }
                ui.label("");
            }
            ui.end_row();
        });

    // Удаляем после отрисовки сетки — когда заём game уже свободен.
    if let Some(idx) = delete_team.take() {
        game.remove_team(idx);
    }
}

fn draw_team_base_info(
    ui: &mut egui::Ui,
    team_index: usize,
    game: &mut Game,
    db: Option<&ChgkDatabase>,
    grid_id: &str,
    pending_delete: &mut Option<usize>,
) {
    draw_team_name_row(ui, team_index, game, db, grid_id, pending_delete);
    draw_team_score_row(ui, team_index, game);
}

fn draw_team_score_row(ui: &mut egui::Ui, team_index: usize, game: &mut Game) {
    let p = palette(ui.visuals().dark_mode);
    colored_cell(ui, &game.team_score_text(team_index), TOTAL_WIDTH, p.total);
    colored_cell(ui, &game.team_rating(team_index).to_string(), RATING_WIDTH, p.total);
}

fn draw_team_name_row(
    ui: &mut egui::Ui,
    team_index: usize,
    game: &mut Game,
    db: Option<&ChgkDatabase>,
    grid_id: &str,
    pending_delete: &mut Option<usize>,
) {
    let p = palette(ui.visuals().dark_mode);
    let edit_id = egui::Id::new(("team_name_edit", grid_id, team_index));
    let editing_id = edit_id.with("editing");
    let mut editing = ui.memory(|m| m.data.get_temp::<bool>(editing_id).unwrap_or(false));
    let confirm_id = egui::Id::new(("delete_confirm", grid_id, team_index));

    ui.horizontal(|ui| {
        // --- имя: жирная метка ИЛИ редактор с подсказками ---
        if editing {
            let edit = ui.add_sized(
                [TEAM_WIDTH - 40.0, CELL_HEIGHT],
                egui::TextEdit::singleline(&mut game.teams[team_index].name),
            );
            if !edit.has_focus() {
                edit.request_focus();
            }
            if let Some(db) = db {
                if draw_name_suggestions(ui, team_index, game, db, edit.clone(), edit_id) {
                    // Выбрали команду из базы — редактор закрываем
                    editing = false;
                }
            }
            if edit.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                editing = false;
            }
        } else {
            let name = game.teams[team_index].name.clone();
            let mut label = ui.add_sized(
                [TEAM_WIDTH - 40.0, CELL_HEIGHT],
                egui::Label::new(
                    egui::RichText::new(name.clone())
                        .strong()
                        .size(25.0)
                        .color(p.text),
                )
                .truncate(),
            );

            // Оригинальное название видно при наведении, если имя меняли
            let original = game.teams[team_index].original_name.clone();
            if original != name {
                label = label.on_hover_text(format!("Оригинал: {original}"));
            }

            if label.double_clicked() {
                editing = true;
            }
        }

        // --- кнопка удаления (без изменений) ---
        if game.team_has_answers(team_index) {
            let btn = ui.add_enabled(
                false,
                egui::Button::new(egui::RichText::new("−").color(egui::Color32::GRAY)),
            );
            btn.on_hover_text("У команды есть заполненные ответы — удалять нельзя");
        } else {
            let mut confirming =
                ui.memory(|m| m.data.get_temp::<bool>(confirm_id).unwrap_or(false));

            let label = if confirming { "Удалить?" } else { "−" };
            let del = ui
                .small_button(egui::RichText::new(label).strong().color(egui::Color32::WHITE));

            if del.clicked() {
                if confirming {
                    *pending_delete = Some(team_index);
                    confirming = false;
                } else {
                    confirming = true;
                }
            } else if ui.input(|i| i.pointer.any_click()) && !del.hovered() {
                confirming = false;
            }

            ui.memory_mut(|m| m.data.insert_temp(confirm_id, confirming));
        }
    });

    ui.memory_mut(|m| m.data.insert_temp(editing_id, editing));
}

fn draw_name_suggestions(
    ui: &mut egui::Ui,
    team_index: usize,
    game: &mut Game,
    db: &ChgkDatabase,
    response: egui::Response,
    edit_id: egui::Id,
) -> bool {
    let sugg_id = edit_id.with("suggestions");
    let popup_id = edit_id.with("popup");
    let open_id = edit_id.with("popup_open");
    let mut chosen_applied = false;

    if response.changed() {
        let suggestions = db.search_team(&game.teams[team_index].name, 5);
        ui.memory_mut(|m| {
            m.data.insert_temp(sugg_id, suggestions.clone());
            m.data.insert_temp(open_id, !suggestions.is_empty());
        });
    }

    let mut open = ui.memory(|m| m.data.get_temp::<bool>(open_id).unwrap_or(false));
    let has_focus = response.has_focus();

    if has_focus || open {
        let suggestions: Vec<(String, String)> =
            ui.memory(|m| m.data.get_temp(sugg_id).unwrap_or_default());

        if !suggestions.is_empty() {
            let pos = response.rect.left_bottom() + egui::vec2(0.0, 4.0);
            let mut chosen: Option<(String, String)> = None;
            let mut popup_hovered = false;

            let area_response = egui::Area::new(popup_id)
                .order(egui::Order::Foreground)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.set_min_width(response.rect.width());

                        for (code, name) in &suggestions {
                            let item = ui.selectable_label(false, format!("{code} — {name}"));
                            popup_hovered |= item.hovered();
                            if item.clicked() {
                                chosen = Some((code.clone(), name.clone()));
                            }
                        }
                    });
                })
                .response;

            let clicked_outside = ui.input(|i| i.pointer.any_click())
                && !response.hovered()
                && !popup_hovered
                && !area_response.contains_pointer();

            if let Some((code, name)) = chosen {
                game.teams[team_index].name = name.clone();
                game.teams[team_index].original_name = name; // оригинал из базы
                game.teams[team_index].id = code;
                open = false;
                chosen_applied = true;
            } else if clicked_outside {
                open = false;
            } else if has_focus {
                open = true;
            }

            ui.memory_mut(|m| m.data.insert_temp(open_id, open));
        }
    }

    chosen_applied
}

impl ChgkApp {
    pub fn game_window_size(&self) -> egui::Vec2 {
        let Some(game) = &self.game else {
            return egui::vec2(200.0, 340.0);
        };

        let team_rows = game.teams.len() as f32;
        let rows_height = (team_rows + 2.0) * CELL_HEIGHT;
        let round_overhead = 80.0;

        let height = if self.rounds_vertical {
            game.rounds.len() as f32 * (rows_height + round_overhead) + HEIGHT_PADDING
        } else {
            rows_height + round_overhead + HEIGHT_PADDING
        };

        let questions_width = game.questions_per_round() as f32 * QUESTION_WIDTH;
        let width = TEAM_WIDTH
            + TOTAL_WIDTH
            + RATING_WIDTH
            + questions_width
            + ROUND_TOTAL_WIDTH
            + WIDTH_PADDING;

        egui::vec2(width, height)
    }
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

    let p = palette(ui.visuals().dark_mode);
    colored_cell(ui, &score_text, ROUND_TOTAL_WIDTH, p.total);
}

fn draw_all_rounds_horizontal(
    ui: &mut egui::Ui,
    game: &mut Game,
    locked: bool,
    db: Option<&ChgkDatabase>,
) {
    let all_rounds: Vec<usize> = (0..game.rounds.len()).collect();
    draw_score_grid(ui, "h_all", game, locked, &all_rounds, 1, db);
}

fn draw_rounds_vertical(
    ui: &mut egui::Ui,
    game: &mut Game,
    locked: bool,
    db: Option<&ChgkDatabase>,
) {
    let mut q_num = 1;
    for r_idx in 0..game.rounds.len() {
        ui.heading(format!("Тур {}", r_idx + 1));
        draw_score_grid(ui, &format!("v_{r_idx}"), game, locked, &[r_idx], q_num, db);
        q_num += game.rounds[r_idx].questions.len();
        ui.add_space(12.0);
    }
}

impl eframe::App for ChgkApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        let p = palette(self.dark_mode);
        // let screen = ctx.input(|i| i.raw.screen_rect);
        ui.painter().rect_filled(ui.max_rect(), 0.0, p.background);


        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
            1200.0, 0.0,
        )));

        for theme in [egui::Theme::Light, egui::Theme::Dark] {
            let mut style = (*ctx.style_of(theme)).clone();
            style.text_styles = [
                (egui::TextStyle::Heading, egui::FontId::proportional(32.0)),
                (egui::TextStyle::Body, egui::FontId::proportional(24.0)),
                (egui::TextStyle::Button, egui::FontId::proportional(24.0)),
                (egui::TextStyle::Small, egui::FontId::proportional(22.0)),
                (egui::TextStyle::Monospace, egui::FontId::monospace(20.0)),
            ].into();
            ctx.set_style_of(theme, style);
        }

        ctx.set_theme(if self.dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });

        apply_theme(&ctx);

        if self.in_setup {
            self.show_setup(ui);
        } else if self.game.is_some() {
            self.show_game(ui, &ctx);
        } else {
            self.show_setup(ui);
        }

        self.show_timer_window(&ctx);
        self.show_rosters_window(&ctx);
        self.show_rosters_overview(&ctx);
    }
}

impl ChgkApp {
    fn toggle_timer(&mut self) {
        self.timer_open = !self.timer_open;
    }

    fn show_timer_window(&mut self, ctx: &egui::Context) {
        let _ticked = self.timer.tick();

        let screen = ctx.input(|i| i.raw.screen_rect.unwrap_or(egui::Rect::NOTHING));

        let mut open = self.timer_open;
        egui::Window::new("⏱  Таймер")
            .open(&mut open)
            .default_pos(screen.right_top() - egui::vec2(540.0, 4.0))
            .default_width(520.0)
            .resizable(false)
            .collapsible(true)
            .show(ctx, |ui| {
                if self.timer.is_running() || self.timer.is_writing() {
                    ui.ctx().request_repaint();
                }
                ui.vertical_centered(|ui| {
                    show_timer(ui, &mut self.timer);
                });
            });

        self.timer_open = open;
    }
}
