use crate::ui::icons as ic;
use crate::ui::theme::{Theme, ThemeVariant};
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub theme: ThemeVariant,
    pub font_size: f32,
    pub show_line_numbers: bool,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            theme: ThemeVariant::Dark,
            font_size: 14.0,
            show_line_numbers: true,
        }
    }
}

impl UserProfile {
    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("fractal-editor").join("profile.json"))
    }
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    pub fn save(&self) {
        if let Some(p) = Self::path() {
            if let Some(d) = p.parent() {
                let _ = std::fs::create_dir_all(d);
            }
            if let Ok(j) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(p, j);
            }
        }
    }
}

pub struct SettingsPanel {
    pub visible: bool,
    theme: Theme,
}

impl SettingsPanel {
    pub fn new() -> Self {
        Self {
            visible: false,
            theme: Theme::default(),
        }
    }
    pub fn update_theme(&mut self, t: Theme) {
        self.theme = t;
    }
    pub fn open(&mut self) {
        self.visible = true;
    }

    pub fn show(&mut self, ctx: &egui::Context, profile: &mut UserProfile) -> bool {
        if !self.visible {
            return false;
        }

        let t = self.theme;
        let mut changed = false;

        let screen = ctx.screen_rect();
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::PanelResizeLine,
            egui::Id::new("settings_scrim"),
        ));
        painter.rect_filled(
            screen,
            egui::Rounding::ZERO,
            egui::Color32::from_black_alpha(120),
        );

        let w = 340.0_f32;

        let h = 52.0
            + 32.0
            + (ThemeVariant::all().len() as f32 * 36.0)
            + 32.0
            + 32.0
            + 46.0
            + 46.0
            + 24.0;
        let panel_rect = egui::Rect::from_center_size(screen.center(), egui::vec2(w, h));

        painter.rect_filled(
            panel_rect.translate(egui::vec2(0.0, 4.0)).expand(8.0),
            egui::Rounding::same(14.0),
            egui::Color32::from_black_alpha(60),
        );
        painter.rect_filled(panel_rect, egui::Rounding::same(10.0), t.panel_bg);
        painter.rect_stroke(
            panel_rect,
            egui::Rounding::same(10.0),
            egui::Stroke::new(1.0, t.border),
        );

        egui::Area::new(egui::Id::new("settings_area"))
            .fixed_pos(panel_rect.min)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_clip_rect(panel_rect);
                ui.set_min_size(egui::vec2(w, h));

                {
                    let style = ui.style_mut();
                    let fg = t.tab_active_fg;
                    let bg = t.button_bg;
                    let hover = t.button_hover_bg;
                    let border = t.border;
                    let accent = t.accent;

                    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, fg);
                    style.visuals.widgets.noninteractive.bg_fill = bg;
                    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);

                    style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, fg);
                    style.visuals.widgets.inactive.bg_fill = bg;
                    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);

                    style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, fg);
                    style.visuals.widgets.hovered.bg_fill = hover;
                    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, accent);

                    style.visuals.widgets.active.fg_stroke =
                        egui::Stroke::new(1.0, egui::Color32::WHITE);
                    style.visuals.widgets.active.bg_fill = accent;
                    style.visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, accent);

                    style.visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(
                        accent.r(),
                        accent.g(),
                        accent.b(),
                        60,
                    );
                    style.visuals.selection.stroke = egui::Stroke::new(1.0, accent);

                    style.visuals.hyperlink_color = accent;

                    style.visuals.extreme_bg_color = bg;

                    style.visuals.override_text_color = Some(fg);
                }

                egui::Frame::none()
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| {
                        ui.set_max_width(w - 48.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Settings")
                                    .size(17.0)
                                    .strong()
                                    .color(t.tab_active_fg),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let close_resp = ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(ic::TAB_CLOSE)
                                                .size(16.0)
                                                .color(t.tab_inactive_fg),
                                        )
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(egui::Stroke::NONE)
                                        .min_size(egui::vec2(28.0, 28.0)),
                                    );
                                    if close_resp.hovered() {
                                        ui.painter().rect_filled(
                                            close_resp.rect,
                                            egui::Rounding::same(5.0),
                                            t.button_hover_bg,
                                        );
                                        ui.painter().text(
                                            close_resp.rect.center(),
                                            egui::Align2::CENTER_CENTER,
                                            ic::TAB_CLOSE,
                                            egui::FontId::proportional(16.0),
                                            t.terminal_error,
                                        );
                                    }
                                    if close_resp.clicked() {
                                        self.visible = false;
                                    }
                                },
                            );
                        });

                        ui.add_space(18.0);

                        section_label(ui, "THEME", t);
                        ui.add_space(8.0);

                        for variant in ThemeVariant::all() {
                            let selected = *variant == profile.theme;
                            let resp = theme_row(ui, variant.label(), selected, t);
                            if resp.clicked() && !selected {
                                profile.theme = *variant;
                                changed = true;
                            }
                        }

                        ui.add_space(18.0);
                        ui.painter().line_segment(
                            [ui.cursor().min, ui.cursor().min + egui::vec2(w - 48.0, 0.0)],
                            egui::Stroke::new(1.0, t.border),
                        );
                        ui.add_space(16.0);

                        section_label(ui, "EDITOR", t);
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Font size")
                                    .size(13.5)
                                    .color(t.tab_active_fg),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Slider::new(&mut profile.font_size, 10.0..=24.0)
                                                .step_by(1.0)
                                                .suffix(" px")
                                                .min_decimals(0)
                                                .max_decimals(0),
                                        )
                                        .changed()
                                    {
                                        changed = true;
                                    }
                                },
                            );
                        });

                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Line numbers")
                                    .size(13.5)
                                    .color(t.tab_active_fg),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.checkbox(&mut profile.show_line_numbers, "").changed() {
                                        changed = true;
                                    }
                                },
                            );
                        });
                    });
            });

        let clicked_outside = ctx.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .press_origin()
                    .map_or(false, |p| !panel_rect.contains(p))
        });
        if clicked_outside {
            self.visible = false;
        }

        if changed {
            profile.save();
        }
        changed
    }
}

fn section_label(ui: &mut egui::Ui, text: &str, t: Theme) {
    ui.label(
        egui::RichText::new(text)
            .size(10.0)
            .strong()
            .color(t.tab_inactive_fg),
    );
}

fn theme_row(ui: &mut egui::Ui, label: &str, selected: bool, t: Theme) -> egui::Response {
    let desired = egui::vec2(ui.available_width(), 34.0);
    let (rect, resp) = ui.allocate_exact_size(desired, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let bg = if selected {
            egui::Color32::from_rgba_premultiplied(t.accent.r(), t.accent.g(), t.accent.b(), 28)
        } else if resp.hovered() {
            t.button_hover_bg
        } else {
            egui::Color32::TRANSPARENT
        };

        ui.painter()
            .rect_filled(rect, egui::Rounding::same(5.0), bg);

        if selected {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.min.y + 7.0),
                    egui::vec2(3.0, rect.height() - 14.0),
                ),
                egui::Rounding::same(2.0),
                t.accent,
            );
        }

        let text_col = if selected { t.tab_active_fg } else { t.menu_fg };
        ui.painter().text(
            egui::pos2(rect.min.x + 16.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.5),
            text_col,
        );

        if selected {
            ui.painter().text(
                egui::pos2(rect.max.x - 14.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                ic::SUCCESS,
                egui::FontId::proportional(14.0),
                t.accent,
            );
        }
    }

    resp
}
