use eframe::egui;
use super::theme::palette;

pub(super) fn toolbar_button_colored(
    ui: &mut egui::Ui,
    text: &str,
    width: f32,
    fill: egui::Color32,
) -> egui::Response {
    let p = palette(ui.visuals().dark_mode);
    styled_button(ui, text, [width, 32.0], fill, p.header_text)
}

pub(super) fn toolbar_button(ui: &mut egui::Ui, text: &str, width: f32) -> egui::Response {
    let p = palette(ui.visuals().dark_mode);
    styled_button(ui, text, [width, 32.0], p.header, p.header_text)
}

fn styled_button(
    ui: &mut egui::Ui,
    label: &str,
    size: [f32; 2],
    fill: egui::Color32,
    text_color: egui::Color32,
) -> egui::Response {
    ui.add_sized(
        size,
        egui::Button::new(
            egui::RichText::new(label)
                .strong()
                .color(text_color),
        )
        .fill(fill)
        .corner_radius(egui::CornerRadius::same(10)),
    )
}
