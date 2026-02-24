use super::theme::Theme;
use eframe::egui;

pub struct Terminal {
    pub output: String,
    pub minimized: bool,
    panel_height: f32,
    theme: Theme,
}

impl Terminal {
    pub fn new(theme: Theme) -> Self {
        Self {
            output: String::new(),
            minimized: false,
            panel_height: 200.0,
            theme,
        }
    }

    pub fn append(&mut self, text: &str) {
        self.output.push_str(text);
        self.minimized = false;
    }

    pub fn clear(&mut self) {
        self.output.clear();
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        let header_height = 28.0;

        egui::TopBottomPanel::bottom("terminal_panel")
            .resizable(!self.minimized)
            .min_height(if self.minimized { header_height } else { 80.0 })
            .max_height(600.0)
            .default_height(if self.minimized {
                header_height
            } else {
                self.panel_height
            })
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let icon = if self.minimized { "▲" } else { "▼" };
                    if ui.small_button(icon).clicked() {
                        self.minimized = !self.minimized;
                    }
                    ui.label(
                        egui::RichText::new("Terminal")
                            .monospace()
                            .color(self.theme.identifier),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✖  Clear").clicked() {
                            self.output.clear();
                        }
                    });
                });

                if self.minimized {
                    return;
                }

                ui.separator();

                let bg = egui::Color32::from_rgb(18, 18, 18);
                egui::Frame::none()
                    .fill(bg)
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                ui.style_mut().override_text_style =
                                    Some(egui::TextStyle::Monospace);

                                if self.output.is_empty() {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(80, 80, 80),
                                        "No output yet. Press ▶ Run to compile and execute.",
                                    );
                                } else {
                                    for line in self.output.lines() {
                                        let lower = line.to_ascii_lowercase();
                                        let color = if lower.contains("error") {
                                            egui::Color32::from_rgb(255, 100, 100)
                                        } else if lower.contains("warning") {
                                            egui::Color32::from_rgb(255, 210, 80)
                                        } else {
                                            self.theme.text_default
                                        };
                                        ui.colored_label(color, line);
                                    }
                                }
                            });
                    });

                self.panel_height = ui.min_rect().height().max(80.0);
            });
    }
}
