use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;

#[derive(Default)]
pub struct SearchBar {
    pub visible: bool,
    pub replace_mode: bool,
    pub query: String,
    pub replace_text: String,
    pub match_case: bool,
    pub current_match: usize,
    pub total_matches: usize,
    pub focus_search: bool,
    pub focus_replace: bool,
}

pub enum SearchBarAction {
    None,
    FindNext,
    FindPrev,
    ReplaceOne,
    ReplaceAll,
    Close,
}

impl SearchBar {
    pub fn open_search(&mut self) {
        self.visible = true;
        self.replace_mode = false;
        self.focus_search = true;
    }

    pub fn open_replace(&mut self) {
        self.visible = true;
        self.replace_mode = true;
        self.focus_search = true;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
        self.replace_text.clear();
        self.current_match = 0;
        self.total_matches = 0;
    }

    pub fn update_matches(&mut self, code: &str) {
        if self.query.is_empty() {
            self.total_matches = 0;
            self.current_match = 0;
            return;
        }
        let count = if self.match_case {
            code.matches(self.query.as_str()).count()
        } else {
            code.to_lowercase()
                .matches(self.query.to_lowercase().as_str())
                .count()
        };
        self.total_matches = count;
        if count == 0 {
            self.current_match = 0;
        } else if self.current_match >= count {
            self.current_match = count - 1;
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, t: &Theme) -> SearchBarAction {
        if !self.visible {
            return SearchBarAction::None;
        }

        // Flags set by keyboard or UI buttons, resolved after the panel renders.
        let mut do_close = false;
        let mut do_find_next = false;
        let mut do_find_prev = false;
        let mut do_replace_one = false;
        let mut do_replace_all = false;

        ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::Escape) {
                do_close = true;
                i.events.retain(|e| !matches!(e, egui::Event::Key { key: egui::Key::Escape, .. }));
            }
            if i.key_pressed(egui::Key::Enter) {
                if i.modifiers.shift {
                    do_find_prev = true;
                } else {
                    do_find_next = true;
                }
                i.events.retain(|e| !matches!(e, egui::Event::Key { key: egui::Key::Enter, .. }));
            }
        });

        if do_close {
            self.close();
            return SearchBarAction::Close;
        }

        let panel_h = if self.replace_mode { 76.0 } else { 44.0 };

        egui::TopBottomPanel::top("search_bar_panel")
            .frame(
                egui::Frame::none()
                    .fill(t.panel_bg)
                    .inner_margin(egui::Margin {
                        left: 12.0,
                        right: 12.0,
                        top: 6.0,
                        bottom: 6.0,
                    }),
            )
            .exact_height(panel_h)
            .show(ctx, |ui| {
                // ── Row 1: Search ─────────────────────────────────────────
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;

                    // Search input
                    let search_resp = ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .desired_width(220.0)
                            .hint_text("Search…")
                            .font(egui::FontId::proportional(13.0)),
                    );
                    if self.focus_search {
                        search_resp.request_focus();
                        self.focus_search = false;
                    }
                    if search_resp.changed() {
                        self.current_match = 0;
                    }

                    // Match count
                    if !self.query.is_empty() {
                        let label = if self.total_matches == 0 {
                            "No results".to_string()
                        } else {
                            format!("{}/{}", self.current_match + 1, self.total_matches)
                        };
                        ui.label(
                            egui::RichText::new(label)
                                .size(11.5)
                                .color(if self.total_matches == 0 {
                                    t.terminal_error
                                } else {
                                    t.tab_inactive_fg
                                }),
                        );
                    }

                    // Match-case toggle "Aa"
                    let case_col = if self.match_case { t.tab_bar_bg } else { t.tab_inactive_fg };
                    let case_fill = if self.match_case { t.accent } else { t.button_bg };
                    let case_btn = ui.add(
                        egui::Button::new(egui::RichText::new("Aa").size(11.5).color(case_col))
                            .fill(case_fill)
                            .stroke(egui::Stroke::new(1.0, t.border))
                            .min_size(egui::vec2(28.0, 22.0)),
                    );
                    if case_btn.clicked() {
                        self.match_case = !self.match_case;
                        self.current_match = 0;
                    }
                    case_btn.on_hover_text("Match case");

                    ui.add_space(4.0);

                    // ↑ Prev
                    let prev_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(ic::TERM_EXPAND).size(13.0).color(t.button_fg),
                        )
                        .fill(t.button_bg)
                        .stroke(egui::Stroke::new(1.0, t.border))
                        .min_size(egui::vec2(26.0, 22.0)),
                    );
                    if prev_btn.clicked() {
                        do_find_prev = true;
                    }
                    prev_btn.on_hover_text("Previous match (Shift+Enter)");

                    // ↓ Next
                    let next_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(ic::TERM_COLLAPSE).size(13.0).color(t.button_fg),
                        )
                        .fill(t.button_bg)
                        .stroke(egui::Stroke::new(1.0, t.border))
                        .min_size(egui::vec2(26.0, 22.0)),
                    );
                    if next_btn.clicked() {
                        do_find_next = true;
                    }
                    next_btn.on_hover_text("Next match (Enter)");

                    // Replace toggle
                    let rep_label = if self.replace_mode { "▲ Replace" } else { "▼ Replace" };
                    let rep_toggle = ui.add(
                        egui::Button::new(
                            egui::RichText::new(rep_label).size(11.5).color(t.tab_inactive_fg),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .stroke(egui::Stroke::NONE),
                    );
                    if rep_toggle.clicked() {
                        self.replace_mode = !self.replace_mode;
                        if self.replace_mode {
                            self.focus_replace = true;
                        }
                    }

                    // Close button — manually painted so the phosphor glyph always shows
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (r, resp) =
                            ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
                        let hov = resp.hovered();
                        if hov {
                            ui.painter()
                                .rect_filled(r, egui::Rounding::same(4.0), t.button_hover_bg);
                        }
                        // Paint glyph explicitly — bypasses egui Button colour override
                        ui.painter().text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            ic::TAB_CLOSE,
                            egui::FontId::proportional(14.0),
                            if hov { t.tab_active_fg } else { t.tab_inactive_fg },
                        );
                        if resp.clicked() {
                            do_close = true;
                        }
                    });
                });

                // ── Row 2: Replace ────────────────────────────────────────
                if self.replace_mode {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;

                        let replace_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.replace_text)
                                .desired_width(220.0)
                                .hint_text("Replace with…")
                                .font(egui::FontId::proportional(13.0)),
                        );
                        if self.focus_replace {
                            replace_resp.request_focus();
                            self.focus_replace = false;
                        }

                        let rep_one = ui.add(
                            egui::Button::new(
                                egui::RichText::new("Replace").size(12.0).color(t.button_fg),
                            )
                            .fill(t.button_bg)
                            .stroke(egui::Stroke::new(1.0, t.border))
                            .min_size(egui::vec2(60.0, 22.0)),
                        );
                        if rep_one.clicked() {
                            do_replace_one = true;
                        }
                        rep_one.on_hover_text("Replace current match");

                        let rep_all = ui.add(
                            egui::Button::new(
                                egui::RichText::new("Replace All").size(12.0).color(t.button_fg),
                            )
                            .fill(t.button_bg)
                            .stroke(egui::Stroke::new(1.0, t.border))
                            .min_size(egui::vec2(80.0, 22.0)),
                        );
                        if rep_all.clicked() {
                            do_replace_all = true;
                        }
                        rep_all.on_hover_text("Replace all matches");
                    });
                }

                // Bottom border line
                let r = ui.min_rect();
                ui.painter().line_segment(
                    [r.left_bottom(), r.right_bottom()],
                    egui::Stroke::new(1.0, t.border),
                );
            });

        // Resolve after panel — order matters: close > replace_all > replace_one > prev > next
        if do_close {
            self.close();
            SearchBarAction::Close
        } else if do_replace_all {
            SearchBarAction::ReplaceAll
        } else if do_replace_one {
            SearchBarAction::ReplaceOne
        } else if do_find_prev {
            SearchBarAction::FindPrev
        } else if do_find_next {
            SearchBarAction::FindNext
        } else {
            SearchBarAction::None
        }
    }
}