use egui::Color32;

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ThemeVariant {
    Dark,
    Light,
}

impl ThemeVariant {
    pub fn label(&self) -> &'static str {
        match self {
            ThemeVariant::Dark => "Dark",
            ThemeVariant::Light => "Light",
        }
    }

    pub fn all() -> &'static [ThemeVariant] {
        &[ThemeVariant::Dark, ThemeVariant::Light]
    }
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub variant: ThemeVariant,

    pub editor_bg: Color32,
    pub text_default: Color32,
    pub keyword: Color32,
    pub type_name: Color32,
    pub number: Color32,
    pub string: Color32,
    pub char_lit: Color32,
    pub comment: Color32,
    pub operator: Color32,
    pub identifier: Color32,
    pub boolean: Color32,
    pub line_numbers_bg: Color32,
    pub line_numbers_fg: Color32,

    pub panel_bg: Color32,
    pub tab_bar_bg: Color32,
    pub tab_active_bg: Color32,
    pub tab_inactive_bg: Color32,
    pub tab_active_fg: Color32,
    pub tab_inactive_fg: Color32,
    pub tab_dirty_dot: Color32,
    pub border: Color32,
    pub accent: Color32,
    pub button_bg: Color32,
    pub button_fg: Color32,
    pub button_hover_bg: Color32,
    pub status_bar_bg: Color32,
    pub status_bar_fg: Color32,
    pub menu_bg: Color32,
    pub menu_fg: Color32,
    pub menu_hover_bg: Color32,
    pub selection: Color32,

    pub terminal_bg: Color32,
    pub terminal_fg: Color32,
    pub terminal_error: Color32,
    pub terminal_warning: Color32,

    pub empty_fg: Color32,
}

impl Theme {
    pub fn from_variant(v: ThemeVariant) -> Self {
        match v {
            ThemeVariant::Dark => Self::dark(),
            ThemeVariant::Light => Self::light(),
        }
    }

    pub fn dark() -> Self {
        let bg = Color32::from_rgb(13, 16, 23);
        let panel = Color32::from_rgb(10, 13, 20);
        let tab_bar = Color32::from_rgb(9, 12, 18);
        let border = Color32::from_rgb(30, 36, 52);
        let accent = Color32::from_rgb(89, 163, 255);

        Self {
            variant: ThemeVariant::Dark,
            editor_bg: bg,
            text_default: Color32::from_rgb(203, 210, 220),
            keyword: Color32::from_rgb(255, 140, 0),
            type_name: Color32::from_rgb(86, 212, 195),
            number: Color32::from_rgb(214, 177, 95),
            string: Color32::from_rgb(163, 207, 98),
            char_lit: Color32::from_rgb(163, 207, 98),
            comment: Color32::from_rgb(75, 83, 98),
            operator: Color32::from_rgb(236, 197, 120),
            identifier: Color32::from_rgb(203, 210, 220),
            boolean: Color32::from_rgb(255, 140, 0),
            line_numbers_bg: panel,

            line_numbers_fg: Color32::from_rgb(78, 90, 116),
            panel_bg: panel,
            tab_bar_bg: tab_bar,
            tab_active_bg: bg,
            tab_inactive_bg: tab_bar,
            tab_active_fg: Color32::from_rgb(203, 210, 220),

            tab_inactive_fg: Color32::from_rgb(100, 112, 138),
            tab_dirty_dot: Color32::from_rgb(236, 197, 120),
            border,
            accent,
            button_bg: Color32::from_rgb(22, 28, 42),
            button_fg: Color32::from_rgb(160, 172, 196),
            button_hover_bg: Color32::from_rgb(34, 42, 62),
            status_bar_bg: Color32::from_rgb(7, 9, 14),
            status_bar_fg: Color32::from_rgb(100, 112, 138),

            menu_bg: Color32::from_rgb(20, 25, 38),
            menu_fg: Color32::from_rgb(200, 208, 224),
            menu_hover_bg: Color32::from_rgb(34, 42, 62),
            selection: Color32::from_rgba_premultiplied(89, 163, 255, 50),
            terminal_bg: Color32::from_rgb(8, 10, 16),
            terminal_fg: Color32::from_rgb(192, 200, 216),
            terminal_error: Color32::from_rgb(240, 100, 90),
            terminal_warning: Color32::from_rgb(236, 197, 120),
            empty_fg: Color32::from_rgb(72, 84, 108),
        }
    }

    pub fn light() -> Self {
        let bg = Color32::from_rgb(250, 250, 252);
        let panel = Color32::from_rgb(238, 240, 246);
        let tab_bar = Color32::from_rgb(226, 229, 238);
        let border = Color32::from_rgb(200, 204, 218);
        let accent = Color32::from_rgb(64, 120, 242);

        Self {
            variant: ThemeVariant::Light,
            editor_bg: bg,
            text_default: Color32::from_rgb(35, 38, 50),
            keyword: Color32::from_rgb(160, 60, 200),
            type_name: Color32::from_rgb(0, 130, 110),
            number: Color32::from_rgb(10, 130, 70),
            string: Color32::from_rgb(180, 60, 30),
            char_lit: Color32::from_rgb(180, 60, 30),
            comment: Color32::from_rgb(112, 128, 100),
            operator: Color32::from_rgb(80, 88, 110),
            identifier: Color32::from_rgb(18, 80, 180),
            boolean: Color32::from_rgb(18, 80, 180),
            line_numbers_bg: panel,
            line_numbers_fg: Color32::from_rgb(155, 162, 185),
            panel_bg: panel,
            tab_bar_bg: tab_bar,
            tab_active_bg: bg,
            tab_inactive_bg: tab_bar,
            tab_active_fg: Color32::from_rgb(30, 34, 48),
            tab_inactive_fg: Color32::from_rgb(105, 114, 140),
            tab_dirty_dot: Color32::from_rgb(200, 120, 20),
            border,
            accent,

            button_bg: Color32::from_rgb(220, 224, 235),
            button_fg: Color32::from_rgb(35, 42, 62),
            button_hover_bg: Color32::from_rgb(204, 210, 228),
            status_bar_bg: Color32::from_rgb(210, 214, 228),
            status_bar_fg: Color32::from_rgb(80, 90, 118),

            menu_bg: Color32::from_rgb(248, 249, 253),
            menu_fg: Color32::from_rgb(28, 34, 54),
            menu_hover_bg: Color32::from_rgb(210, 216, 234),
            selection: Color32::from_rgba_premultiplied(64, 120, 242, 50),

            terminal_bg: Color32::from_rgb(232, 235, 245),
            terminal_fg: Color32::from_rgb(35, 42, 62),
            terminal_error: Color32::from_rgb(192, 28, 28),
            terminal_warning: Color32::from_rgb(148, 90, 0),

            empty_fg: Color32::from_rgb(148, 156, 180),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
