use eframe::egui;

pub(super) const TOOLBAR_BLUE: egui::Color32 = egui::Color32::from_rgb(45, 75, 115);
pub(super) const TOOLBAR_STROKE: egui::Color32 = egui::Color32::from_rgb(80, 105, 145);
pub(super) const LOCKED_COLOR: egui::Color32 = egui::Color32::from_rgb(45, 115, 75);
pub(super) const UNLOCKED_COLOR: egui::Color32 = egui::Color32::from_rgb(125, 55, 60);

pub(super) fn toolbar_button(
    ui: &mut egui::Ui,
    text: &str,
    width: f32,
    color: egui::Color32,
) -> egui::Response {
    ui.add_sized(
        [width, 34.0],
        egui::Button::new(
            egui::RichText::new(text)
                .size(15.0)
                .color(egui::Color32::WHITE),
        )
        .fill(color)
        .stroke(egui::Stroke::new(1.0, TOOLBAR_STROKE)),
    )
}
