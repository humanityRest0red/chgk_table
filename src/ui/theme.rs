use eframe::egui::Color32;
use eframe::egui;

pub(super) const TEAM_WIDTH: f32 = 220.0;
pub(super) const TOTAL_WIDTH: f32 = 90.0;
pub(super) const RATING_WIDTH: f32 = 42.0;
pub(super) const QUESTION_WIDTH: f32 = 42.0;
pub(super) const ROUND_TOTAL_WIDTH: f32 = 72.0;
pub(super) const CELL_HEIGHT: f32 = 40.0;
pub(super) const WIDTH_PADDING: f32 = 65.0;
pub(super) const HEIGHT_PADDING: f32 = 55.0;


#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color32,   // фон окна
    pub surface: Color32,      // фон панелей, карточка настроек
    pub card: Color32,         // вложенные карточки, поля ввода, пустые клетки
    pub border: Color32,       // тонкие границы
    pub text: Color32,         // основной текст
    pub text_muted: Color32,   // вторичный текст
    pub accent: Color32,       // жёлтая кнопка «Применить изменения»
    pub accent_text: Color32,  // текст НА жёлтой кнопке
    pub header: Color32,       // шапки таблиц и тулбар (золото вместо синего)
    pub header_text: Color32,  // текст на шапках/кнопках тулбара
    pub total: Color32,        // колонки «Итог»/«R», итоги туров
    pub success: Color32,      // клетки «+», роли «Б»
    pub danger: Color32,       // клетки «−», блокировка ввода
    pub question: Color32,     // клетки «?»
}

pub fn palette(dark_mode: bool) -> Palette {
    if dark_mode {
        Palette {
            background:   Color32::from_rgb(16, 16, 18),
            surface:      Color32::from_rgb(40, 41, 44),
            card:         Color32::from_rgb(52, 53, 56),
            border:       Color32::from_rgb(66, 67, 70),
            text:         Color32::from_rgb(238, 239, 242),
            text_muted:   Color32::from_rgb(155, 158, 165),
            accent:       Color32::from_rgb(209, 186, 142), // #D1BA8E — жёлтая кнопка с макета
            accent_text:  Color32::from_rgb(43, 36, 22),    // тёмный текст на жёлтом
            header:       Color32::from_rgb(201, 159, 110),   // коричневый
            header_text:  Color32::from_rgb(20, 15, 8),     // чёрный текст на коричневом
            // total:        Color32::from_rgb(52, 53, 56),
            total:        Color32::from_rgb(201, 159, 110),
            success:      Color32::from_rgb(55, 125, 85),
            danger:       Color32::from_rgb(170, 75, 80),
            question:     Color32::from_rgb(220, 175, 65),
        }
    } else {
        Palette {
            background:   Color32::from_rgb(244, 242, 238), // тёплый айвори
            surface:      Color32::from_rgb(255, 255, 255),
            card:         Color32::from_rgb(250, 249, 246),
            border:       Color32::from_rgb(219, 214, 205),
            text:         Color32::from_rgb(30, 28, 24),
            text_muted:   Color32::from_rgb(140, 135, 128),
            accent:       Color32::from_rgb(10, 39, 71),    // синий NAVY
            accent_text:  Color32::WHITE,                   // белый текст на синем
            header:       Color32::from_rgb(10, 39, 71),    // синий NAVY
            header_text:  Color32::WHITE,                   // белый текст на синем

            // accent:       Color32::WHITE, // тот же NAVY, что у шапок
            // accent_text:  Color32::from_rgb(10, 39, 91),
            // header:       Color32::WHITE,    // синий NAVY, как было изначально
            // header_text:  Color32::from_rgb(10, 39, 91),                   // белый текст на синем
            // total:        Color32::from_rgb(75, 15, 45), // burgundy
            total:        Color32::from_rgb(10, 39, 71),

            // success:      Color32::from_rgb(60, 140, 95),
            // danger:       Color32::from_rgb(200, 90, 90),
            // question:     Color32::from_rgb(235, 190, 80),
            
            
            success:   Color32::from_rgb(45, 106, 79),    // #2D6A4F — хвойный, глубокий
            danger:    Color32::from_rgb(170, 74, 68),    // #AA4A44 — терракота, не алый
            question:  Color32::from_rgb(212, 160, 60), 
        }
    }
}

pub fn apply_theme(ctx: &egui::Context) {
    for &dark in &[true, false] {
        let p = palette(dark);
        let theme = if dark { egui::Theme::Dark } else { egui::Theme::Light };

        let mut v = ctx.style_of(theme).visuals.clone();

        v.panel_fill = p.background;
        v.window_fill = p.surface;
        v.extreme_bg_color = p.card;
        v.override_text_color = Some(p.text);

        for (state, bg, fg) in [
            (&mut v.widgets.noninteractive, p.card, p.text),
            (&mut v.widgets.inactive, p.card, p.text),
            (&mut v.widgets.hovered, p.border, p.text),
            (&mut v.widgets.active, p.accent, p.accent_text),
        ] {
            state.bg_fill = bg;
            state.fg_stroke.color = fg;
            state.bg_stroke.color = p.border;
        }

        v.selection.bg_fill = p.accent;
        v.selection.stroke.color = p.accent_text;

        ctx.set_visuals_of(theme, v);
    }
}
