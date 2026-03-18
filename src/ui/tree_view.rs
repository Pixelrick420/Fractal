use super::debugger::{DebugSession, TreeNode};
use super::theme::Theme;
use eframe::egui;

pub struct TreeViewWindow {
    pub open: bool,
    pub title: String,
}

impl TreeViewWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            title: "AST Tree".into(),
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        session: &DebugSession,
        active_node_id: usize,
        theme: &Theme,
    ) {
        if !self.open {
            return;
        }

        let t = theme;

        egui::Window::new(&self.title)
            .id(egui::Id::new("fractal_tree_view_window"))
            .default_size([420.0, 560.0])
            .min_size([280.0, 200.0])
            .resizable(true)
            .collapsible(true)
            .frame(
                egui::Frame::new()
                    .fill(t.panel_bg)
                    .stroke(egui::Stroke::new(1.0, t.border))
                    .corner_radius(egui::CornerRadius::same(8))
                    .shadow(egui::Shadow {
                        offset: [0, 6],
                        blur: 18,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(100),
                    })
                    .inner_margin(egui::Margin::same(0.0 as i8 as i8)),
            )
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(t.tab_bar_bg)
                    .inner_margin(egui::Margin::symmetric(12.0 as i8 as i8, 8.0 as i8 as i8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("⬡  AST Tree")
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
                                        egui::RichText::new(format!("step {}/{}", cur, total))
                                            .size(10.5)
                                            .color(t.tab_inactive_fg),
                                    );
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

                egui::ScrollArea::both()
                    .id_salt("tree_view_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(6.0);
                        egui::Frame::new()
                            .inner_margin(egui::Margin::symmetric(
                                10.0 as i8 as i8,
                                0.0 as i8 as i8,
                            ))
                            .show(ui, |ui| {
                                draw_tree(ui, &session.tree, 0, active_node_id, t);
                            });
                        ui.add_space(6.0);
                    });
            });
    }
}

fn draw_tree(
    ui: &mut egui::Ui,
    table: &[TreeNode],
    root_id: usize,
    active_node_id: usize,
    t: &Theme,
) {
    draw_node(ui, table, root_id, active_node_id, t, true);
}

fn draw_node(
    ui: &mut egui::Ui,
    table: &[TreeNode],
    id: usize,
    active_node_id: usize,
    t: &Theme,
    is_last: bool,
) {
    let Some(node) = table.get(id) else { return };

    let indent = node.depth as f32 * 16.0;
    let row_h = 22.0_f32;
    let is_active = id == active_node_id;

    let (row_rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), row_h),
        egui::Sense::hover(),
    );

    if is_active {
        ui.painter().rect_filled(
            row_rect,
            egui::CornerRadius::same(4),
            egui::Color32::from_rgba_premultiplied(t.accent.r(), t.accent.g(), t.accent.b(), 38),
        );

        ui.painter().rect_filled(
            egui::Rect::from_min_size(row_rect.min, egui::vec2(3.0, row_h)),
            egui::CornerRadius::same(2),
            t.accent,
        );
    }

    let connector_x = row_rect.left() + indent + 8.0;
    let mid_y = row_rect.center().y;

    if node.depth > 0 {
        ui.painter().line_segment(
            [
                egui::pos2(connector_x - 8.0, mid_y),
                egui::pos2(connector_x + 4.0, mid_y),
            ],
            egui::Stroke::new(1.0, t.border),
        );

        let top_y = row_rect.top();
        ui.painter().line_segment(
            [
                egui::pos2(connector_x - 8.0, top_y),
                egui::pos2(connector_x - 8.0, mid_y),
            ],
            egui::Stroke::new(1.0, t.border),
        );
        if !is_last {
            ui.painter().line_segment(
                [
                    egui::pos2(connector_x - 8.0, mid_y),
                    egui::pos2(connector_x - 8.0, row_rect.bottom()),
                ],
                egui::Stroke::new(1.0, t.border),
            );
        }
    }

    let has_children = !node.children.is_empty();
    if has_children {
        let dot_center = egui::pos2(connector_x + 6.0, mid_y);
        ui.painter()
            .circle_filled(dot_center, 3.5, if is_active { t.accent } else { t.border });
    }

    let label_x = connector_x + 16.0;
    let (text_col, bg) = node_colors(node, is_active, t);

    let badge = node_kind_badge(&node.label);
    let badge_w = 44.0_f32;

    if !badge.is_empty() {
        let badge_rect =
            egui::Rect::from_min_size(egui::pos2(label_x, mid_y - 8.0), egui::vec2(badge_w, 16.0));
        ui.painter()
            .rect_filled(badge_rect, egui::CornerRadius::same(3), bg);
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            badge,
            egui::FontId::monospace(9.5),
            t.tab_bar_bg,
        );
    }

    let label_start = label_x + if badge.is_empty() { 0.0 } else { badge_w + 5.0 };
    ui.painter().text(
        egui::pos2(label_start, mid_y),
        egui::Align2::LEFT_CENTER,
        &node.label,
        egui::FontId::monospace(11.5),
        text_col,
    );

    let n = node.children.len();
    for (i, &child_id) in node.children.iter().enumerate() {
        draw_node(ui, table, child_id, active_node_id, t, i == n - 1);
    }
}

fn node_colors(node: &TreeNode, is_active: bool, t: &Theme) -> (egui::Color32, egui::Color32) {
    if is_active {
        return (t.accent, t.accent);
    }
    let label = &node.label;
    let badge_bg = match node_kind(label) {
        NodeKind::Decl => t.type_name,
        NodeKind::Control => t.keyword,
        NodeKind::Expr => t.fn_name,
        NodeKind::Literal => t.number,
        NodeKind::Type => t.type_name,
        NodeKind::Structural => t.struct_name,
        NodeKind::Other => t.tab_inactive_fg,
    };
    let text_col = if node.depth == 0 {
        t.tab_active_fg
    } else if node.children.is_empty() {
        t.tab_inactive_fg
    } else {
        t.tab_active_fg
    };
    (text_col, badge_bg)
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
    let l = label;
    if l.starts_with("Decl") || l.starts_with("StructDecl") {
        NodeKind::Decl
    } else if l.starts_with("If")
        || l.starts_with("While")
        || l.starts_with("For")
        || l.starts_with("Return")
        || l.starts_with("Exit")
        || l.starts_with("Break")
        || l.starts_with("Continue")
    {
        NodeKind::Control
    } else if l.starts_with("Add")
        || l.starts_with("Mul")
        || l.starts_with("Cmp")
        || l.starts_with("Log")
        || l.starts_with("Bit")
        || l.starts_with("Shift")
        || l.starts_with("Unary")
        || l.starts_with("Cast")
        || l.starts_with("Assign")
        || l.starts_with("Chain")
        || l.starts_with("ExprStmt")
    {
        NodeKind::Expr
    } else if l.starts_with("Int")
        || l.starts_with("Float")
        || l.starts_with("Char")
        || l.starts_with("Bool")
        || l.starts_with("Str")
        || l.starts_with("Null")
        || l.starts_with("Ident")
        || l.starts_with("ArrayLit")
        || l.starts_with("StructLit")
    {
        NodeKind::Literal
    } else if l.starts_with(':') || l.starts_with("Type") {
        NodeKind::Type
    } else if l.starts_with("StructDef")
        || l.starts_with("FuncDef")
        || l.starts_with("Param")
        || l.starts_with("Program")
    {
        NodeKind::Structural
    } else {
        NodeKind::Other
    }
}
