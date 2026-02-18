use egui::Color32;

#[derive(Clone, Copy)]
pub struct Theme {
    pub background: Color32,
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
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background:      Color32::from_rgb(30,  30,  30),
            text_default:    Color32::from_rgb(212, 212, 212),
            keyword:         Color32::from_rgb(197, 134, 192),
            type_name:       Color32::from_rgb(78,  201, 176),
            number:          Color32::from_rgb(181, 206, 168),
            string:          Color32::from_rgb(206, 145, 120),
            char_lit:        Color32::from_rgb(209, 105, 105),
            comment:         Color32::from_rgb(106, 153, 85),
            operator:        Color32::from_rgb(212, 212, 212),
            identifier:      Color32::from_rgb(156, 220, 254),
            boolean:         Color32::from_rgb(86,  156, 214),
            line_numbers_bg: Color32::from_rgb(25,  25,  25),
            line_numbers_fg: Color32::from_rgb(100, 100, 100),
        }
    }
}
