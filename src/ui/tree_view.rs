use super::debugger::DebugSession;
use super::theme::Theme;
use eframe::egui;

#[allow(unused_imports)]
pub struct TreeViewWindow {
    pub open: bool,
    pub title: String,

    last_scrolled_node: Option<usize>,
}

impl TreeViewWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            title: "AST Tree".into(),
            last_scrolled_node: None,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        session: &mut DebugSession,
        active_node_id: usize,
        theme: &Theme,
    ) {
        if !self.open {
            self.last_scrolled_node = None;
            return;
        }

        let node_changed = self.last_scrolled_node != Some(active_node_id);
        if node_changed {
            session.reveal_node(active_node_id);
            self.last_scrolled_node = Some(active_node_id);
        }

        let t = *theme;
        let mut open = self.open;

        egui::Window::new("Abstract Syntax Tree")
            .id(egui::Id::new("fractal_tree_view"))
            .open(&mut open)
            .default_size([460.0, 580.0])
            .min_size([300.0, 200.0])
            .resizable(true)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(t.panel_bg)
                    .stroke(egui::Stroke::new(1.0, t.border))
                    .inner_margin(egui::Margin::ZERO),
            )
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(t.tab_bar_bg)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("⬡  Abstract Syntax Tree")
                                    .size(12.5)
                                    .color(t.tab_active_fg)
                                    .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let total = session.total_steps();
                                    let cur = session.cursor();
                                    ui.label(
                                        egui::RichText::new(format!("step {cur}/{total}"))
                                            .size(10.5)
                                            .color(t.tab_inactive_fg),
                                    );
                                    ui.add_space(8.0);
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("⊟ Collapse all")
                                                    .size(10.5)
                                                    .color(t.tab_inactive_fg),
                                            )
                                            .fill(egui::Color32::TRANSPARENT)
                                            .stroke(egui::Stroke::new(1.0, t.border)),
                                        )
                                        .clicked()
                                    {
                                        session.collapse_all(0);
                                        if let Some(root) = session.tree.get_mut(0) {
                                            root.collapsed = false;
                                        }
                                    }
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

                let mut toggle_id: Option<usize> = None;
                let mut scroll_to: Option<egui::Rect> = None;

                egui::ScrollArea::both()
                    .id_salt("tree_view_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(6.0);

                        ui.set_min_width(380.0);

                        draw_subtree(
                            ui,
                            &session.tree,
                            0,
                            active_node_id,
                            &t,
                            true,
                            &mut toggle_id,
                            &mut scroll_to,
                        );

                        ui.add_space(12.0);

                        if let Some(id) = toggle_id {
                            session.toggle_collapsed(id);
                        }

                        if node_changed {
                            if let Some(rect) = scroll_to {
                                ui.scroll_to_rect(
                                    rect.expand2(egui::vec2(0.0, 60.0)),
                                    Some(egui::Align::Center),
                                );
                            }
                        }
                    });
            });

        self.open = open;
    }
}

const INDENT_W: f32 = 20.0;
const ROW_H: f32 = 24.0;
const TRUNK_X: f32 = 8.0;

fn draw_subtree(
    ui: &mut egui::Ui,
    table: &[super::debugger::TreeNode],
    id: usize,
    active_node_id: usize,
    t: &Theme,
    is_last: bool,
    toggle_out: &mut Option<usize>,
    scroll_to: &mut Option<egui::Rect>,
) {
    let Some(node) = table.get(id) else { return };

    let depth = node.depth;
    let is_active = id == active_node_id;
    let has_children = !node.children.is_empty();
    let is_collapsed = node.collapsed;

    let avail_w = ui.available_width().max(380.0);
    let (row_rect, _) = ui.allocate_exact_size(egui::vec2(avail_w, ROW_H), egui::Sense::hover());

    let origin = row_rect.left();

    if is_active {
        ui.painter()
            .rect_filled(row_rect, egui::CornerRadius::same(3), t.accent);

        ui.painter().rect_filled(
            egui::Rect::from_min_size(row_rect.min, egui::vec2(3.0, ROW_H)),
            egui::CornerRadius::same(1),
            egui::Color32::from_rgba_premultiplied(0, 0, 0, 60),
        );
        *scroll_to = Some(row_rect);
    } else if ui.rect_contains_pointer(row_rect) {
        ui.painter().rect_filled(
            row_rect,
            egui::CornerRadius::same(3),
            egui::Color32::from_rgba_premultiplied(
                t.button_hover_bg.r(),
                t.button_hover_bg.g(),
                t.button_hover_bg.b(),
                100,
            ),
        );
    }

    let mid_y = row_rect.center().y;
    let line_col =
        egui::Color32::from_rgba_premultiplied(t.border.r(), t.border.g(), t.border.b(), 160);

    if depth > 0 {
        let branch_x = origin + (depth as f32 - 1.0) * INDENT_W + TRUNK_X;
        let branch_end = branch_x + INDENT_W - 4.0;

        ui.painter().line_segment(
            [
                egui::pos2(branch_x, row_rect.top()),
                egui::pos2(branch_x, mid_y),
            ],
            egui::Stroke::new(1.0, line_col),
        );

        if !is_last {
            ui.painter().line_segment(
                [
                    egui::pos2(branch_x, mid_y),
                    egui::pos2(branch_x, row_rect.bottom()),
                ],
                egui::Stroke::new(1.0, line_col),
            );
        }

        ui.painter().line_segment(
            [egui::pos2(branch_x, mid_y), egui::pos2(branch_end, mid_y)],
            egui::Stroke::new(1.0, line_col),
        );
    }

    if depth > 1 {
        let mut cur_id = id;
        loop {
            let cur = match table.get(cur_id) {
                Some(n) => n,
                None => break,
            };
            let parent_id = match cur.parent {
                Some(p) => p,
                None => break,
            };
            let parent = match table.get(parent_id) {
                Some(n) => n,
                None => break,
            };

            let is_last_of_parent = parent.children.last() == Some(&cur_id);

            if !is_last_of_parent && parent.depth < depth.saturating_sub(1) {
                let trunk_x = origin + parent.depth as f32 * INDENT_W + TRUNK_X;
                ui.painter().line_segment(
                    [
                        egui::pos2(trunk_x, row_rect.top()),
                        egui::pos2(trunk_x, row_rect.bottom()),
                    ],
                    egui::Stroke::new(1.0, line_col),
                );
            }

            if parent.depth == 0 {
                break;
            }
            cur_id = parent_id;
        }
    }

    let toggle_center_x = if depth == 0 {
        origin + TRUNK_X
    } else {
        origin + depth as f32 * INDENT_W
    };

    if has_children {
        let toggle_size = 16.0;
        let toggle_rect = egui::Rect::from_center_size(
            egui::pos2(toggle_center_x, mid_y),
            egui::vec2(toggle_size, toggle_size),
        );
        let toggle_resp = ui.interact(
            toggle_rect,
            egui::Id::new(("tree_toggle", id)),
            egui::Sense::click(),
        );
        if toggle_resp.hovered() {
            ui.painter().rect_filled(
                toggle_rect,
                egui::CornerRadius::same(3),
                egui::Color32::from_rgba_premultiplied(
                    t.accent.r(),
                    t.accent.g(),
                    t.accent.b(),
                    45,
                ),
            );
        }
        let arrow = if is_collapsed { "▶" } else { "▼" };

        let arrow_col = if is_active {
            match t.variant {
                crate::ui::theme::ThemeVariant::Dark => egui::Color32::WHITE,
                crate::ui::theme::ThemeVariant::Light => egui::Color32::from_rgb(20, 20, 40),
            }
        } else {
            t.tab_inactive_fg
        };
        ui.painter().text(
            toggle_rect.center(),
            egui::Align2::CENTER_CENTER,
            arrow,
            egui::FontId::proportional(9.0),
            arrow_col,
        );
        if toggle_resp.clicked() {
            *toggle_out = Some(id);
        }
    } else {
        let dot_col = if is_active {
            match t.variant {
                crate::ui::theme::ThemeVariant::Dark => egui::Color32::WHITE,
                crate::ui::theme::ThemeVariant::Light => egui::Color32::from_rgb(20, 20, 40),
            }
        } else {
            line_col
        };
        ui.painter()
            .circle_filled(egui::pos2(toggle_center_x, mid_y), 2.5, dot_col);
    }

    let label_x = toggle_center_x + 12.0;
    let mut text_x = label_x;

    let badge = node_kind_badge(&node.label);
    let badge_bg = node_badge_color(&node.label, t);

    if !badge.is_empty() {
        let badge_w = (badge.len() as f32 * 6.5 + 8.0).max(34.0);
        let badge_rect =
            egui::Rect::from_min_size(egui::pos2(text_x, mid_y - 8.0), egui::vec2(badge_w, 16.0));
        ui.painter()
            .rect_filled(badge_rect, egui::CornerRadius::same(3), badge_bg);

        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            badge,
            egui::FontId::monospace(9.0),
            t.editor_bg,
        );
        text_x += badge_w + 6.0;
    }

    let label_font = egui::FontId::monospace(if is_active { 11.5 } else { 11.0 });
    let text_col = node_text_color(is_active, t, depth, has_children);

    let full_label = &node.label;
    let display_label = if full_label.len() > 56 {
        format!("{}…", &full_label[..54])
    } else {
        full_label.clone()
    };

    ui.painter().text(
        egui::pos2(text_x, mid_y),
        egui::Align2::LEFT_CENTER,
        &display_label,
        label_font.clone(),
        text_col,
    );

    if is_collapsed && has_children {
        let char_w = ui.fonts_mut(|f| f.glyph_width(&label_font, 'x')).max(1.0);
        let label_w = char_w * display_label.len() as f32;
        ui.painter().text(
            egui::pos2(text_x + label_w + 4.0, mid_y),
            egui::Align2::LEFT_CENTER,
            format!("+{} hidden", node.children.len()),
            egui::FontId::proportional(9.5),
            t.tab_inactive_fg,
        );
    }

    let body_resp = ui.interact(
        egui::Rect::from_min_max(egui::pos2(label_x, row_rect.top()), row_rect.max),
        egui::Id::new(("tree_body", id)),
        egui::Sense::click(),
    );
    if body_resp.clicked() && has_children && toggle_out.is_none() {
        *toggle_out = Some(id);
    }

    if !is_collapsed {
        let children: Vec<usize> = node.children.clone();
        let n = children.len();
        for (i, &child_id) in children.iter().enumerate() {
            draw_subtree(
                ui,
                table,
                child_id,
                active_node_id,
                t,
                i == n - 1,
                toggle_out,
                scroll_to,
            );
        }
    }
}

fn node_text_color(is_active: bool, t: &Theme, depth: usize, has_children: bool) -> egui::Color32 {
    if is_active {
        return match t.variant {
            crate::ui::theme::ThemeVariant::Dark => egui::Color32::WHITE,
            crate::ui::theme::ThemeVariant::Light => egui::Color32::from_rgb(20, 20, 40),
        };
    }
    if depth == 0 {
        t.tab_active_fg
    } else if !has_children {
        t.tab_inactive_fg
    } else {
        t.text_default
    }
}

fn node_badge_color(label: &str, t: &Theme) -> egui::Color32 {
    match node_kind(label) {
        NodeKind::Decl => t.type_name,
        NodeKind::Control => t.keyword,
        NodeKind::Expr => t.fn_name,
        NodeKind::Literal => t.number,
        NodeKind::Type => t.type_name,
        NodeKind::Structural => t.struct_name,
        NodeKind::Other => t.tab_inactive_fg,
    }
}

fn node_kind_badge(label: &str) -> &'static str {
    match node_kind(label) {
        NodeKind::Decl => "DECL",
        NodeKind::Control => "CTRL",
        NodeKind::Expr => "EXPR",
        NodeKind::Literal => "LIT",
        NodeKind::Type => "TYPE",
        NodeKind::Structural => "STRUCT",
        NodeKind::Other => "",
    }
}

enum NodeKind {
    Decl,
    Control,
    Expr,
    Literal,
    Type,
    Structural,
    Other,
}

fn node_kind(label: &str) -> NodeKind {
    if label.starts_with("Decl") || label.starts_with("StructDecl") {
        NodeKind::Decl
    } else if label.starts_with("If")
        || label.starts_with("While")
        || label.starts_with("For")
        || label.starts_with("Return")
        || label.starts_with("Exit")
        || label.starts_with("Break")
        || label.starts_with("Continue")
    {
        NodeKind::Control
    } else if label.starts_with("Add")
        || label.starts_with("Mul")
        || label.starts_with("Cmp")
        || label.starts_with("Log")
        || label.starts_with("Bit")
        || label.starts_with("Shift")
        || label.starts_with("Unary")
        || label.starts_with("Cast")
        || label.starts_with("Assign")
        || label.starts_with("Chain")
        || label.starts_with("ExprStmt")
    {
        NodeKind::Expr
    } else if label.starts_with("Int")
        || label.starts_with("Float")
        || label.starts_with("Char")
        || label.starts_with("Bool")
        || label.starts_with("Str")
        || label.starts_with("Null")
        || label.starts_with("Ident")
        || label.starts_with("ArrayLit")
        || label.starts_with("StructLit")
    {
        NodeKind::Literal
    } else if label.starts_with(':') || label.starts_with("Type") {
        NodeKind::Type
    } else if label.starts_with("StructDef")
        || label.starts_with("FuncDef")
        || label.starts_with("Param")
        || label.starts_with("Program")
    {
        NodeKind::Structural
    } else {
        NodeKind::Other
    }
}
