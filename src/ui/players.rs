use super::chgk_app::ChgkApp;
use super::theme::palette;

use crate::models::chgk_db::{ChgkDatabase, PlayerRecord, RosterSearchState};
use crate::models::chgk_game::{Player, Team};

use eframe::egui;
use std::collections::HashSet;

pub(crate) const DATABASE_PATH: &str = "data/players.xlsx";

impl ChgkApp {
    pub(super) fn show_rosters_window(&mut self, ctx: &egui::Context) {
        if !self.rosters_open {
            return;
        }

        let mut open = true;
        let Self {
            game,
            chgk_db,
            roster_states,
            selected_team,
            ..
        } = self;

        let Some(game) = game else {
            self.rosters_open = open;
            return;
        };

        // Состояния поиска всегда по числу команд
        roster_states.resize_with(game.teams.len(), RosterSearchState::default);
        if *selected_team >= game.teams.len() {
            *selected_team = 0;
        }

        egui::Window::new("Настройка составов")
            .open(&mut open)
            .resizable(true)
            // Увеличили размер, добавили минимальный, чтобы окно не становилось слишком узким
            .default_width(960.0)
            .default_height(640.0)
            .min_size(egui::Vec2::new(700.0, 500.0))
            .show(ctx, |ui| {
                // --- Загрузка базы ---
                if chgk_db.is_none()
                    && ui
                        .button("📂 Загрузить базу игроков (players.xlsx)")
                        .clicked()
                {
                    match ChgkDatabase::load(std::path::Path::new(DATABASE_PATH)) {
                        Ok(db) => *chgk_db = Some(db),
                        Err(e) => eprintln!("Ошибка загрузки базы: {e}"),
                    }
                }

                if let Some(db) = chgk_db {
                    ui.label(format!(
                        "База загружена: {} игроков, {} команд",
                        db.players.len(),
                        db.teams.len()
                    ));
                }

                ui.add_space(4.0);

                // --- Левая панель: список команд (все команды сразу) ---
                egui::Panel::left("roster_teams_panel")
                    .resizable(true)
                    .default_size(240.0)
                    .show(ui, |ui| {
                        ui.heading("Команды");
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for t_idx in 0..game.teams.len() {
                                let selected = t_idx == *selected_team;
                                let label = format!(
                                    "{} ({})",
                                    game.teams[t_idx].name,
                                    game.teams[t_idx].roster.len()
                                );
                                if team_label(ui, selected, label).clicked() {
        *selected_team = t_idx;
    }
                            }
                        });
                    });

                // Занятые игроки ДРУГИХ команд — считаем до мутабельного заимствования team
                let taken_elsewhere: HashSet<(String, String)> = game
                    .teams
                    .iter()
                    .enumerate()
                    .filter(|(t, _)| *t != *selected_team)
                    .flat_map(|(_, team)| {
                        team.roster
                            .iter()
                            .map(|p| (p.last_name.clone(), p.first_name.clone()))
                    })
                    .collect();

                let team = &mut game.teams[*selected_team];
                let state = &mut roster_states[*selected_team];

                ui.heading(format!("Состав: {}", team.name));
                ui.separator();

                // --- Список игроков ---
                let mut remove_idx: Option<usize> = None;
                egui::ScrollArea::vertical()
                    .max_height((ui.available_height() - 90.0).max(60.0))
                    .show(ui, |ui| {
                        if team.roster.is_empty() {
                            ui.weak("Пока пусто — добавьте игроков поиском ниже");
                        }
                        for (p_idx, player) in team.roster.iter().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.button("x").clicked() {
                                    remove_idx = Some(p_idx);
                                }
                                ui.label(player.full_name());
                                let role_color = role_color(ui, &player.role);
                                ui.colored_label(role_color, &player.role);
                            });
                        }
                    });
                if let Some(p_idx) = remove_idx {
                    team.roster.remove(p_idx);
                    state.query.clear();
                    state.suggestions.clear();
                }

                ui.add_space(6.0);
                ui.separator();

                // --- Поиск по базе ---
                let Some(db) = chgk_db else {
                    ui.label("База игроков не загружена — можно добавлять вручную кнопкой ниже");
                    if ui.button("➕ Добавить игрока вручную").clicked() && team.roster.len() < 9
                    {
                        team.roster.push(Player::default());
                    }
                    return;
                };

                let edit = ui.add(
                    egui::TextEdit::singleline(&mut state.query)
                        .hint_text("Фамилия игрока…")
                        .desired_width(400.0), // Чуть шире, чтобы фамилии не обрезались
                );

                if edit.changed() {
                    state.suggestions = db.search(&state.query, 5);
                }

                if !state.query.is_empty() && !state.suggestions.is_empty() {
                    // Выбор запоминаем, изменения — только после цикла
                    let mut chosen: Option<PlayerRecord> = None;

                    for rec in &state.suggestions {
                        // Уже есть в этой команде?
                        let in_this = team.roster.iter().any(|p| {
                            p.last_name == rec.last_name && p.first_name == rec.first_name
                        });
                        // Уже есть в любой другой команде?
                        let in_other = taken_elsewhere
                            .contains(&(rec.last_name.clone(), rec.first_name.clone()));

                        let label =
                            format!("{} — {} ({})", rec.full_name(), rec.team_name, rec.role);
                        let label = if in_this {
                            format!("{label}  •  уже в этой команде")
                        } else if in_other {
                            format!("{label}  •  уже в другой команде")
                        } else {
                            label
                        };

                        let mut clicked = false;
                        ui.add_enabled_ui(!in_this && !in_other, |ui| {
                            if ui.small_button(label).clicked() {
                                clicked = true;
                            }
                        });
                        if clicked {
                            chosen = Some(rec.clone());
                            break;
                        }
                    }

                    if let Some(rec) = chosen {
                        maybe_autofill_team(team, &rec);
                        let player = rec.to_player_for_team(&team.id, &team.name);
                        team.roster.push(player);
                        state.query.clear();
                        state.suggestions.clear();
                    }
                }

                if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(rec) = state.suggestions.first().cloned() {
                        maybe_autofill_team(team, &rec);
                        let player = rec.to_player_for_team(&team.id, &team.name);
                        team.roster.push(player);
                        state.query.clear();
                        state.suggestions.clear();
                    }
                }
            });

        self.rosters_open = open;
    }

    pub(super) fn show_rosters_overview(&mut self, ctx: &egui::Context) {
        if !self.rosters_overview_open {
            return;
        }

        let mut open = true;
        let Self {
            game,
            selected_team,
            ..
        } = self;

        let Some(game) = game else {
            self.rosters_overview_open = open;
            return;
        };

        let mut open_rosters_requested = false;

        egui::Window::new("Все составы")
            .open(&mut open)
            .resizable(true)
            .default_size([1100.0, 600.0])
            .show(ctx, |ui| {
                if game.teams.is_empty() {
                    ui.label("Команд пока нет");
                    return;
                }

                // Сколько строк в самой длинной команде
                let max_players = game
                    .teams
                    .iter()
                    .map(|t| t.roster.len())
                    .max()
                    .unwrap_or(0)
                    .max(1);

                egui::ScrollArea::both().show(ui, |ui| {
    egui::Grid::new("rosters_overview_grid")
        .striped(true)
        .spacing([16.0, 3.0])
        .show(ui, |ui| {
            const COLUMNS: usize = 3;

            // Проходим по командам группами по 3
            for chunk_start in (0..game.teams.len()).step_by(COLUMNS) {
                let chunk_end = (chunk_start + COLUMNS).min(game.teams.len());

               for t_idx in chunk_start..chunk_end {
    let team = &game.teams[t_idx];
    let selected = t_idx == *selected_team;
    let label = format!("{} ({})", team.name, team.roster.len());
    if team_label(ui, selected, label).clicked() {
        *selected_team = t_idx;
        open_rosters_requested = true;
    }
}

                ui.end_row();

                // Строки игроков — те же 3 команды
                let max_players = game.teams[chunk_start..chunk_end]
                    .iter()
                    .map(|t| t.roster.len())
                    .max()
                    .unwrap_or(0);

                for p_idx in 0..max_players {
                    for team in &game.teams[chunk_start..chunk_end] {
                        match team.roster.get(p_idx) {
                            Some(player) => {
                                let role_color = role_color(ui, &player.role);
                                ui.horizontal(|ui| {
                                    ui.colored_label(role_color, &player.role);
                                    ui.label(player.full_name());
                                });
                            }
                            None => {
                                ui.label("");
                            }
                        }
                    }
                    ui.end_row();
                }

                // Пустая строка-разделитель между тройками команд
                for _ in 0..chunk_end - chunk_start {
                    ui.label("");
                }
                ui.end_row();
            }
        });
});

            });

        if open_rosters_requested {
            self.rosters_open = true;
        }

        self.rosters_overview_open = open;
    }
}

fn team_label(ui: &mut egui::Ui, selected: bool, label: String) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let text_color = if dark {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(30) // почти чёрный на светлом фоне
    };

    ui.visuals_mut().selection.bg_fill = if dark {
        egui::Color32::from_gray(90) // тёмно-серый вместо синего
    } else {
        egui::Color32::from_gray(215) // светло-серый вместо синего
    };
    ui.visuals_mut().selection.stroke = (1.0, text_color).into();

    ui.selectable_label(selected, egui::RichText::new(label).strong().color(text_color))
}


fn team_name_is_default(name: &str) -> bool {
    name.is_empty() || name.starts_with("Команда ")
}

fn maybe_autofill_team(team: &mut Team, rec: &PlayerRecord) {
    if team_name_is_default(&team.name) && !rec.team_name.is_empty() {
        team.name = rec.team_name.clone();
    }
    if team.id.is_empty() && !rec.team_code.is_empty() {
        team.id = rec.team_code.clone();
    }
}

fn role_color(ui: &egui::Ui, role: &str) -> egui::Color32 {
    let p = palette(ui.visuals().dark_mode);
    match role {
        "Б" => p.success,
        "Л" => p.header,
        _ => p.text_muted,
    }
}