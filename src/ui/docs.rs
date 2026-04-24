use crate::ui::highlighter::Highlighter;
use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;
use serde::Deserialize;

#[derive(PartialEq, Clone, Copy)]
pub enum Chapter {
    QuickReference,
    GettingStarted,
    TypesVariables,
    Operators,
    FunctionsControl,
    Structs,
    Modules,
    StdLib,
    CommonPatterns,
}

impl Chapter {
    fn label(self) -> &'static str {
        match self {
            Self::QuickReference => "Quick Reference",
            Self::GettingStarted => "Getting Started",
            Self::TypesVariables => "Types & Variables",
            Self::Operators => "Operators",
            Self::FunctionsControl => "Control Flow",
            Self::Structs => "Structs",
            Self::Modules => "Modules",
            Self::StdLib => "Standard Library",
            Self::CommonPatterns => "Common Patterns",
        }
    }

    fn id(&self) -> &'static str {
        match self {
            Self::QuickReference => "quick_reference",
            Self::GettingStarted => "getting_started",
            Self::TypesVariables => "types_variables",
            Self::Operators => "operators",
            Self::FunctionsControl => "functions_control",
            Self::Structs => "structs",
            Self::Modules => "modules",
            Self::StdLib => "stdlib",
            Self::CommonPatterns => "common_patterns",
        }
    }
}

const CHAPTERS: &[Chapter] = &[
    Chapter::QuickReference,
    Chapter::GettingStarted,
    Chapter::TypesVariables,
    Chapter::Operators,
    Chapter::FunctionsControl,
    Chapter::Structs,
    Chapter::Modules,
    Chapter::StdLib,
    Chapter::CommonPatterns,
];

#[derive(Debug, Deserialize)]
struct DocsJson {
    chapters: Vec<ChapterData>,
}

#[derive(Debug, Deserialize)]
struct ChapterData {
    id: String,
    sections: Vec<SectionData>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SectionData {
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    table: Option<TableData>,
    #[serde(default)]
    note: Option<NoteData>,
    #[serde(default)]
    subsections: Option<Vec<SectionData>>,
    #[serde(default)]
    subcontent: Option<String>,
    #[serde(default)]
    subcontent2: Option<String>,
    #[serde(default)]
    code2: Option<String>,
    #[serde(default)]
    code3: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TableData {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct NoteData {
    kind: String,
    text: String,
}

fn load_docs_json() -> DocsJson {
    let json_str = include_str!("../docs.json");
    serde_json::from_str(json_str).expect("Failed to parse docs.json")
}

static DOCS_JSON: std::sync::LazyLock<DocsJson> =
    std::sync::LazyLock::new(load_docs_json);

pub struct DocsWindow {
    pub open: bool,
    chapter: Chapter,
    theme: Theme,
    search_query: String,
    search_focused: bool,
}

impl DocsWindow {
    pub fn new(theme: Theme) -> Self {
        Self {
            open: false,
            chapter: Chapter::QuickReference,
            theme,
            search_query: String::new(),
            search_focused: false,
        }
    }

    pub fn update_theme(&mut self, t: Theme) {
        self.theme = t;
    }

    fn get_chapter_data(&self, ch: Chapter) -> Option<&ChapterData> {
        let id = ch.id();
        DOCS_JSON.chapters.iter().find(|c| c.id == id)
    }

    fn chapter_search_text(&self, ch: Chapter) -> String {
        let id = ch.id();
        format!(
            "{} {} {}",
            ch.label(),
            id,
            self.get_chapter_data(ch)
                .map(|c| c.sections.iter().map(|s| s.title.clone()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default()
        )
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        let full = ui.max_rect();

        ui.painter()
            .rect_filled(full, egui::CornerRadius::ZERO, t.panel_bg);

        let sidebar_w = 220.0;
        let sep_x = full.min.x + sidebar_w;

        ui.painter().rect_filled(
            egui::Rect::from_min_max(full.min, egui::pos2(sep_x, full.max.y)),
            egui::CornerRadius::ZERO,
            t.line_numbers_bg,
        );
        ui.painter().line_segment(
            [egui::pos2(sep_x, full.min.y), egui::pos2(sep_x, full.max.y)],
            egui::Stroke::new(1.0, t.border),
        );

        ui.painter().text(
            egui::pos2(full.min.x + 18.0, full.min.y + 20.0),
            egui::Align2::LEFT_CENTER,
            "DOCUMENTATION",
            egui::FontId::proportional(9.5),
            t.tab_inactive_fg,
        );

        let search_rect = egui::Rect::from_min_size(
            egui::pos2(full.min.x + 10.0, full.min.y + 36.0),
            egui::vec2(sidebar_w - 20.0, 28.0),
        );

        let search_stroke = if self.search_focused { t.accent } else { t.border };
        egui::Frame::new()
            .fill(t.button_bg)
            .corner_radius(egui::CornerRadius::same(5))
            .stroke(egui::Stroke::new(1.0, search_stroke))
            .show(
                &mut ui.new_child(egui::UiBuilder::new().max_rect(search_rect)),
                |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(7.0);
                        ui.label(
                            egui::RichText::new(ic::MAGNIFY)
                                .size(11.0)
                                .color(t.tab_inactive_fg),
                        );
                        ui.add_space(4.0);
                        let search_id = egui::Id::new("docs_search_field");
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.search_query)
                                .id(search_id)
                                .hint_text("Search docs...")
                                .frame(false)
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Small),
                        );
                        self.search_focused = resp.has_focus();

                        if !self.search_query.is_empty() {
                            let clear_id = egui::Id::new("docs_search_clear");
                            let clear_rect = egui::Rect::from_center_size(
                                egui::pos2(search_rect.right() - 14.0, search_rect.center().y),
                                egui::vec2(16.0, 16.0),
                            );
                            let clear_resp = ui.interact(clear_rect, clear_id, egui::Sense::click());
                            ui.painter().text(
                                clear_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                ic::TAB_CLOSE,
                                egui::FontId::proportional(10.0),
                                if clear_resp.hovered() {
                                    t.terminal_error
                                } else {
                                    t.tab_inactive_fg
                                },
                            );
                            if clear_resp.clicked() {
                                self.search_query.clear();
                            }
                        }
                    });
                },
            );

        let mut cursor_y = full.min.y + 76.0;
        let query_lower = self.search_query.to_lowercase();
        let searching = !query_lower.is_empty();
        let mut any_result = false;

        for ch in CHAPTERS {
            if searching {
                let search_text = self.chapter_search_text(*ch).to_lowercase();
                let hits = search_text.match_indices(query_lower.as_str()).count();
                if hits == 0 {
                    continue;
                }
                any_result = true;

                let row = egui::Rect::from_min_size(
                    egui::pos2(full.min.x, cursor_y),
                    egui::vec2(sidebar_w, 36.0),
                );
                let id = egui::Id::new(("doc_ch_s", ch.label()));
                let resp = ui.interact(row, id, egui::Sense::click());
                let selected = *ch == self.chapter;
                let hovered = resp.hovered();

                draw_chapter_row(ui, row, selected, hovered, ch.label(), &t);

                let badge_str = format!("{hits}");
                let badge_center = egui::pos2(row.right() - 18.0, row.center().y);
                let badge_rect = egui::Rect::from_center_size(badge_center, egui::vec2(22.0, 16.0));
                ui.painter().rect_filled(
                    badge_rect,
                    egui::CornerRadius::same(8),
                    egui::Color32::from_rgba_premultiplied(t.accent.r(), t.accent.g(), t.accent.b(), 90),
                );
                ui.painter().text(
                    badge_center,
                    egui::Align2::CENTER_CENTER,
                    &badge_str,
                    egui::FontId::proportional(10.0),
                    t.tab_active_fg,
                );

                if resp.clicked() {
                    self.chapter = *ch;
                }
                cursor_y += 36.0;
            } else {
                let row = egui::Rect::from_min_size(
                    egui::pos2(full.min.x, cursor_y),
                    egui::vec2(sidebar_w, 36.0),
                );
                let id = egui::Id::new(("doc_ch", ch.label()));
                let resp = ui.interact(row, id, egui::Sense::click());
                let selected = *ch == self.chapter;
                let hovered = resp.hovered();

                draw_chapter_row(ui, row, selected, hovered, ch.label(), &t);

                if resp.clicked() {
                    self.chapter = *ch;
                }
                cursor_y += 36.0;
            }
        }

        if searching && !any_result {
            ui.painter().text(
                egui::pos2(full.min.x + 18.0, cursor_y + 18.0),
                egui::Align2::LEFT_CENTER,
                "No results",
                egui::FontId::proportional(12.5),
                t.tab_inactive_fg,
            );
        }

        let content_rect = egui::Rect::from_min_max(egui::pos2(sep_x + 1.0, full.min.y), full.max);

        let mut content_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(content_rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
        );

        let search_q = self.search_query.clone();
        let chapter = self.chapter;

        egui::ScrollArea::vertical()
            .id_salt("docs_content_scroll")
            .auto_shrink([false, false])
            .show(&mut content_ui, |ui| {
                let pad = 48.0;
                ui.add_space(32.0);
                let w = (content_rect.width() - pad * 2.0).max(200.0);
                ui.set_max_width(w + pad * 2.0);
                ui.horizontal(|ui| {
                    ui.add_space(pad);
                    ui.vertical(|ui| {
                        ui.set_max_width(w);
                        let t = &self.theme;
                        if !search_q.is_empty() {
                            render_search_header(ui, t, &search_q, chapter);
                        }
                        match chapter {
                            Chapter::QuickReference => render_chapter(ui, t, "quick_reference"),
                            Chapter::GettingStarted => render_chapter(ui, t, "getting_started"),
                            Chapter::TypesVariables => render_chapter(ui, t, "types_variables"),
                            Chapter::Operators => render_chapter(ui, t, "operators"),
                            Chapter::FunctionsControl => render_chapter(ui, t, "functions_control"),
                            Chapter::Structs => render_chapter(ui, t, "structs"),
                            Chapter::Modules => render_chapter(ui, t, "modules"),
                            Chapter::StdLib => render_chapter(ui, t, "stdlib"),
                            Chapter::CommonPatterns => render_chapter(ui, t, "common_patterns"),
                        }
                    });
                });
                ui.add_space(64.0);
            });
    }
}

fn draw_chapter_row(
    ui: &egui::Ui,
    row: egui::Rect,
    selected: bool,
    hovered: bool,
    label: &str,
    t: &Theme,
) {
    let bg = if selected {
        t.button_hover_bg
    } else if hovered {
        egui::Color32::from_rgba_premultiplied(
            t.tab_active_fg.r(),
            t.tab_active_fg.g(),
            t.tab_active_fg.b(),
            12,
        )
    } else {
        egui::Color32::TRANSPARENT
    };

    ui.painter().rect_filled(row, egui::CornerRadius::ZERO, bg);

    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(row.min, egui::vec2(3.0, row.height())),
            egui::CornerRadius::ZERO,
            t.accent,
        );
    }

    let fg = if selected || hovered {
        t.tab_active_fg
    } else {
        t.menu_fg
    };

    ui.painter().text(
        egui::pos2(row.min.x + 18.0, row.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(13.0),
        fg,
    );
}

fn render_search_header(ui: &mut egui::Ui, t: &Theme, query: &str, chapter: Chapter) {
    note(
        ui,
        &format!(
            "Showing chapter \"{}\" - keywords matching \"{}\" highlighted in sidebar.",
            chapter.label(),
            query
        ),
        t,
    );
}

fn render_chapter(ui: &mut egui::Ui, t: &Theme, chapter_id: &str) {
    let ch = match chapter_id {
        "quick_reference" => Chapter::QuickReference,
        "getting_started" => Chapter::GettingStarted,
        "types_variables" => Chapter::TypesVariables,
        "operators" => Chapter::Operators,
        "functions_control" => Chapter::FunctionsControl,
        "structs" => Chapter::Structs,
        "modules" => Chapter::Modules,
        "stdlib" => Chapter::StdLib,
        "common_patterns" => Chapter::CommonPatterns,
        _ => Chapter::QuickReference,
    };

    h1(ui, ch.label(), t);

    if let Some(chapter) = DOCS_JSON.chapters.iter().find(|c| c.id == chapter_id) {
        for section in &chapter.sections {
            render_section(ui, t, section);
        }
    }
}

fn render_section(ui: &mut egui::Ui, t: &Theme, section: &SectionData) {
    rule(ui, t);
    h2(ui, &section.title, t);

    if let Some(desc) = &section.description {
        para(ui, desc, t);
    }

    if let Some(table) = &section.table {
        if table.headers.len() == 2 {
            let rows: Vec<(&str, &str)> = table
                .rows
                .iter()
                .filter_map(|r| {
                    if r.len() >= 2 {
                        Some((r[0].as_str(), r[1].as_str()))
                    } else {
                        None
                    }
                })
                .collect();
            kv2(ui, &section.title, [&table.headers[0], &table.headers[1]], &rows, t);
        } else if table.headers.len() == 3 {
            let rows: Vec<(&str, &str, &str)> = table
                .rows
                .iter()
                .filter_map(|r| {
                    if r.len() >= 3 {
                        Some((r[0].as_str(), r[1].as_str(), r[2].as_str()))
                    } else {
                        None
                    }
                })
                .collect();
            kv3(ui, &section.title, [&table.headers[0], &table.headers[1], &table.headers[2]], &rows, t);
        }
    }

    if let Some(code_snippet) = &section.code {
        code(ui, code_snippet, t);
    }

    if let Some(note_data) = &section.note {
        match note_data.kind.as_str() {
            "warning" => warning(ui, &note_data.text, t),
            _ => note(ui, &note_data.text, t),
        }
    }

    if let Some(subcontent) = &section.subcontent {
        para(ui, subcontent, t);
    }

    if let Some(code2) = &section.code2 {
        code(ui, code2, t);
    }

    if let Some(subcontent2) = &section.subcontent2 {
        para(ui, subcontent2, t);
    }

    if let Some(code3) = &section.code3 {
        code(ui, code3, t);
    }
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

fn h1(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.label(
        egui::RichText::new(text)
            .size(24.0)
            .strong()
            .color(t.tab_active_fg),
    );
    ui.add_space(4.0);
}

fn h2(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(26.0);
    ui.label(
        egui::RichText::new(text)
            .size(14.0)
            .strong()
            .color(t.accent),
    );
    ui.add_space(6.0);
}

#[allow(dead_code)]
fn h3(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(16.0);
    ui.label(
        egui::RichText::new(text)
            .size(12.5)
            .strong()
            .color(t.tab_active_fg),
    );
    ui.add_space(4.0);
}

fn para(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.label(egui::RichText::new(text).size(13.5).color(t.text_default));
    ui.add_space(4.0);
}

fn rule(ui: &mut egui::Ui, t: &Theme) {
    ui.add_space(10.0);
    let (r, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(
        r,
        egui::CornerRadius::ZERO,
        egui::Color32::from_rgba_premultiplied(t.border.r(), t.border.g(), t.border.b(), 120),
    );
    ui.add_space(10.0);
}

fn code(ui: &mut egui::Ui, src: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(t.editor_bg)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let job = Highlighter::new(*t).highlight_to_layout_job(src, egui::FontId::monospace(12.5));
            ui.label(egui::WidgetText::LayoutJob(job.into()));
        });
    ui.add_space(8.0);
}

fn note(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_premultiplied(
            t.border.r(),
            t.border.g(),
            t.border.b(),
            60,
        ))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Note  ")
                        .size(12.5)
                        .strong()
                        .color(t.tab_active_fg),
                );
                ui.label(egui::RichText::new(text).size(13.0).color(t.tab_active_fg));
            });
        });
    ui.add_space(8.0);
}

fn warning(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::new()
        .fill(egui::Color32::from_rgba_premultiplied(
            t.border.r(),
            t.border.g(),
            t.border.b(),
            60,
        ))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new("Warning  ")
                        .size(12.5)
                        .strong()
                        .color(t.tab_active_fg),
                );
                ui.label(egui::RichText::new(text).size(13.0).color(t.tab_active_fg));
            });
        });
    ui.add_space(8.0);
}

fn kv2(ui: &mut egui::Ui, id: &str, cols: [&str; 2], rows: &[(&str, &str)], t: &Theme) {
    egui::Frame::new()
        .fill(t.editor_bg)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new(id)
                .num_columns(2)
                .spacing([32.0, 8.0])
                .show(ui, |ui| {
                    for h in cols {
                        ui.label(egui::RichText::new(h).size(11.5).strong().color(t.accent));
                    }
                    ui.end_row();
                    for (a, b) in rows {
                        ui.label(
                            egui::RichText::new(*a)
                                .monospace()
                                .size(12.5)
                                .color(t.type_name),
                        );
                        ui.label(egui::RichText::new(*b).size(13.0).color(t.text_default));
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}

fn kv3(ui: &mut egui::Ui, id: &str, cols: [&str; 3], rows: &[(&str, &str, &str)], t: &Theme) {
    egui::Frame::new()
        .fill(t.editor_bg)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new(id)
                .num_columns(3)
                .spacing([28.0, 8.0])
                .show(ui, |ui| {
                    for h in cols {
                        ui.label(egui::RichText::new(h).size(11.5).strong().color(t.accent));
                    }
                    ui.end_row();
                    for (a, b, c) in rows {
                        ui.label(
                            egui::RichText::new(*a)
                                .monospace()
                                .size(12.5)
                                .color(t.type_name),
                        );
                        ui.label(egui::RichText::new(*b).size(13.0).color(t.text_default));
                        ui.label(
                            egui::RichText::new(*c)
                                .monospace()
                                .size(12.5)
                                .color(t.number),
                        );
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}

#[allow(dead_code)]
fn shortcuts_table(ui: &mut egui::Ui, id: &str, rows: &[(&str, &str)], t: &Theme) {
    egui::Frame::new()
        .fill(t.editor_bg)
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(16))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            egui::Grid::new(id)
                .num_columns(2)
                .spacing([40.0, 8.0])
                .show(ui, |ui| {
                    for (k, v) in rows {
                        ui.label(
                            egui::RichText::new(*k)
                                .monospace()
                                .size(12.5)
                                .color(t.number),
                        );
                        ui.label(egui::RichText::new(*v).size(13.0).color(t.text_default));
                        ui.end_row();
                    }
                });
        });
    ui.add_space(4.0);
}