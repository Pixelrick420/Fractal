use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;

struct Span {
    text: String,
    color: egui::Color32,
    bold: bool,
}

fn parse_ansi(line: &str, default_color: egui::Color32) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();

    let mut pos = 0;
    let mut cur_color = default_color;
    let mut bold = false;
    let mut text_start = 0;

    macro_rules! flush {
        ($end:expr) => {
            if text_start < $end {
                let text = line[text_start..$end].to_owned();
                if !text.is_empty() {
                    spans.push(Span {
                        text,
                        color: cur_color,
                        bold,
                    });
                }
            }
        };
    }

    while pos < len {
        if bytes[pos] != 0x1B {
            pos += 1;
            continue;
        }

        flush!(pos);

        if pos + 1 >= len || bytes[pos + 1] != b'[' {
            pos += 1;
            text_start = pos;
            continue;
        }

        let seq_start = pos + 2;
        let mut seq_end = seq_start;
        while seq_end < len && !bytes[seq_end].is_ascii_alphabetic() {
            seq_end += 1;
        }

        if seq_end < len && bytes[seq_end] == b'm' {
            let params_str = &line[seq_start..seq_end];
            apply_sgr(params_str, default_color, &mut cur_color, &mut bold);
        }

        pos = seq_end + 1;
        text_start = pos;
    }

    flush!(pos);
    spans
}

fn apply_sgr(
    params: &str,
    default_color: egui::Color32,
    cur_color: &mut egui::Color32,
    bold: &mut bool,
) {
    let mut parts = params.split(';').filter_map(|s| s.parse::<u8>().ok());

    while let Some(code) = parts.next() {
        match code {
            0 => {
                *cur_color = default_color;
                *bold = false;
            }
            1 => *bold = true,
            22 => *bold = false,
            39 => *cur_color = default_color,

            30 => *cur_color = egui::Color32::from_rgb(64, 64, 64),
            31 => *cur_color = egui::Color32::from_rgb(204, 0, 0),
            32 => *cur_color = egui::Color32::from_rgb(0, 170, 0),
            33 => *cur_color = egui::Color32::from_rgb(170, 85, 0),
            34 => *cur_color = egui::Color32::from_rgb(0, 85, 204),
            35 => *cur_color = egui::Color32::from_rgb(170, 0, 170),
            36 => *cur_color = egui::Color32::from_rgb(0, 170, 170),
            37 => *cur_color = egui::Color32::from_rgb(192, 192, 192),

            38 => match parts.next() {
                Some(5) => {
                    if let Some(idx) = parts.next() {
                        *cur_color = ansi_256_color(idx);
                    }
                }
                Some(2) => {
                    let r = parts.next().unwrap_or(0);
                    let g = parts.next().unwrap_or(0);
                    let b = parts.next().unwrap_or(0);
                    *cur_color = egui::Color32::from_rgb(r, g, b);
                }
                _ => {}
            },

            90 => *cur_color = egui::Color32::from_rgb(128, 128, 128),
            91 => *cur_color = egui::Color32::from_rgb(255, 85, 85),
            92 => *cur_color = egui::Color32::from_rgb(85, 255, 85),
            93 => *cur_color = egui::Color32::from_rgb(255, 255, 85),
            94 => *cur_color = egui::Color32::from_rgb(85, 85, 255),
            95 => *cur_color = egui::Color32::from_rgb(255, 85, 255),
            96 => *cur_color = egui::Color32::from_rgb(85, 255, 255),
            97 => *cur_color = egui::Color32::WHITE,

            _ => {}
        }
    }
}

fn ansi_256_color(idx: u8) -> egui::Color32 {
    match idx {
        0 => egui::Color32::from_rgb(0, 0, 0),
        1 => egui::Color32::from_rgb(204, 0, 0),
        2 => egui::Color32::from_rgb(0, 170, 0),
        3 => egui::Color32::from_rgb(170, 85, 0),
        4 => egui::Color32::from_rgb(0, 85, 204),
        5 => egui::Color32::from_rgb(170, 0, 170),
        6 => egui::Color32::from_rgb(0, 170, 170),
        7 => egui::Color32::from_rgb(192, 192, 192),
        8 => egui::Color32::from_rgb(128, 128, 128),
        9 => egui::Color32::from_rgb(255, 85, 85),
        10 => egui::Color32::from_rgb(85, 255, 85),
        11 => egui::Color32::from_rgb(255, 255, 85),
        12 => egui::Color32::from_rgb(85, 85, 255),
        13 => egui::Color32::from_rgb(255, 85, 255),
        14 => egui::Color32::from_rgb(85, 255, 255),
        15 => egui::Color32::WHITE,

        16..=231 => {
            let i = idx - 16;
            let b = (i % 6) * 51;
            let g = ((i / 6) % 6) * 51;
            let r = (i / 36) * 51;
            egui::Color32::from_rgb(r, g, b)
        }

        232..=255 => {
            let v = 8 + (idx - 232) * 10;
            egui::Color32::from_rgb(v, v, v)
        }
    }
}

fn has_ansi(line: &str) -> bool {
    line.contains('\x1B')
}

#[derive(Clone, Copy, PartialEq)]
enum LineKind {
    ErrorHeader,
    WarnHeader,
    NoteOrHelp,
    Location,
    Gutter,
    SubNote,
    Plain,
}

fn classify(line: &str) -> LineKind {
    let trimmed = line.trim_start();
    if line.starts_with("error") {
        return LineKind::ErrorHeader;
    }
    if line.starts_with("warning") {
        return LineKind::WarnHeader;
    }
    if line.starts_with("note:") || line.starts_with("help:") {
        return LineKind::NoteOrHelp;
    }
    if trimmed.starts_with("--> ") {
        return LineKind::Location;
    }
    if trimmed.starts_with("= note:") || trimmed.starts_with("= help:") {
        return LineKind::SubNote;
    }
    let maybe_gutter = trimmed
        .find('|')
        .map(|i| trimmed[..i].trim().chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false);
    if maybe_gutter {
        return LineKind::Gutter;
    }
    LineKind::Plain
}

struct Segment {
    text: String,
    color: egui::Color32,
}

fn seg(text: impl Into<String>, color: egui::Color32) -> Segment {
    Segment {
        text: text.into(),
        color,
    }
}

fn segments_for_line(line: &str, kind: LineKind, t: &Theme) -> Vec<Segment> {
    match kind {
        LineKind::ErrorHeader => segments_kw(line, t.terminal_error, t),
        LineKind::WarnHeader => segments_kw(line, t.terminal_warning, t),
        LineKind::NoteOrHelp => {
            let (kw, rest) = split_at_colon(line);
            vec![seg(kw, t.terminal_hint), seg(rest, t.terminal_fg)]
        }
        LineKind::Location => {
            if let Some(i) = line.find("-->") {
                vec![
                    seg(&line[..i + 3], t.terminal_gutter),
                    seg(&line[i + 3..], t.terminal_location),
                ]
            } else {
                vec![seg(line, t.terminal_location)]
            }
        }
        LineKind::Gutter => {
            if let Some(p) = line.find('|') {
                let content = &line[p + 1..];
                let is_caret = !content.trim().is_empty()
                    && content
                        .trim()
                        .chars()
                        .all(|c| matches!(c, '^' | '-' | '+' | '~' | ' '));
                vec![
                    seg(&line[..p], t.terminal_line_num),
                    seg("|", t.terminal_gutter),
                    seg(
                        content,
                        if is_caret {
                            t.terminal_caret
                        } else {
                            t.terminal_fg
                        },
                    ),
                ]
            } else {
                vec![seg(line, t.terminal_gutter)]
            }
        }
        LineKind::SubNote => {
            if let Some(eq) = line.find('=') {
                let rest = line[eq + 1..].trim_start();
                let (kw, msg) = split_at_colon(rest);
                vec![
                    seg(&line[..eq + 1], t.terminal_gutter),
                    seg(" ", t.terminal_gutter),
                    seg(kw, t.terminal_hint),
                    seg(msg, t.terminal_fg),
                ]
            } else {
                vec![seg(line, t.terminal_hint)]
            }
        }
        LineKind::Plain => vec![seg(line, t.terminal_fg)],
    }
}

fn segments_kw(line: &str, kw_color: egui::Color32, t: &Theme) -> Vec<Segment> {
    let kw_end = line
        .find(|c: char| !c.is_alphabetic())
        .unwrap_or(line.len());
    let keyword = &line[..kw_end];
    let rest = &line[kw_end..];
    if rest.starts_with('[') {
        if let Some(close) = rest.find(']') {
            return vec![
                seg(keyword, kw_color),
                seg(&rest[..=close], t.terminal_error_code),
                seg(&rest[close + 1..], t.terminal_fg),
            ];
        }
    }
    vec![seg(keyword, kw_color), seg(rest, t.terminal_fg)]
}

fn split_at_colon(s: &str) -> (&str, &str) {
    if let Some(i) = s.find(':') {
        (&s[..i + 1], &s[i + 1..])
    } else {
        (s, "")
    }
}

pub struct Terminal {
    pub output: String,
    pub minimized: bool,
    saved_height: f32,
    theme: Theme,
}

impl Terminal {
    pub fn new(theme: Theme) -> Self {
        Self {
            output: String::new(),
            minimized: false,
            saved_height: 200.0,
            theme,
        }
    }

    pub fn update_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn append(&mut self, text: &str) {
        self.output.push_str(text);
        self.minimized = false;
    }

    pub fn clear(&mut self) {
        self.output.clear();
    }

    pub fn toggle_minimized(&mut self) {
        self.minimized = !self.minimized;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let t = self.theme;
        let header_h = 32.0;

        let frame = egui::Frame::none()
            .fill(t.terminal_bg)
            .inner_margin(egui::Margin {
                left: 12.0,
                right: 12.0,
                top: 0.0,
                bottom: 0.0,
            });

        if self.minimized {
            let resp = egui::TopBottomPanel::bottom("terminal_panel_mini")
                .frame(frame)
                .resizable(false)
                .exact_height(header_h)
                .show_separator_line(false)
                .show(ctx, |ui| self.draw_header(ui, t));
            self.draw_border(ctx, resp.response.rect, t);
        } else {
            let resp = egui::TopBottomPanel::bottom("terminal_panel_expanded")
                .frame(frame)
                .resizable(true)
                .min_height(80.0)
                .max_height(600.0)
                .default_height(self.saved_height)
                .show_separator_line(false)
                .show(ctx, |ui| {
                    self.draw_header(ui, t);
                    ui.add_space(3.0);
                    self.draw_output(ui, t);
                });
            let h = resp.response.rect.height();
            if h >= 80.0 {
                self.saved_height = h;
            }
            self.draw_border(ctx, resp.response.rect, t);
        }
    }

    fn draw_header(&mut self, ui: &mut egui::Ui, t: Theme) {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            let toggle_icon = if self.minimized {
                ic::TERM_EXPAND
            } else {
                ic::TERM_COLLAPSE
            };
            let toggle_resp = ui.add(
                egui::Button::new(
                    egui::RichText::new(toggle_icon)
                        .size(13.0)
                        .color(t.tab_inactive_fg),
                )
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE),
            );
            if toggle_resp.hovered() {
                ui.painter().rect_filled(
                    toggle_resp.rect.expand(2.0),
                    egui::Rounding::same(4.0),
                    t.button_hover_bg,
                );
                ui.painter().text(
                    toggle_resp.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    toggle_icon,
                    egui::FontId::proportional(13.0),
                    t.tab_active_fg,
                );
            }
            if toggle_resp.clicked() {
                self.toggle_minimized();
            }

            ui.label(
                egui::RichText::new(format!("{}  TERMINAL", ic::TERMINAL))
                    .size(11.0)
                    .color(t.tab_inactive_fg)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let clear_id = egui::Id::new("terminal_clear_btn");
                let (clear_rect, _) =
                    ui.allocate_exact_size(egui::vec2(64.0, 24.0), egui::Sense::hover());
                let clear_hovered = ui.rect_contains_pointer(clear_rect);
                ui.painter().text(
                    clear_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{}  Clear", ic::TERM_CLEAR),
                    egui::FontId::proportional(11.5),
                    if clear_hovered {
                        t.terminal_error
                    } else {
                        t.tab_inactive_fg
                    },
                );
                if ui
                    .interact(clear_rect, clear_id, egui::Sense::click())
                    .clicked()
                {
                    self.output.clear();
                }
            });
        });
    }

    fn draw_output(&self, ui: &mut egui::Ui, t: Theme) {
        egui::Frame::none()
            .fill(t.terminal_bg)
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());

                        if self.output.is_empty() {
                            ui.label(
                                egui::RichText::new(
                                    "No output yet. Press Run to compile and execute.",
                                )
                                .size(12.0)
                                .color(
                                    egui::Color32::from_rgba_unmultiplied(
                                        t.terminal_fg.r(),
                                        t.terminal_fg.g(),
                                        t.terminal_fg.b(),
                                        140,
                                    ),
                                ),
                            );
                        } else {
                            for line in self.output.lines() {
                                self.draw_line(ui, line, &t);
                            }
                        }
                    });
            });
    }

    fn draw_line(&self, ui: &mut egui::Ui, line: &str, t: &Theme) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;

            if has_ansi(line) {
                for span in &parse_ansi(line, t.terminal_fg) {
                    if span.text.is_empty() {
                        continue;
                    }
                    let rt = egui::RichText::new(&span.text)
                        .size(12.5)
                        .color(span.color)
                        .monospace();
                    ui.label(if span.bold { rt.strong() } else { rt });
                }
            } else {
                let kind = classify(line);
                for s in &segments_for_line(line, kind, t) {
                    if s.text.is_empty() {
                        continue;
                    }
                    ui.label(
                        egui::RichText::new(&s.text)
                            .size(12.5)
                            .color(s.color)
                            .monospace(),
                    );
                }
            }
        });
    }

    fn draw_border(&self, ctx: &egui::Context, rect: egui::Rect, t: Theme) {
        ctx.layer_painter(egui::LayerId::background()).line_segment(
            [rect.left_top(), rect.right_top()],
            egui::Stroke::new(1.0, t.border),
        );
    }
}
