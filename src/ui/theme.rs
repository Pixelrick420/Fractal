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
    pub bracket: Color32,
    pub angle_bracket: Color32,
    pub identifier: Color32,
    pub boolean: Color32,
    pub fn_name: Color32,
    pub struct_name: Color32,
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

    pub terminal_error_code: Color32,

    pub terminal_hint: Color32,

    pub terminal_location: Color32,

    pub terminal_gutter: Color32,

    pub terminal_line_num: Color32,

    pub terminal_caret: Color32,

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
        let bg = Color32::from_rgb(13, 17, 23);
        let panel = Color32::from_rgb(22, 27, 34);
        let tab_bar = Color32::from_rgb(13, 17, 23);
        let border = Color32::from_rgb(48, 54, 61);
        let accent = Color32::from_rgb(88, 166, 255);

        Self {
            variant: ThemeVariant::Dark,
            editor_bg: bg,
            text_default: Color32::from_rgb(201, 209, 217),
            keyword: Color32::from_rgb(255, 123, 114),
            type_name: Color32::from_rgb(121, 192, 255),
            number: Color32::from_rgb(240, 173, 57),
            string: Color32::from_rgb(165, 214, 255),
            char_lit: Color32::from_rgb(165, 214, 255),
            comment: Color32::from_rgb(139, 148, 158),
            operator: Color32::from_rgb(201, 209, 217),
            bracket: Color32::from_rgb(201, 209, 217),
            angle_bracket: Color32::from_rgb(139, 148, 158),
            identifier: Color32::from_rgb(201, 209, 217),
            boolean: Color32::from_rgb(255, 123, 114),
            fn_name: Color32::from_rgb(210, 168, 255),
            struct_name: Color32::from_rgb(126, 231, 135),
            line_numbers_bg: panel,
            line_numbers_fg: Color32::from_rgb(48, 54, 61),
            panel_bg: panel,
            tab_bar_bg: tab_bar,
            tab_active_bg: bg,
            tab_inactive_bg: tab_bar,
            tab_active_fg: Color32::from_rgb(201, 209, 217),
            tab_inactive_fg: Color32::from_rgb(139, 148, 158),
            tab_dirty_dot: Color32::from_rgb(240, 173, 57),
            border,
            accent,
            button_bg: Color32::from_rgb(33, 38, 45),
            button_fg: Color32::from_rgb(201, 209, 217),
            button_hover_bg: Color32::from_rgb(48, 54, 61),
            status_bar_bg: Color32::from_rgb(13, 17, 23),
            status_bar_fg: Color32::from_rgb(139, 148, 158),
            menu_bg: Color32::from_rgb(22, 27, 34),
            menu_fg: Color32::from_rgb(201, 209, 217),
            menu_hover_bg: Color32::from_rgb(33, 38, 45),
            selection: Color32::from_rgba_unmultiplied(56, 139, 253, 90),
            terminal_bg: Color32::from_rgb(13, 17, 23),
            terminal_fg: Color32::from_rgb(201, 209, 217),
            terminal_error: Color32::from_rgb(255, 123, 114),
            terminal_warning: Color32::from_rgb(210, 153, 34),
            terminal_error_code: Color32::from_rgb(255, 90, 70),
            terminal_hint: Color32::from_rgb(86, 182, 194),
            terminal_location: Color32::from_rgb(150, 175, 255),
            terminal_gutter: Color32::from_rgb(68, 76, 86),
            terminal_line_num: Color32::from_rgb(100, 110, 122),
            terminal_caret: Color32::from_rgb(240, 173, 57),
            empty_fg: Color32::from_rgb(48, 54, 61),
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
            number: Color32::from_rgb(9, 134, 88),
            string: Color32::from_rgb(180, 60, 30),
            char_lit: Color32::from_rgb(180, 60, 30),
            comment: Color32::from_rgb(112, 128, 100),
            operator: Color32::from_rgb(80, 88, 110),
            bracket: Color32::from_rgb(80, 88, 110),
            angle_bracket: Color32::from_rgb(140, 148, 170),
            identifier: Color32::from_rgb(18, 80, 180),
            boolean: Color32::from_rgb(18, 80, 180),
            fn_name: Color32::from_rgb(111, 66, 193),
            struct_name: Color32::from_rgb(0, 112, 96),
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
            selection: Color32::from_rgb(173, 214, 255),
            terminal_bg: Color32::from_rgb(232, 235, 245),
            terminal_fg: Color32::from_rgb(35, 42, 62),
            terminal_error: Color32::from_rgb(192, 28, 28),
            terminal_warning: Color32::from_rgb(148, 90, 0),
            terminal_error_code: Color32::from_rgb(180, 20, 20),
            terminal_hint: Color32::from_rgb(0, 120, 140),
            terminal_location: Color32::from_rgb(60, 80, 200),
            terminal_gutter: Color32::from_rgb(170, 176, 198),
            terminal_line_num: Color32::from_rgb(130, 138, 165),
            terminal_caret: Color32::from_rgb(148, 90, 0),
            empty_fg: Color32::from_rgb(148, 156, 180),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
