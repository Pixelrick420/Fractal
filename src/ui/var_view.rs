use super::debugger::DebugFrame;
use super::theme::Theme;
use eframe::egui;

pub struct VarViewWindow {
    pub open: bool,
    pub title: String,
    output_history: String,
}

impl VarViewWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            title: "Variable State".into(),
            output_history: String::new(),
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

        egui::Window::new("Variable State")
            .id(egui::Id::new("fractal_var_view"))
            .open(&mut open)
            .default_size([320.0, 460.0])
            .min_size([240.0, 160.0])
            .max_size([560.0, 860.0])
            .resizable(true)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(t.panel_bg)
                    .stroke(egui::Stroke::new(1.0, t.border))
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                ui.set_min_width(240.0);

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

                        let scopes: Vec<_> = frame.scopes.iter().rev().collect();

                        if scopes.is_empty() {
                            ui.horizontal(|ui| {
                                ui.add_space(12.0);
                                ui.label(
                                    egui::RichText::new("No variables yet")
                                        .size(11.5)
                                        .color(muted_text)
                                        .italics(),
                                );
                            });
                        }

                        for (idx, scope) in scopes.iter().enumerate() {
                            let is_top = idx == 0;

                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                let hdr_text = if scope.label == "global" {
                                    "global scope".into()
                                } else {
                                    format!("fn: {}", scope.label)
                                };
                                ui.label(
                                    egui::RichText::new(hdr_text)
                                        .size(10.5)
                                        .color(if is_top { t.accent } else { muted_text })
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

                            ui.add_space(6.0);

                            if idx < scopes.len() - 1 {
                                let (sr, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width().max(10.0) - 16.0, 1.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    sr,
                                    egui::CornerRadius::ZERO,
                                    egui::Color32::from_rgba_premultiplied(
                                        t.border.r(),
                                        t.border.g(),
                                        t.border.b(),
                                        100,
                                    ),
                                );
                                ui.add_space(6.0);
                            }
                        }

                        if !frame.call_stack.is_empty() {
                            ui.add_space(4.0);
                            let (cs_sep, _) = ui.allocate_exact_size(
                                egui::vec2(ui.available_width(), 1.0),
                                egui::Sense::hover(),
                            );
                            ui.painter()
                                .rect_filled(cs_sep, egui::CornerRadius::ZERO, t.border);
                            ui.add_space(6.0);

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

                            let stack = &frame.call_stack;
                            for (i, name) in stack.iter().enumerate().rev() {
                                let is_cur = i == stack.len() - 1;
                                let col = if is_cur { t.accent } else { muted_text };
                                let depth_indent = (stack.len() - 1 - i) as f32 * 12.0;
                                ui.horizontal(|ui| {
                                    ui.add_space(16.0 + depth_indent);
                                    ui.label(
                                        egui::RichText::new(if is_cur { "▶ " } else { "  " })
                                            .size(10.0)
                                            .color(col),
                                    );
                                    ui.label(
                                        egui::RichText::new(name).size(11.0).color(col).monospace(),
                                    );
                                });
                            }
                        }

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

        ui.painter().text(
            egui::pos2(row_rect.left() + 6.0, row_rect.center().y),
            egui::Align2::LEFT_CENTER,
            clip_str(&row.name, col_name_w - 10.0, &font, ui),
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
