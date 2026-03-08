use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use eframe::egui;

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
                        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

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
                                let lower = line.to_ascii_lowercase();
                                let color = if lower.contains("error") {
                                    t.terminal_error
                                } else if lower.contains("warning") {
                                    t.terminal_warning
                                } else {
                                    t.terminal_fg
                                };
                                ui.label(
                                    egui::RichText::new(line)
                                        .size(12.5)
                                        .color(color)
                                        .monospace(),
                                );
                            }
                        }
                    });
            });
    }

    fn draw_border(&self, ctx: &egui::Context, rect: egui::Rect, t: Theme) {
        ctx.layer_painter(egui::LayerId::background()).line_segment(
            [rect.left_top(), rect.right_top()],
            egui::Stroke::new(1.0, t.border),
        );
    }
}
