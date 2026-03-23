use super::debugger::DebugFrame;
use super::theme::Theme;
use eframe::egui;

pub struct VarViewWindow {
    pub open: bool,
    pub title: String,
    output_history: String,
    /// Index into frame.scopes that the user has selected.
    /// 0 = the top (currently-executing) frame, which is the default.
    /// None means "follow the top frame automatically".
    selected_scope: Option<usize>,
}

impl VarViewWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            title: "Variable State".into(),
            output_history: String::new(),
            selected_scope: None,
        }
    }

    pub fn push_output(&mut self, s: &str) {
        if !s.is_empty() {
            self.output_history.push_str(s);
        }
    }

    pub fn clear_output(&mut self) {
        self.output_history.clear();
    }

    /// Called whenever the debug session steps forward so the selection
    /// resets to the top (active) frame.
    pub fn on_step(&mut self) {
        self.selected_scope = None;
    }

    pub fn show(&mut self, ctx: &egui::Context, frame: &DebugFrame, theme: &Theme) {
        if !self.open {
            return;
        }

        let t = *theme;
        let output = self.output_history.clone();
        let mut open = self.open;

        let body_text = t.tab_active_fg;
        let muted_text = t.tab_inactive_fg;
        let value_col = t.tab_active_fg;

        let changed_bg = {
            let a = t.tab_dirty_dot;
            let b = t.panel_bg;
            egui::Color32::from_rgb(
                ((a.r() as u16 * 80 + b.r() as u16 * 175) / 255) as u8,
                ((a.g() as u16 * 80 + b.g() as u16 * 175) / 255) as u8,
                ((a.b() as u16 * 80 + b.b() as u16 * 175) / 255) as u8,
            )
        };
        let changed_fg = t.tab_dirty_dot;

        let alt_row = {
            use crate::ui::theme::ThemeVariant;
            match t.variant {
                ThemeVariant::Dark => egui::Color32::from_rgba_premultiplied(255, 255, 255, 18),
                ThemeVariant::Light => egui::Color32::from_rgba_premultiplied(0, 0, 0, 22),
            }
        };
        let accent_bg =
            egui::Color32::from_rgba_premultiplied(t.accent.r(), t.accent.g(), t.accent.b(), 22);

        // scopes[0] is the TOP (active) frame. The displayed scope is either
        // what the user clicked or always index-0.
        let num_scopes = frame.scopes.len();
        let effective_scope = self.selected_scope.unwrap_or(0).min(num_scopes.saturating_sub(1));

        egui::Window::new("Variable State")
            .id(egui::Id::new("fractal_var_view"))
            .open(&mut open)
            .default_size([340.0, 500.0])
            .min_size([240.0, 160.0])
            .max_size([600.0, 900.0])
            .resizable(true)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(t.panel_bg)
                    .stroke(egui::Stroke::new(1.0, t.border))
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                ui.set_min_width(240.0);

                // ── Header bar ──────────────────────────────────────────────
                egui::Frame::new()
                    .fill(accent_bg)
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let line_txt = if frame.source_line > 0 {
                                format!("  ln {}", frame.source_line)
                            } else {
                                String::new()
                            };
                            ui.label(
                                egui::RichText::new(format!("▶  {}{}", frame.step_label, line_txt))
                                    .size(11.0)
                                    .color(t.accent)
                                    .monospace(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let (status, col) = if frame.finished {
                                        ("finished", muted_text)
                                    } else if frame.error.is_some() {
                                        ("error", t.terminal_error)
                                    } else {
                                        ("running", t.struct_name)
                                    };
                                    ui.label(egui::RichText::new(status).size(10.0).color(col));
                                },
                            );
                        });
                    });

                let (sep, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 1.0),
                    egui::Sense::hover(),
                );
                ui.painter()
                    .rect_filled(sep, egui::CornerRadius::ZERO, t.border);

                egui::ScrollArea::vertical()
                    .id_salt("var_view_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(8.0);

                        // ── Call-stack selector ──────────────────────────────
                        // Displayed top→bottom so the active (top) frame is first.
                        // frame.scopes[0] = active callee, last = main/<main>.
                        // frame.call_stack is bottom→top so last = active name.
                        if !frame.call_stack.is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                ui.label(
                                    egui::RichText::new("call stack")
                                        .size(10.0)
                                        .color(muted_text)
                                        .strong(),
                                );
                            });
                            ui.add_space(4.0);

                            // Stack entries: iterate call_stack in reverse so the active
                            // function appears first (matches scopes[0]).
                            let stack = &frame.call_stack;
                            for (visual_idx, stack_entry) in stack.iter().enumerate().rev().enumerate() {
                                let (stack_idx, name) = stack_entry;
                                // scope_idx: the stack's top = scopes[0], bottom = scopes[last]
                                let scope_idx = stack.len() - 1 - stack_idx;
                                let is_selected = scope_idx == effective_scope;

                                let row_bg = if is_selected {
                                    accent_bg
                                } else {
                                    egui::Color32::TRANSPARENT
                                };

                                let (row_rect, row_resp) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 22.0),
                                    egui::Sense::click(),
                                );

                                if row_resp.clicked() {
                                    self.selected_scope = Some(scope_idx);
                                }

                                if row_resp.hovered() && !is_selected {
                                    ui.painter().rect_filled(
                                        row_rect,
                                        egui::CornerRadius::ZERO,
                                        egui::Color32::from_rgba_premultiplied(
                                            t.accent.r(), t.accent.g(), t.accent.b(), 12,
                                        ),
                                    );
                                } else if is_selected {
                                    ui.painter().rect_filled(
                                        row_rect,
                                        egui::CornerRadius::ZERO,
                                        row_bg,
                                    );
                                }

                                let col = if is_selected { t.accent } else { muted_text };
                                let depth_indent = visual_idx as f32 * 10.0;
                                let arrow = if is_selected { "▶ " } else { "  " };

                                ui.painter().text(
                                    egui::pos2(row_rect.left() + 16.0 + depth_indent, row_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    format!("{}{}", arrow, name),
                                    egui::FontId::monospace(11.0),
                                    col,
                                );
                            }

                            ui.add_space(6.0);
                            let (cs_sep, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 1.0),
                                egui::Sense::hover(),
                            );
                            ui.painter()
                                .rect_filled(cs_sep, egui::CornerRadius::ZERO, t.border);
                            ui.add_space(6.0);
                        }

                        // ── Variable table for the selected scope ────────────
                        if frame.scopes.is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new("No variables yet")
                                        .size(11.5)
                                        .color(muted_text)
                                        .italics(),
                                );
                            });
                        } else {
                            let scope = &frame.scopes[effective_scope];

                            // Scope header
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                let hdr_text = if scope.label == "global"
                                    || scope.label == "<main>"
                                {
                                    "global scope".into()
                                } else {
                                    format!("fn: {}", scope.label)
                                };
                                ui.label(
                                    egui::RichText::new(hdr_text)
                                        .size(10.5)
                                        .color(t.accent)
                                        .strong(),
                                );
                            });
                            ui.add_space(4.0);

                            if scope.vars.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.add_space(16.0);
                                    ui.label(
                                        egui::RichText::new("(no variables)")
                                            .size(11.0)
                                            .color(muted_text)
                                            .italics(),
                                    );
                                });
                            } else {
                                ui.add_space(2.0);
                                let indent = egui::Frame::new().inner_margin(egui::Margin {
                                    left: 8,
                                    right: 4,
                                    top: 0,
                                    bottom: 0,
                                });
                                indent.show(ui, |ui| {
                                    draw_var_table(
                                        ui,
                                        scope.vars.as_slice(),
                                        body_text,
                                        muted_text,
                                        value_col,
                                        changed_bg,
                                        changed_fg,
                                        alt_row,
                                        t.type_name,
                                        t.border,
                                    );
                                });
                            }
                        }

                        // ── Error banner ─────────────────────────────────────
                        if let Some(err) = &frame.error {
                            ui.add_space(8.0);
                            egui::Frame::new()
                                .fill(egui::Color32::from_rgba_premultiplied(
                                    t.terminal_error.r(),
                                    t.terminal_error.g(),
                                    t.terminal_error.b(),
                                    40,
                                ))
                                .stroke(egui::Stroke::new(1.0, t.terminal_error))
                                .inner_margin(egui::Margin::symmetric(10, 6))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(
                                        egui::RichText::new(format!("⚠  {}", err))
                                            .size(11.0)
                                            .color(t.terminal_error),
                                    );
                                });
                        }

                        // ── Output so far ────────────────────────────────────
                        if !output.is_empty() {
                            ui.add_space(8.0);
                            let (o_sep, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 1.0),
                                egui::Sense::hover(),
                            );
                            ui.painter()
                                .rect_filled(o_sep, egui::CornerRadius::ZERO, t.border);
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                ui.label(
                                    egui::RichText::new("output so far")
                                        .size(10.0)
                                        .color(muted_text)
                                        .strong(),
                                );
                            });
                            ui.add_space(4.0);

                            egui::Frame::new()
                                .fill(t.editor_bg)
                                .stroke(egui::Stroke::new(1.0, t.border))
                                .corner_radius(egui::CornerRadius::same(4))
                                .inner_margin(egui::Margin::same(8))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(
                                        egui::RichText::new(&output)
                                            .size(11.0)
                                            .color(t.terminal_fg)
                                            .monospace(),
                                    );
                                });
                        }

                        ui.add_space(12.0);
                    });
            });

        self.open = open;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_var_table(
    ui: &mut egui::Ui,
    vars: &[super::debugger::VarRow],
    body_text: egui::Color32,
    _muted: egui::Color32,
    value_col: egui::Color32,
    changed_bg: egui::Color32,
    changed_fg: egui::Color32,
    alt_row: egui::Color32,
    type_col: egui::Color32,
    border_col: egui::Color32,
) {
    let available_w = ui.available_width().max(160.0);

    let col_name_w = (available_w * 0.30).min(100.0).max(52.0);
    let col_type_w = (available_w * 0.20).min(70.0).max(44.0);
    let col_val_w = (available_w - col_name_w - col_type_w).max(40.0);
    let row_h = 22.0_f32;

    let (hdr_rect, _) = ui.allocate_exact_size(egui::vec2(available_w, 17.0), egui::Sense::hover());
    ui.painter().rect_filled(
        hdr_rect,
        egui::CornerRadius::same(3),
        egui::Color32::from_rgba_premultiplied(border_col.r(), border_col.g(), border_col.b(), 130),
    );
    let hfont = egui::FontId::proportional(9.5);
    for (x, lbl) in [
        (hdr_rect.left() + 6.0, "NAME"),
        (hdr_rect.left() + col_name_w + 6.0, "TYPE"),
        (hdr_rect.left() + col_name_w + col_type_w + 6.0, "VALUE"),
    ] {
        ui.painter().text(
            egui::pos2(x, hdr_rect.center().y),
            egui::Align2::LEFT_CENTER,
            lbl,
            hfont.clone(),
            body_text,
        );
    }

    let font = egui::FontId::monospace(10.5);

    for (idx, row) in vars.iter().enumerate() {
        let (row_rect, _) =
            ui.allocate_exact_size(egui::vec2(available_w, row_h), egui::Sense::hover());

        if row.changed {
            ui.painter()
                .rect_filled(row_rect, egui::CornerRadius::same(2), changed_bg);
        } else if idx % 2 == 1 {
            ui.painter()
                .rect_filled(row_rect, egui::CornerRadius::ZERO, alt_row);
        }

        // Strip the fractal_ prefix for display
        let display_name = row.name.strip_prefix("fractal_").unwrap_or(&row.name);

        ui.painter().text(
            egui::pos2(row_rect.left() + 6.0, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            clip_str(display_name, col_name_w - 10.0, &font, ui),
            font.clone(),
            body_text,
        );

        ui.painter().text(
            egui::pos2(row_rect.left() + col_name_w + 6.0, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            clip_str(&row.type_label, col_type_w - 10.0, &font, ui),
            font.clone(),
            type_col,
        );

        let vc = if row.changed { changed_fg } else { value_col };
        ui.painter().text(
            egui::pos2(
                row_rect.left() + col_name_w + col_type_w + 6.0,
                row_rect.center().y,
            ),
            egui::Align2::LEFT_CENTER,
            clip_str(&row.value, col_val_w - 14.0, &font, ui),
            font.clone(),
            vc,
        );

        if row.changed {
            ui.painter().circle_filled(
                egui::pos2(row_rect.right() - 5.0, row_rect.center().y),
                3.0,
                changed_fg,
            );
        }
    }
}

fn clip_str(s: &str, max_px: f32, font: &egui::FontId, ui: &egui::Ui) -> String {
    if max_px <= 0.0 || s.is_empty() {
        return String::new();
    }
    let char_w = ui.fonts_mut(|f| f.glyph_width(font, 'x')).max(1.0);
    let max_chars = ((max_px / char_w) as usize).max(1);
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}