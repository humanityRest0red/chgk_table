mod models;
mod ui;
mod utils;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_position([100.0, 100.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Таблица «Что? Где? Когда?»",
        options,
        Box::new(|_creation_context| Ok(Box::new(ui::chgk_app::ChgkApp::default()))),
    )
}
