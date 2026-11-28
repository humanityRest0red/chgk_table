#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod ui;
mod utils;

fn main() -> eframe::Result<()> {
    // #[cfg(target_os = "linux")]
    // {
    //     if std::env::var_os("WINIT_UNIX_BACKEND").is_none() {
    //         unsafe { std::env::set_var("WINIT_UNIX_BACKEND", "x11") };
    //     }
    // }

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/klio.png"))
        .expect("не удалось загрузить иконку");
    
    const WIN_SIZE: [f32; 2] = [1200.0, 800.0];

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_position([100.0, 100.0])
            .with_icon(std::sync::Arc::new(icon))
            // Обход бага winit: окна с resizable(false) на Windows
            // не перетаскиваются за заголовок. Оставляем resizable,
            // но фиксируем размер равными min и max.
            .with_resizable(true),
            // .with_min_inner_size(WIN_SIZE)
            // .with_max_inner_size(WIN_SIZE),
        ..Default::default()
    };

    eframe::run_native(
        "Таблица «Что? Где? Когда?»",
        options,
        Box::new(|_creation_context| Ok(Box::new(ui::chgk_app::ChgkApp::default()))),
    )
}
