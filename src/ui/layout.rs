use eframe::egui;

use super::chgk_app::ChgkApp;

pub(super) const TEAM_WIDTH: f32 = 220.0;
pub(super) const TOTAL_WIDTH: f32 = 90.0;
pub(super) const RATING_WIDTH: f32 = 42.0;
pub(super) const QUESTION_WIDTH: f32 = 42.0;
pub(super) const ROUND_TOTAL_WIDTH: f32 = 72.0;
pub(super) const CELL_HEIGHT: f32 = 40.0;
pub(super) const WIDTH_PADDING: f32 = 65.0;
pub(super) const HEIGHT_PADDING: f32 = 30.0;

const NAVY: egui::Color32 = egui::Color32::from_rgb(10, 39, 91);
pub(super) const BURGUNDY: egui::Color32 = egui::Color32::from_rgb(75, 15, 45);

pub(super) const ROUND_FILL_COLOR: egui::Color32 = BURGUNDY;
// const ROUND_TEXT_COLOR: egui::Color32 = egui::Color32::WHITE;
pub(super) const HEADER_FILL_COLOR: egui::Color32 = NAVY;
pub(super) const HEADER_TEXT_COLOR: egui::Color32 = egui::Color32::WHITE;

impl ChgkApp {
    pub(super) fn game_window_size(&self) -> egui::Vec2 {
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
        let width = TEAM_WIDTH + TOTAL_WIDTH + RATING_WIDTH + questions_width + ROUND_TOTAL_WIDTH + WIDTH_PADDING;

        egui::vec2(width, height)
    }
}
