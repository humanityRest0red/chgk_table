use egui_toast::Toasts;
// use rodio;

use crate::models::chgk_db::{ChgkDatabase, RosterSearchState};
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
    pub(crate) timer_open: bool,
    pub(crate) toasts: Toasts,
    pub(crate) chgk_db: Option<ChgkDatabase>,
    pub(crate) roster_states: Vec<RosterSearchState>,
    pub(crate) rosters_open: bool,
    pub(crate) rosters_overview_open: bool,
    pub(crate) selected_team: usize,
    pub(crate) in_setup: bool, 
    // pub(crate) audio: rodio::OutputStreamHandle,
}
