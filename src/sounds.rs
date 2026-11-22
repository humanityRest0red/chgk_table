use std::io::BufReader;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread;

#[derive(Clone, Copy)]
pub enum SoundEvent {
    Start,   // старт отсчёта
    Tick,    // тики последних 10 секунд
    Writing, // переход к записи (10 секунд)
    Final,   // 3, 2, 1
    Finish,  // время вышло
}

fn file_for(event: SoundEvent) -> &'static str {
    // Пока один файл.
    match event {
        SoundEvent::Start => "assets/beep.mp3",
        SoundEvent::Tick => "assets/beep.mp3",
        SoundEvent::Writing => "assets/beep.mp3",
        SoundEvent::Final => "assets/beep.mp3",
        SoundEvent::Finish => "assets/beep.mp3",
    }
}

fn volume_for(event: SoundEvent) -> f32 {
    match event {
        SoundEvent::Start | SoundEvent::Writing | SoundEvent::Finish => 1.0,
        SoundEvent::Tick => 0.45,
        SoundEvent::Final => 0.8,
    }
}

pub struct SoundPlayer {
    tx: Option<Sender<SoundEvent>>,
    muted: AtomicBool,
}

impl SoundPlayer {
    pub fn new() -> Self {
        match spawn_worker() {
            Ok(tx) => Self {
                tx: Some(tx),
                muted: AtomicBool::new(false),
            },
            Err(err) => {
                eprintln!("звук недоступен: {err}");
                Self {
                    tx: None,
                    muted: AtomicBool::new(false),
                }
            }
        }
    }

    /// Возвращает новое состояние mute.
    pub fn toggle_mute(&self) -> bool {
        !self.muted.fetch_xor(true, Ordering::Relaxed)
    }

    pub fn is_muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    pub fn play(&self, event: SoundEvent) {
        if self.muted.load(Ordering::Relaxed) {
            return;
        }
        if let Some(tx) = &self.tx {
            let _ = tx.send(event);
        }
    }
}

fn spawn_worker() -> Result<Sender<SoundEvent>, String> {
    let stream = rodio::OutputStream::try_default()
        .map_err(|e| format!("не удалось открыть аудиоустройство: {e}"))?;

    let (tx, rx) = mpsc::channel::<SoundEvent>();
    thread::spawn(move || {
        let _keep_alive = stream; // без этого поток закроется
        for event in rx {
            if let Err(err) = play_via_rodio(&stream.1, event) {
                eprintln!("звук: {err}, пробую paplay");
                spawn_paplay(event);
            }
        }
    });
    Ok(tx)
}

fn play_via_rodio(handle: &rodio::OutputStreamHandle, event: SoundEvent) -> Result<(), String> {
    let path = file_for(event);
    let file = std::fs::File::open(path).map_err(|e| format!("не открыть {path}: {e}"))?;
    let source = rodio::Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
    let sink = rodio::Sink::try_new(handle).map_err(|e| e.to_string())?;
    sink.set_volume(volume_for(event));
    sink.append(source);
    sink.detach(); // воспроизведение продолжится после выхода из функции
    Ok(())
}

fn spawn_paplay(event: SoundEvent) {
    let _ = Command::new("paplay").arg(file_for(event)).spawn();
}
