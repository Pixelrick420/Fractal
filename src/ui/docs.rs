use crate::ui::highlighter::Highlighter;
use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;

#[derive(PartialEq, Clone, Copy)]
enum Chapter {
    GettingStarted,
    TypesVariables,
    Operators,
    FunctionsControl,
    Structs,
    StdLib,
}

impl Chapter {
    fn label(self) -> &'static str {
        match self {
            Self::GettingStarted => "Getting Started",
            Self::TypesVariables => "Types & Variables",
            Self::Operators => "Operators",
            Self::FunctionsControl => "Control Flow",
            Self::Structs => "Structs",
            Self::StdLib => "Standard Library",
        }
    }
}

const CHAPTERS: &[Chapter] = &[
    Chapter::GettingStarted,
    Chapter::TypesVariables,
    Chapter::Operators,
    Chapter::FunctionsControl,
    Chapter::Structs,
    Chapter::StdLib,
];

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
            chapter: Chapter::GettingStarted,
            theme,
            search_query: String::new(),
            search_focused: false,
        }
    }

    pub fn update_theme(&mut self, t: Theme) {
        self.theme = t;
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        let full = ui.max_rect();

        ui.painter()
            .rect_filled(full, egui::Rounding::ZERO, t.panel_bg);

        let sidebar_w = 220.0;
        let sep_x = full.min.x + sidebar_w;

        ui.painter().rect_filled(
            egui::Rect::from_min_max(full.min, egui::pos2(sep_x, full.max.y)),
            egui::Rounding::ZERO,
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

        let search_stroke = if self.search_focused {
            t.accent
        } else {
            t.border
        };
        egui::Frame::none()
            .fill(t.button_bg)
            .rounding(egui::Rounding::same(5.0))
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
                                .hint_text("Search docs…")
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
                            let clear_resp =
                                ui.interact(clear_rect, clear_id, egui::Sense::click());
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
                let hits = chapter_search_text(*ch)
                    .to_lowercase()
                    .match_indices(query_lower.as_str())
                    .count();
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
                    egui::Rounding::same(8.0),
                    egui::Color32::from_rgba_premultiplied(
                        t.accent.r(),
                        t.accent.g(),
                        t.accent.b(),
                        45,
                    ),
                );
                ui.painter().text(
                    badge_center,
                    egui::Align2::CENTER_CENTER,
                    &badge_str,
                    egui::FontId::proportional(10.0),
                    t.accent,
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
                            Chapter::GettingStarted => render_getting_started(ui, t),
                            Chapter::TypesVariables => render_types_variables(ui, t),
                            Chapter::Operators => render_operators(ui, t),
                            Chapter::FunctionsControl => render_functions_control(ui, t),
                            Chapter::Structs => render_structs(ui, t),
                            Chapter::StdLib => render_stdlib(ui, t),
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

    ui.painter().rect_filled(row, egui::Rounding::ZERO, bg);

    if selected {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(row.min, egui::vec2(3.0, row.height())),
            egui::Rounding::ZERO,
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

fn chapter_search_text(ch: Chapter) -> &'static str {
    match ch {
        Chapter::GettingStarted => {
            "getting started program structure !start !end comments # ### import \
            keyboard shortcuts run compile Ctrl+S Ctrl+O Ctrl+N Ctrl+D terminal !exit \
            exit code greet hello world"
        }
        Chapter::TypesVariables => {
            "types variables int float char boolean void array list string declaration \
            assignment default values NULL literals binary hex octal decimal 0b 0x 0o 0d \
            type casting :int :float :char :boolean :array :list strong typing static \
            uninitialised initialiser"
        }
        Chapter::Operators => {
            "operators arithmetic unary binary plus minus multiply divide modulo \
            bitwise NOT AND OR XOR shift left right logical !not !and !or comparison \
            greater less equal not-equal ~= assignment += -= *= /= %= &= |= ^= \
            symbol :: -> type keyword"
        }
        Chapter::FunctionsControl => {
            "functions !func return !return control flow !if !elif !else !for !while \
            !break !continue conditionals loops for loop while loop indentation \
            curly braces scoping local global variables shadowing factorial clamp"
        }
        Chapter::Structs => {
            "structs struct user defined types :struct member access :: fields \
            nested struct fixed size complex types Vec2 Vec3 Rect Particle initialise"
        }
        Chapter::StdLib => {
            "standard library print input append pop insert delete find array list \
            io format string placeholder math import"
        }
    }
}

fn render_search_header(ui: &mut egui::Ui, t: &Theme, query: &str, chapter: Chapter) {
    note(
        ui,
        &format!(
            "Showing chapter \"{}\" — keywords matching \"{}\" highlighted in sidebar.",
            chapter.label(),
            query
        ),
        t,
    );
}

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
        egui::Rounding::ZERO,
        egui::Color32::from_rgba_premultiplied(t.border.r(), t.border.g(), t.border.b(), 120),
    );
    ui.add_space(10.0);
}

fn code(ui: &mut egui::Ui, src: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::none()
        .fill(t.editor_bg)
        .rounding(egui::Rounding::same(7.0))
        .inner_margin(egui::Margin::same(16.0))
        .stroke(egui::Stroke::new(1.0, t.border))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            let job =
                Highlighter::new(*t).highlight_to_layout_job(src, egui::FontId::monospace(12.5));
            ui.label(egui::WidgetText::LayoutJob(job));
        });
    ui.add_space(8.0);
}

fn note(ui: &mut egui::Ui, text: &str, t: &Theme) {
    ui.add_space(6.0);
    egui::Frame::none()
        .fill(egui::Color32::from_rgba_premultiplied(
            t.border.r(),
            t.border.g(),
            t.border.b(),
            60,
        ))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
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
    egui::Frame::none()
        .fill(egui::Color32::from_rgba_premultiplied(
            t.border.r(),
            t.border.g(),
            t.border.b(),
            60,
        ))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(14.0, 10.0))
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
    egui::Frame::none()
        .fill(t.editor_bg)
        .rounding(egui::Rounding::same(7.0))
        .inner_margin(egui::Margin::same(16.0))
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
    egui::Frame::none()
        .fill(t.editor_bg)
        .rounding(egui::Rounding::same(7.0))
        .inner_margin(egui::Margin::same(16.0))
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

fn shortcuts_table(ui: &mut egui::Ui, id: &str, rows: &[(&str, &str)], t: &Theme) {
    egui::Frame::none()
        .fill(t.editor_bg)
        .rounding(egui::Rounding::same(7.0))
        .inner_margin(egui::Margin::same(16.0))
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

fn render_getting_started(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Getting Started", t);
    para(
        ui,
        "Fractal is a statically-typed, strongly-typed language. Every program \
        must begin with !start and end with !end. Only code between those markers \
        is compiled and executed.",
        t,
    );

    rule(ui, t);

    h2(ui, "Program Structure", t);
    para(
        ui,
        "Variables declared directly inside !start (with no extra block nesting) \
        are global to the whole program. Everything else is local to its nearest { } block. \
        The compiler treats !start as { and !end as }.",
        t,
    );
    code(
        ui,
        "!start\n    # global variable — accessible everywhere\n    :int count = 0;\n\n    !func greet() -> :void {\n        print(\"Hello, Fractal!\\n\");\n    }\n\n    greet();\n!end",
        t,
    );

    h2(ui, "Comments", t);
    para(
        ui,
        "Single-line comments start with #. Block comments are wrapped in ### delimiters.",
        t,
    );
    code(
        ui,
        "# single-line comment\n\n###\n    This is a\n    multi-line comment.\n###",
        t,
    );

    h2(ui, "Importing", t);
    para(
        ui,
        "Use !import to bring in another Fractal file or a standard module.",
        t,
    );
    code(ui, "!import \"math\";", t);

    h2(ui, "Exiting", t);
    para(
        ui,
        "!exit terminates the program immediately with an integer exit code. \
        It can appear anywhere — inside or outside functions.",
        t,
    );
    code(ui, "!exit 0;   # success\n!exit 1;   # failure", t);

    h2(ui, "Keyboard Shortcuts", t);
    shortcuts_table(
        ui,
        "shortcuts_grid",
        &[
            ("Ctrl+S", "Save and auto-format"),
            ("Ctrl+Shift+S", "Save As…"),
            ("Ctrl+O", "Open file"),
            ("Ctrl+N", "New tab"),
            ("Ctrl+D", "Toggle documentation"),
            ("Ctrl+F", "Find in file"),
            ("Ctrl+H", "Find & Replace"),
            ("Ctrl+`", "Toggle terminal"),
        ],
        t,
    );

    note(
        ui,
        "Press Run (or use the toolbar) to compile and execute. \
        The fractal-compiler binary must be on PATH or in the same directory as this editor.",
        t,
    );
}

fn render_types_variables(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Types & Variables", t);
    para(
        ui,
        "Fractal is statically and strongly typed. Every variable must carry an explicit \
        type annotation prefixed with ':'. All type mismatches are compile-time errors — \
        there is no implicit casting.",
        t,
    );

    rule(ui, t);

    h2(ui, "Simple Types", t);
    kv3(
        ui,
        "simple_types",
        ["Type", "Description", "Default"],
        &[
            (":int", "64-bit signed integer", "0"),
            (":float", "64-bit IEEE 754 float", "0.0"),
            (":char", "32-bit Unicode character (UTF-32)", "'\\0'"),
            (":boolean", "Boolean — true or false", "false"),
            (":void", "The null type; !null is its only value", "—"),
        ],
        t,
    );
    note(
        ui,
        "Simple-type variables declared without an initialiser receive the default value above. \
        Arrays, lists, and structs have no default — a warning is issued if declared without one.",
        t,
    );

    h2(ui, "Iterable Types", t);
    kv3(
        ui,
        "iter_types",
        ["Type", "Description", "Example"],
        &[
            (
                ":array<T, N>",
                "Fixed-size, single element type",
                ":array<:int, 5>",
            ),
            (
                ":list<T>",
                "Variable-size, single element type",
                ":list<:float>",
            ),
        ],
        t,
    );
    para(
        ui,
        "Strings are char arrays or char lists. A double-quoted literal initialises a char array.",
        t,
    );

    h2(ui, "Integer Literals", t);
    shortcuts_table(
        ui,
        "int_lit",
        &[
            ("255  or  0d255", "Decimal"),
            ("0xFF", "Hexadecimal"),
            ("0b11111111", "Binary"),
            ("0o377", "Octal"),
        ],
        t,
    );

    h2(ui, "Float Literals", t);
    shortcuts_table(
        ui,
        "float_lit",
        &[
            ("1.5", "Plain float"),
            ("1.5e6", "With positive exponent"),
            ("9.9e-1", "With negative exponent"),
        ],
        t,
    );

    h2(ui, "Char Literals", t);
    shortcuts_table(
        ui,
        "char_lit",
        &[
            ("'A'", "ASCII character"),
            ("'\\n'", "Newline"),
            ("'\\t'", "Tab"),
            ("'\\\\'", "Backslash"),
            ("'\\''", "Single quote"),
            ("'\\0'", "Null character"),
        ],
        t,
    );

    h2(ui, "Array & String Literals", t);
    para(
        ui,
        "A char array can be initialised with a double-quoted string literal.",
        t,
    );
    code(
        ui,
        ":array<:int, 5>  nums  = [10, 20, 30, 40, 50];\n:array<:char, 6> hello = \"hello!\";\n:list<:float>    vals  = [1.1, 2.2, 3.3];",
        t,
    );

    h2(ui, "Declarations", t);
    code(
        ui,
        ":int     count  = 42;\n:float   ratio  = 0.618;\n:char    letter = 'F';\n:boolean flag   = true;\n\n# Uninitialised — gets default value\n:int   zero;     # == 0\n:float origin;   # == 0.0",
        t,
    );

    h2(ui, "Type Casting", t);
    para(
        ui,
        "Use the :Type(expr) syntax. No other implicit conversions exist. \
        The permitted casts are:",
        t,
    );
    kv2(
        ui,
        "cast_table",
        ["Cast", "Meaning"],
        &[
            (
                ":int(expr)",
                "float → int (truncates), char → codepoint, boolean → 0/1",
            ),
            (":float(expr)", "int → float, boolean → 0.0/1.0"),
            (":char(expr)", "int → Unicode character"),
            (
                ":boolean(expr)",
                "int → false if 0 else true; float → false if 0.0 else true",
            ),
        ],
        t,
    );
    code(
        ui,
        "# Numeric\n:float f = :float(42);        # 42.0\n:int   n = :int(3.99);        # 3  (truncates)\n\n# Character\n:char c  = :char(65);         # 'A'\n:int  cp = :int('Z');         # 90\n\n# Boolean\n:boolean b = :boolean(0);     # false\n:int     i = :int(true);      # 1\n\n# Chained\n:char ch = :char(:int(:float(66.9)));  # 'B'",
        t,
    );

    h2(ui, "Indexing Arrays & Lists", t);
    code(
        ui,
        ":int first = arr[0];\n:int last  = arr[4];\n:int i     = 2;\n:int mid   = arr[i];   # computed index",
        t,
    );
}

fn render_operators(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Operators", t);
    para(
        ui,
        "All operators are listed below grouped by kind. \
        Fractal uses ~= for not-equal (not !=) and the ! prefix for logical operators.",
        t,
    );

    rule(ui, t);

    h2(ui, "Arithmetic", t);
    kv2(
        ui,
        "arith_ops",
        ["Operator", "Description"],
        &[
            ("+ −", "Unary plus / negate"),
            ("+", "Addition"),
            ("−", "Subtraction"),
            ("*", "Multiplication"),
            ("/", "Division"),
            ("%", "Modulo (integer only)"),
        ],
        t,
    );

    h2(ui, "Bitwise (integer only)", t);
    kv2(
        ui,
        "bitwise_ops",
        ["Operator", "Description"],
        &[
            ("~", "Bitwise NOT (unary)"),
            ("&", "Bitwise AND"),
            ("|", "Bitwise OR"),
            ("^", "Bitwise XOR"),
            ("<<", "Left shift"),
            (">>", "Right shift"),
        ],
        t,
    );

    h2(ui, "Logical (boolean only)", t);
    kv2(
        ui,
        "logical_ops",
        ["Operator", "Description"],
        &[
            ("!not", "Logical NOT (unary)"),
            ("!and", "Logical AND"),
            ("!or", "Logical OR"),
        ],
        t,
    );
    code(
        ui,
        ":boolean a = true;\n:boolean b = false;\n:boolean c = !not a;           # false\n:boolean d = a !and b;         # false\n:boolean e = a !or  b;         # true\n:boolean f = (!not b) !and a;  # true",
        t,
    );

    h2(ui, "Comparison", t);
    kv2(
        ui,
        "cmp_ops",
        ["Operator", "Description"],
        &[
            (">", "Greater than"),
            ("<", "Less than"),
            (">=", "Greater than or equal"),
            ("<=", "Less than or equal"),
            ("==", "Equal"),
            ("~=", "Not equal (all types)"),
        ],
        t,
    );
    note(ui, "Fractal uses ~= for not-equal, not !=.", t);

    h2(ui, "Assignment", t);
    kv2(
        ui,
        "assign_ops",
        ["Operator", "Equivalent to"],
        &[
            ("=", "Simple assignment"),
            ("+=", "a = a + b"),
            ("-=", "a = a − b"),
            ("*=", "a = a * b"),
            ("/=", "a = a / b"),
            ("%=", "a = a % b  (int only)"),
            ("&=", "a = a & b  (int only)"),
            ("|=", "a = a | b  (int only)"),
            ("^=", "a = a ^ b  (int only)"),
        ],
        t,
    );

    h2(ui, "Special Symbols", t);
    kv2(
        ui,
        "other_sym",
        ["Symbol", "Meaning"],
        &[
            (":", "Type prefix — :int, :float, …"),
            ("!", "Keyword prefix — !if, !func, …"),
            ("->", "Function return type"),
            ("::", "Struct member access"),
            ("()", "Function call, grouping, type cast"),
            ("[]", "Array / list index"),
            ("{}", "Block / scope delimiter"),
            ("<>", "Type parameter (generics)"),
        ],
        t,
    );

    h2(ui, "Example — Mixed Expression", t);
    code(
        ui,
        ":int a = 255;\n:int b = 0xFF;\n\n:int bitwise = (a & b) | (a ^ 0b10101010);\n:int shifted  = a >> 1;\n\n:boolean ok = (a ~= 0) !and (:float(a) > 0.0);",
        t,
    );
}

fn render_functions_control(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Control Flow", t);
    para(
        ui,
        "Fractal has four control-flow constructs: !if / !elif / !else, !for, !while, \
        and !func. All blocks require curly braces. \
        The opening brace must be on the same line as the keyword.",
        t,
    );

    rule(ui, t);

    h2(ui, "Functions", t);
    para(
        ui,
        "Declare functions at the top level with !func. Functions may not be nested \
        inside !if, !for, !while, or another !func body.",
        t,
    );
    code(
        ui,
        "!func add(:int a, :int b) -> :int {\n    !return a + b;\n}\n\n!func clamp(:float val, :float lo, :float hi) -> :float {\n    !if (val < lo) { !return lo; }\n    !if (val > hi) { !return hi; }\n    !return val;\n}\n\n:int   sum = add(10, 32);\n:float cl  = clamp(150.0, 0.0, 100.0);",
        t,
    );
    note(
        ui,
        "!func definitions must appear at the top level of the program — not inside any block.",
        t,
    );

    h2(ui, "Conditionals — !if / !elif / !else", t);
    code(
        ui,
        "!if (grade >= 90) {\n    letter = 'A';\n}\n!elif (grade >= 80) {\n    letter = 'B';\n}\n!elif (grade >= 70) {\n    letter = 'C';\n}\n!else {\n    letter = 'F';\n}",
        t,
    );
    warning(
        ui,
        "The opening { must be on the same line as the condition. \
        Placing { on the next line produces a compiler warning.",
        t,
    );

    h2(ui, "For Loop", t);
    para(
        ui,
        "Syntax: !for (:int var, start, exclusive_end, step). \
        The loop variable must be :int and must not shadow any variable in an enclosing scope.",
        t,
    );
    code(
        ui,
        "# Count 0 to 9\n!for (:int i, 0, 10, 1) {\n    print(\"{}\", i);\n}\n\n# Nested loops\n!for (:int row, 0, 4, 1) {\n    !for (:int col, 0, 4, 1) {\n        print(\"{},{} \", row, col);\n    }\n}",
        t,
    );
    note(
        ui,
        "The !for loop variable must be :int — using :float is a compile error. \
        It also must not shadow any variable already in scope.",
        t,
    );

    h2(ui, "While Loop", t);
    code(
        ui,
        ":int n = 27;\n:int steps = 0;\n\n!while (n ~= 1) {\n    !if (is_even(n)) {\n        n /= 2;\n    } !else {\n        n = n * 3 + 1;\n    }\n    steps += 1;\n}",
        t,
    );

    h2(ui, "Break & Continue", t);
    code(
        ui,
        "# !break exits the nearest loop\n!for (:int i, 0, 100, 1) {\n    !if (i == 42) { !break; }\n}\n\n# !continue skips to the next iteration\n:int odd_sum = 0;\n!for (:int i, 0, 20, 1) {\n    !if (is_even(i)) { !continue; }\n    odd_sum += i;\n}",
        t,
    );

    h2(ui, "Variable Scoping", t);
    para(
        ui,
        "All variables are local to their nearest { } block. \
        Variables declared directly inside !start (no nesting) are global.",
        t,
    );
    code(
        ui,
        ":int global = 10;   # accessible everywhere\n\n!if (true) {\n    :int local = global + 1;  # only visible in this block\n    global = local;\n}\n\n# local is no longer accessible here",
        t,
    );

    h2(ui, "Full Example — Factorial", t);
    code(
        ui,
        "!start\n\n    !func factorial(:int n) -> :int {\n        :int result = 1;\n        !for (:int i, 1, n, 1) {\n            result *= i;\n        }\n        !return result;\n    }\n\n    print(\"{}\", factorial(6));   # 720\n\n!end",
        t,
    );
}

fn render_structs(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Structs", t);
    para(
        ui,
        "User-defined structures group related fields into a named type. \
        Structs are always fixed-size. Members are accessed with the :: operator.",
        t,
    );

    rule(ui, t);

    h2(ui, "Defining a Struct", t);
    code(
        ui,
        ":struct<Vec2> {\n    :float x;\n    :float y;\n};\n\n:struct<Vec3> {\n    :float x;\n    :float y;\n    :float z;\n};",
        t,
    );

    h2(ui, "Nested Structs", t);
    code(
        ui,
        ":struct<Rect> {\n    :struct<Vec2> top_left;\n    :struct<Vec2> bottom_right;\n    :float width;\n    :float height;\n};\n\n:struct<Particle> {\n    :struct<Vec3>     pos;\n    :struct<Vec3>     vel;\n    :float            mass;\n    :boolean          active;\n    :int              id;\n    :array<:float, 3> color;\n};",
        t,
    );

    h2(ui, "Initialising Structs", t);
    code(
        ui,
        ":struct<Vec2> origin = { x = 0.0, y = 0.0 };\n:struct<Vec2> point  = { x = 3.0, y = 4.0 };\n\n:struct<Rect> box1 = {\n    top_left     = { x = 0.0,  y = 0.0  },\n    bottom_right = { x = 10.0, y = 10.0 },\n    width  = 10.0,\n    height = 10.0\n};",
        t,
    );

    h2(ui, "Member Access with ::", t);
    code(
        ui,
        ":float px    = point::x;\n:float box_w = box1::width;\n\n# Deep access through nested structs\n:float tl_x = box1::top_left::x;",
        t,
    );

    h2(ui, "Structs in Functions", t);
    code(
        ui,
        "!func dot2(:struct<Vec2> a, :struct<Vec2> b) -> :float {\n    !return (a::x * b::x) + (a::y * b::y);\n}\n\n:float d = dot2(origin, point);",
        t,
    );

    warning(
        ui,
        "Declaring a struct without an initialiser produces a compiler warning — structs have no default value.",
        t,
    );
}

fn render_stdlib(ui: &mut egui::Ui, t: &Theme) {
    h1(ui, "Standard Library", t);
    para(
        ui,
        "The standard library provides I/O functions and collection methods. \
        Import additional modules with !import (e.g. !import \"math\";).",
        t,
    );

    rule(ui, t);

    h2(ui, "print", t);
    para(
        ui,
        "Writes a formatted string to standard output. \
        Use {} as a placeholder for each argument, in order.",
        t,
    );
    code(
        ui,
        "print(\"{}\", 42);                       # 42\nprint(\"{} {} {}\", 1.0, 'A', true);     # 1 A true\nprint(\"Sum: {}\", a + b);",
        t,
    );

    h2(ui, "input", t);
    para(
        ui,
        "Reads from standard input and fills placeholder variables from the format string. \
        A type mismatch between the format and the variable is a compile error.",
        t,
    );
    code(
        ui,
        ":int n;\ninput(\"{}\", n);   # reads an integer from stdin",
        t,
    );

    h2(ui, "Array Methods", t);
    kv3(
        ui,
        "arr_methods",
        ["Function", "Description", "Returns"],
        &[(
            "find(arr, value)",
            "Index of first match, or −1 if not found",
            ":int",
        )],
        t,
    );

    h2(ui, "List Methods", t);
    kv3(
        ui,
        "list_methods",
        ["Function", "Description", "Returns"],
        &[
            ("append(lst, value)", "Add element to the end", ":void"),
            ("pop(lst)", "Remove and return the last element", "T"),
            ("insert(lst, index)", "Insert element at index", ":void"),
            ("delete(lst, index)", "Remove element at index", ":void"),
            ("find(lst, value)", "First index of value, or −1", ":int"),
        ],
        t,
    );
    code(
        ui,
        ":list<:int> nums = [1, 2, 3];\n\nappend(nums, 4);\nappend(nums, 5);\n\n:int last = pop(nums);        # 5\n:int idx  = find(nums, 2);    # 1\ninsert(nums, 0);              # insert 0 at front\ndelete(nums, 0);              # remove index 0",
        t,
    );

    h2(ui, "Full Example", t);
    code(
        ui,
        "!start\n\n    !func is_even(:int n) -> :boolean {\n        !return (n % 2) == 0;\n    }\n\n    :list<:int> evens = [0];\n\n    !for (:int i, 1, 20, 1) {\n        !if (is_even(i)) {\n            append(evens, i);\n        }\n    }\n\n    :int idx = find(evens, 8);   # 4\n    print(\"Found 8 at index {}\", idx);\n\n!end",
        t,
    );
}
