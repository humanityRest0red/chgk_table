use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Ready,
    Main,
    Writing,
    Finished,
}

pub struct GameTimer {
    phase: Phase,
    deadline: Option<Instant>,
    paused_seconds: u64,
    warned_at_ten: bool,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            phase: Phase::Ready,
            deadline: None,
            paused_seconds: 60,
            warned_at_ten: false,
        }
    }
}

impl GameTimer {
    pub fn tick(&mut self) -> bool {
        let Some(deadline) = self.deadline else {
            return false;
        };

        let remaining = deadline.saturating_duration_since(Instant::now()).as_secs();

        if self.phase == Phase::Main && remaining <= 10 && !self.warned_at_ten {
            self.warned_at_ten = true;
            return true;
        }

        if Instant::now() >= deadline {
            match self.phase {
                Phase::Main => {
                    self.phase = Phase::Writing;
                    self.deadline = Some(Instant::now() + Duration::from_secs(10));
                    return true;
                }
                Phase::Writing => {
                    self.phase = Phase::Finished;
                    self.deadline = None;
                    return true;
                }
                Phase::Ready | Phase::Finished => {}
            }
        }

        false
    }

    pub fn start(&mut self) {
        self.phase = Phase::Main;
        self.deadline = Some(Instant::now() + Duration::from_secs(60));
        self.paused_seconds = 60;
        self.warned_at_ten = false;
    }

    pub fn reset(&mut self) {
        self.phase = Phase::Ready;
        self.deadline = None;
        self.paused_seconds = 60;
        self.warned_at_ten = false;
    }

    pub fn remaining(&self) -> u64 {
        self.deadline
            .map(|deadline| {
                deadline
                    .saturating_duration_since(Instant::now())
                    .as_secs()
                    .saturating_add(1)
            })
            .unwrap_or_else(|| {
                if self.phase == Phase::Ready {
                    60
                } else {
                    self.paused_seconds
                }
            })
    }

    pub fn is_running(&self) -> bool {
        self.deadline.is_some()
    }

    pub fn is_finished(&self) -> bool {
        self.phase == Phase::Finished
    }

    pub fn is_writing(&self) -> bool {
        self.phase == Phase::Writing
    }

    pub fn pause(&mut self) {
        if let Some(deadline) = self.deadline {
            self.paused_seconds = deadline
                .saturating_duration_since(Instant::now())
                .as_secs()
                .saturating_add(1);
            self.deadline = None;
        }
    }

    pub fn resume(&mut self) {
        if self.paused_seconds > 0 {
            self.deadline = Some(Instant::now() + Duration::from_secs(self.paused_seconds));
        }
    }
}
