use crate::ui::editor::CodeEditor;
use crate::ui::icons as ic;
use crate::ui::theme::Theme;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

pub struct Tab {
    pub code: String,
    pub last_saved_code: String,
    pub current_file: Option<PathBuf>,
    pub editor: CodeEditor,
    pub output_rx: Option<Arc<Mutex<Vec<String>>>>,
    pub is_running: bool,
    pub id: usize,
}

static TAB_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Tab {
    fn next_id() -> usize {
        TAB_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn new(theme: Theme) -> Self {
        let code = String::from("!start\n# code here\n!end\n");
        Self {
            last_saved_code: code.clone(),
            code,
            current_file: None,
            editor: CodeEditor::new(theme),
            output_rx: None,
            is_running: false,
            id: Self::next_id(),
        }
    }

    pub fn from_file(path: PathBuf, content: String, theme: Theme) -> Self {
        Self {
            last_saved_code: content.clone(),
            code: content,
            current_file: Some(path),
            editor: CodeEditor::new(theme),
            output_rx: None,
            is_running: false,
            id: Self::next_id(),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.code != self.last_saved_code
    }

    pub fn display_name(&self) -> String {
        self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    pub fn is_pristine_new(&self) -> bool {
        self.current_file.is_none() && !self.is_dirty()
    }
}

pub enum TabBarAction {
    None,
    Activate(usize),
    Close(usize),
    New,
}

pub fn show_tab_bar(
    ctx: &eframe::egui::Context,
    tabs: &[Tab],
    active: usize,
    theme: &Theme,
) -> TabBarAction {
    use eframe::egui;

    let mut action = TabBarAction::None;
    let t = theme;

    egui::TopBottomPanel::top("tab_bar")
        .frame(
            egui::Frame::new()
                .fill(t.tab_bar_bg)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |ui| {
            ui.set_min_height(34.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                for (i, tab) in tabs.iter().enumerate() {
                    let is_active = i == active;
                    let tab_w = 160.0_f32;

                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(tab_w, 34.0), egui::Sense::hover());
                    let painter = ui.painter_at(rect);

                    painter.rect_filled(
                        rect,
                        egui::CornerRadius::ZERO,
                        if is_active {
                            t.tab_active_bg
                        } else {
                            t.tab_inactive_bg
                        },
                    );

                    if is_active {
                        painter.rect_filled(
                            egui::Rect::from_min_size(rect.min, egui::vec2(tab_w, 2.0)),
                            egui::CornerRadius::ZERO,
                            t.accent,
                        );
                    }

                    painter.line_segment(
                        [rect.right_top(), rect.right_bottom()],
                        egui::Stroke::new(1.0, t.border),
                    );

                    let close_r = egui::Rect::from_center_size(
                        egui::pos2(rect.right() - 16.0, rect.center().y),
                        egui::vec2(20.0, 20.0),
                    );
                    let close_resp = ui.interact(
                        close_r,
                        egui::Id::new(("tab_close", i)),
                        egui::Sense::click(),
                    );
                    let close_col = if close_resp.hovered() {
                        t.terminal_error
                    } else {
                        egui::Color32::from_rgba_premultiplied(
                            t.tab_inactive_fg.r(),
                            t.tab_inactive_fg.g(),
                            t.tab_inactive_fg.b(),
                            if is_active { 180 } else { 90 },
                        )
                    };
                    painter.text(
                        close_r.center(),
                        egui::Align2::CENTER_CENTER,
                        ic::TAB_CLOSE,
                        egui::FontId::proportional(13.0),
                        close_col,
                    );

                    let mut text_left = rect.left() + 10.0;
                    if tab.is_dirty() {
                        let dot_pos = egui::pos2(text_left + 4.0, rect.center().y);
                        painter.circle_filled(dot_pos, 3.0, t.tab_dirty_dot);
                        text_left += 12.0;
                    }

                    let name_rect = egui::Rect::from_min_max(
                        egui::pos2(text_left, rect.top()),
                        egui::pos2(close_r.left() - 2.0, rect.bottom()),
                    );
                    let name_col = if is_active {
                        t.tab_active_fg
                    } else {
                        t.tab_inactive_fg
                    };
                    painter.text(
                        egui::pos2(name_rect.left(), name_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        tab.display_name(),
                        egui::FontId::proportional(12.5),
                        name_col,
                    );

                    let body_rect = egui::Rect::from_min_max(
                        rect.min,
                        egui::pos2(close_r.left() - 2.0, rect.max.y),
                    );
                    if ui
                        .interact(
                            body_rect,
                            egui::Id::new(("tab_body", i)),
                            egui::Sense::click(),
                        )
                        .clicked()
                        && !is_active
                    {
                        action = TabBarAction::Activate(i);
                    }
                    if close_resp.clicked() {
                        action = TabBarAction::Close(i);
                    }
                }

                let new_btn_size = egui::vec2(34.0, 34.0);
                let (new_rect, new_resp) =
                    ui.allocate_exact_size(new_btn_size, egui::Sense::click());

                let new_bg = if new_resp.hovered() {
                    t.button_hover_bg
                } else {
                    egui::Color32::TRANSPARENT
                };
                ui.painter_at(new_rect)
                    .rect_filled(new_rect, egui::CornerRadius::same(4), new_bg);
                let new_col = if new_resp.hovered() {
                    t.tab_active_fg
                } else {
                    t.tab_inactive_fg
                };
                ui.painter_at(new_rect).text(
                    new_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    ic::TAB_NEW,
                    egui::FontId::proportional(15.0),
                    new_col,
                );

                if new_resp.clicked() {
                    action = TabBarAction::New;
                }
            });

            let r = ui.min_rect();
            ui.painter().line_segment(
                [r.left_bottom(), r.right_bottom()],
                egui::Stroke::new(1.0, t.border),
            );
        });

    action
}
