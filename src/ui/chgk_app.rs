use egui_toast::Toasts;

use crate::models::chgk_game::Game;
use crate::models::timer::GameTimer;

pub struct ChgkApp {
    pub(crate) round_count: usize,
    pub(crate) questions_per_round: usize,
    pub(crate) team_count: usize,
    pub(crate) game: Option<Game>,
    pub(crate) results_locked: bool,
    pub(crate) rounds_vertical: bool,
    pub(crate) dark_mode: bool,
    pub(crate) timer: GameTimer,
    pub(crate) toasts: Toasts,
}
