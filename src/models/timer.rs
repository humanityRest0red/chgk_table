use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Ready,
    Main,
    Writing,
    Finished,
}

/// События таймера: уровень приложения превращает их в звуки.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerEvent {
    /// Нажали «Старт» — начало отсчёта.
    Start,
    /// Новый тик в последние 10 секунд главной фазы (10, 9, …, 1).
    SecondTick,
    /// Время на обсуждение вышло, начались 10 секунд на запись.
    WritingStart,
    /// Тик на 3, 2 или 1 секунде фазы записи.
    FinalTick,
    /// Всё время вышло.
    Finished,
}

pub struct GameTimer {
    phase: Phase,
    deadline: Option<Instant>,
    paused_seconds: u64,
    warned_at_ten: bool,
    last_ticked_second: Option<u64>,
    events: Vec<TimerEvent>,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            phase: Phase::Ready,
            deadline: None,
            paused_seconds: 60,
            warned_at_ten: false,
            last_ticked_second: None,
            events: Vec::new(),
        }
    }
}

impl GameTimer {
    /// Вызывать каждый кадр. Возвращает true один раз — при переходе
    /// в последние 10 секунд (как раньше). Сами события — через take_events().
    pub fn tick(&mut self) -> bool {
        let mut warned = false;

        let Some(deadline) = self.deadline else {
            return false;
        };
        let now = Instant::now();

        // Сначала смены фаз.
        if now >= deadline {
            match self.phase {
                Phase::Main => {
                    self.phase = Phase::Writing;
                    self.deadline = Some(now + Duration::from_secs(10));
                    self.last_ticked_second = None;
                    self.events.push(TimerEvent::WritingStart);
                    return false;
                }
                Phase::Writing => {
                    self.phase = Phase::Finished;
                    self.deadline = None;
                    self.events.push(TimerEvent::Finished);
                    return false;
                }
                Phase::Ready | Phase::Finished => {}
            }
        }

        let Some(deadline) = self.deadline else {
            return false;
        };
        let remaining = deadline.saturating_duration_since(now).as_secs();

        match self.phase {
            Phase::Main => {
                if remaining <= 10 && self.last_ticked_second != Some(remaining) {
                    self.last_ticked_second = Some(remaining);
                    self.events.push(TimerEvent::SecondTick);
                }
                if remaining <= 10 && !self.warned_at_ten {
                    self.warned_at_ten = true;
                    warned = true;
                }
            }
            Phase::Writing => {
                if (1..=3).contains(&remaining) && self.last_ticked_second != Some(remaining) {
                    self.last_ticked_second = Some(remaining);
                    self.events.push(TimerEvent::FinalTick);
                }
            }
            Phase::Ready | Phase::Finished => {}
        }

        warned
    }

    /// Забрать накопившиеся события (после этого список пуст).
    pub fn take_events(&mut self) -> Vec<TimerEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn start(&mut self) {
        self.phase = Phase::Main;
        self.deadline = Some(Instant::now() + Duration::from_secs(60));
        self.paused_seconds = 60;
        self.warned_at_ten = false;
        self.last_ticked_second = None;
        self.events.push(TimerEvent::Start);
    }

    pub fn reset(&mut self) {
        self.phase = Phase::Ready;
        self.deadline = None;
        self.paused_seconds = 60;
        self.warned_at_ten = false;
        self.last_ticked_second = None;
        self.events.clear();
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
