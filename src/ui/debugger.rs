use crate::compiler::parser::ParseNode;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

// ─── Value type (kept for AST eval preview only) ─────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FractalValue {
    Int(i64),
    Float(f64),
    Char(char),
    Bool(bool),
    Str(String),
    Array(Vec<FractalValue>),
    List(Vec<FractalValue>),
    Struct(HashMap<String, FractalValue>),
    Null,
    Void,
}

impl FractalValue {
    pub fn display(&self) -> String {
        match self {
            Self::Int(n) => n.to_string(),
            Self::Float(f) => format!("{:.4}", f),
            Self::Char(c) => format!("'{}'", c),
            Self::Bool(b) => b.to_string(),
            Self::Str(s) => format!("\"{}\"", s),
            Self::Null => "null".into(),
            Self::Void => "void".into(),
            Self::Array(v) | Self::List(v) => {
                let inner: Vec<String> = v.iter().map(|x| x.display()).collect();
                format!("[{}]", inner.join(", "))
            }
            Self::Struct(f) => {
                let p: Vec<String> = f
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.display()))
                    .collect();
                format!("{{{}}}", p.join(", "))
            }
        }
    }
    pub fn type_label(&self) -> &'static str {
        match self {
            Self::Int(_) => ":int",
            Self::Float(_) => ":float",
            Self::Char(_) => ":char",
            Self::Bool(_) => ":bool",
            Self::Str(_) => ":str",
            Self::Array(_) => ":array",
            Self::List(_) => ":list",
            Self::Struct(_) => ":struct",
            Self::Null => "null",
            Self::Void => "void",
        }
    }
}

// ─── AST Tree ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: usize,
    pub label: String,
    pub depth: usize,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub collapsed: bool,
}

pub fn build_tree_table(root: &ParseNode) -> Vec<TreeNode> {
    let mut t = Vec::new();
    visit_node(root, None, 0, &mut t);
    t
}

fn visit_node(
    node: &ParseNode,
    parent: Option<usize>,
    depth: usize,
    t: &mut Vec<TreeNode>,
) -> usize {
    let id = t.len();
    t.push(TreeNode {
        id,
        label: node_label(node),
        depth,
        children: vec![],
        parent,
        collapsed: false,
    });
    let kids: Vec<usize> = children_of(node)
        .into_iter()
        .map(|c| visit_node(c, Some(id), depth + 1, t))
        .collect();
    t[id].children = kids;
    id
}

use crate::compiler::parser::AccessStep;

fn children_of(n: &ParseNode) -> Vec<&ParseNode> {
    match n {
        ParseNode::Program(items) => items.iter().collect(),
        ParseNode::FuncDef {
            params,
            body,
            return_type,
            ..
        } => {
            let mut c: Vec<&ParseNode> = params.iter().collect();
            c.push(return_type);
            c.extend(body);
            c
        }
        ParseNode::StructDef { fields, .. } => fields.iter().collect(),
        ParseNode::StructDecl { init, .. } => init.as_deref().into_iter().collect(),
        ParseNode::Decl { init, .. } => init.as_deref().into_iter().collect(),
        ParseNode::Assign { lvalue, expr, .. } => vec![lvalue.as_ref(), expr.as_ref()],
        ParseNode::If {
            condition,
            then_block,
            else_block,
        } => {
            let mut c = vec![condition.as_ref()];
            c.extend(then_block);
            if let Some(eb) = else_block {
                c.extend(eb);
            }
            c
        }
        ParseNode::For {
            start,
            stop,
            step,
            body,
            ..
        } => {
            let mut c = vec![start.as_ref(), stop.as_ref(), step.as_ref()];
            c.extend(body);
            c
        }
        ParseNode::While { condition, body } => {
            let mut c = vec![condition.as_ref()];
            c.extend(body);
            c
        }
        ParseNode::Return(e) | ParseNode::Exit(e) => vec![e.as_ref()],
        ParseNode::ExprStmt(e) => vec![e.as_ref()],
        ParseNode::LogOr { left, right }
        | ParseNode::LogAnd { left, right }
        | ParseNode::BitOr { left, right }
        | ParseNode::BitXor { left, right }
        | ParseNode::BitAnd { left, right }
        | ParseNode::BitShift { left, right, .. }
        | ParseNode::Add { left, right, .. }
        | ParseNode::Mul { left, right, .. }
        | ParseNode::Cmp { left, right, .. } => vec![left.as_ref(), right.as_ref()],
        ParseNode::LogNot { operand } | ParseNode::Unary { operand, .. } => vec![operand.as_ref()],
        ParseNode::Cast { expr, .. } => vec![expr.as_ref()],
        ParseNode::ArrayLit(elems) => elems.iter().collect(),
        ParseNode::AccessChain { steps, .. } => steps
            .iter()
            .flat_map(|s| match s {
                AccessStep::Index(i) => vec![i.as_ref()],
                AccessStep::Call(a) => a.iter().collect(),
                _ => vec![],
            })
            .collect(),
        _ => vec![],
    }
}

fn node_label(n: &ParseNode) -> String {
    match n {
        ParseNode::Program(_) => "Program".into(),
        ParseNode::FuncDef {
            name, return_type, ..
        } => format!("FuncDef {}  → {}", name, type_str(return_type)),
        ParseNode::Param { data_type, name } => format!("Param {} : {}", name, type_str(data_type)),
        ParseNode::StructDef { name, .. } => format!("StructDef {}", name),
        ParseNode::StructDecl {
            var_name,
            struct_name,
            ..
        } => format!("StructDecl {} : {}", var_name, struct_name),
        ParseNode::Decl {
            data_type,
            name,
            init,
        } => format!(
            "Decl {} : {}{}",
            name,
            type_str(data_type),
            if init.is_some() { " =" } else { "" }
        ),
        ParseNode::Assign { op, .. } => format!("Assign {:?}", op),
        ParseNode::If { .. } => "If".into(),
        ParseNode::For { var_name, .. } => format!("For {}", var_name),
        ParseNode::While { .. } => "While".into(),
        ParseNode::Return(_) => "Return".into(),
        ParseNode::Exit(_) => "Exit".into(),
        ParseNode::Break => "Break".into(),
        ParseNode::Continue => "Continue".into(),
        ParseNode::ExprStmt(_) => "ExprStmt".into(),
        ParseNode::AccessChain { base, steps } => {
            let chain: String = steps
                .iter()
                .map(|s| match s {
                    AccessStep::Field(f) => format!("::{}", f),
                    AccessStep::Index(_) => "[…]".into(),
                    AccessStep::Call(a) => format!("({})", a.len()),
                })
                .collect();
            format!("Chain {}{}", base, chain)
        }
        ParseNode::LogOr { .. } => "LogOr".into(),
        ParseNode::LogAnd { .. } => "LogAnd".into(),
        ParseNode::LogNot { .. } => "LogNot".into(),
        ParseNode::Cmp { op, .. } => format!("Cmp {:?}", op),
        ParseNode::BitOr { .. } => "BitOr".into(),
        ParseNode::BitXor { .. } => "BitXor".into(),
        ParseNode::BitAnd { .. } => "BitAnd".into(),
        ParseNode::BitShift { op, .. } => format!("Shift {:?}", op),
        ParseNode::Add { op, .. } => format!("Add {:?}", op),
        ParseNode::Mul { op, .. } => format!("Mul {:?}", op),
        ParseNode::Unary { op, .. } => format!("Unary {:?}", op),
        ParseNode::Cast { target_type, .. } => format!("Cast → {}", type_str(target_type)),
        ParseNode::ArrayLit(e) => format!("ArrayLit [{}]", e.len()),
        ParseNode::StructLit(_) => "StructLit".into(),
        ParseNode::Identifier(s) => format!("Ident {}", s),
        ParseNode::IntLit(n) => format!("Int {}", n),
        ParseNode::FloatLit(f) => format!("Float {}", f),
        ParseNode::CharLit(c) => format!("Char '{}'", c),
        ParseNode::StringLit(s) => format!("Str \"{}\"", &s[..s.len().min(20)]),
        ParseNode::BoolLit(b) => format!("Bool {}", b),
        ParseNode::Null => "Null".into(),
        ParseNode::TypeInt => ":int".into(),
        ParseNode::TypeFloat => ":float".into(),
        ParseNode::TypeChar => ":char".into(),
        ParseNode::TypeBoolean => ":bool".into(),
        ParseNode::TypeVoid => ":void".into(),
        ParseNode::TypeArray { elem, size } => format!(":array<{},{}>", type_str(elem), size),
        ParseNode::TypeList { elem } => format!(":list<{}>", type_str(elem)),
        ParseNode::TypeStruct { name } => format!(":struct<{}>", name),
        _ => "Node".into(),
    }
}

fn type_str(n: &ParseNode) -> String {
    match n {
        ParseNode::TypeInt => ":int".into(),
        ParseNode::TypeFloat => ":float".into(),
        ParseNode::TypeChar => ":char".into(),
        ParseNode::TypeBoolean => ":bool".into(),
        ParseNode::TypeVoid => ":void".into(),
        ParseNode::TypeArray { elem, size } => format!(":array<{},{}>", type_str(elem), size),
        ParseNode::TypeList { elem } => format!(":list<{}>", type_str(elem)),
        ParseNode::TypeStruct { name } => format!(":struct<{}>", name),
        _ => "?".into(),
    }
}

// ─── Debug snapshot (read from file) ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DebugSnapshot {
    pub step: usize,
    pub label: String,
    pub source_line: usize,
    pub scopes: Vec<ScopeSnapshot>,
    pub call_stack: Vec<String>,
    pub output_since_last: String,
    pub finished: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScopeSnapshot {
    pub label: String,
    pub vars: Vec<VarRow>,
}

#[derive(Debug, Clone)]
pub struct VarRow {
    pub name: String,
    pub type_label: String,
    pub value: String,
    pub changed: bool,
}

#[derive(Debug, Clone)]
pub struct DebugFrame {
    pub active_node_id: usize,
    pub step_label: String,
    pub source_line: usize,
    pub scopes: Vec<ScopeSnapshot>,
    pub call_stack: Vec<String>,
    pub finished: bool,
    pub error: Option<String>,
    pub buffered_output: String,
}

// ─── Debug session ────────────────────────────────────────────────────────────

pub struct DebugSession {
    debug_file: PathBuf,
    snapshots: Vec<DebugSnapshot>,
    cursor: usize,
    file_offset: u64,
    pub tree: Vec<TreeNode>,
    pub finished: bool,
    /// Maps snapshot label (e.g. "Decl x", "If", "For i") → tree node id.
    /// Built once from the tree on construction, used to highlight the
    /// correct AST node as the debugger steps through snapshots.
    label_to_node: HashMap<String, usize>,
}

impl DebugSession {
    pub fn new(root: &ParseNode, debug_file: PathBuf) -> Self {
        let tree = build_tree_table(root);

        // Build label → node-id lookup.
        // We do a *first-match* insert so that for duplicate labels (e.g. two
        // "If" nodes) the shallowest / earliest occurrence wins initially.
        // The real match improves incrementally as the step counter advances
        // (see `find_node_for_label` below).
        let mut label_to_node: HashMap<String, usize> = HashMap::new();
        for node in &tree {
            label_to_node.entry(node.label.clone()).or_insert(node.id);
        }

        Self {
            debug_file,
            snapshots: Vec::new(),
            cursor: 0,
            file_offset: 0,
            tree,
            finished: false,
            label_to_node,
        }
    }

    /// Poll the debug file for new snapshots without advancing the cursor.
    pub fn poll_file(&mut self) {
        let Ok(content) = fs::read_to_string(&self.debug_file) else {
            return;
        };
        let offset = self.file_offset as usize;
        if offset > content.len() {
            return;
        }
        let new_part = &content[offset..];
        if new_part.is_empty() {
            return;
        }

        // Walk line by line, tracking exact byte positions so the offset
        // is always correct regardless of blank/unparseable lines.
        let mut consumed = 0usize;
        for raw_line in new_part.lines() {
            // Account for the newline character(s)
            let line_bytes = raw_line.len() + 1; // +1 for '\n'
            let trimmed = raw_line.trim();
            if !trimmed.is_empty() {
                if let Some(snap) = parse_snapshot_line(trimmed) {
                    let was_finished = snap.finished;
                    self.snapshots.push(snap);
                    if was_finished {
                        self.finished = true;
                    }
                }
            }
            consumed += line_bytes;
        }
        self.file_offset = (offset + consumed) as u64;
    }

    pub fn total_steps(&self) -> usize {
        self.snapshots.len()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn steps_available(&self) -> usize {
        self.snapshots.len().saturating_sub(self.cursor)
    }

    pub fn current_frame(&self) -> DebugFrame {
        if self.snapshots.is_empty() {
            return placeholder_frame();
        }
        let idx = self.cursor.saturating_sub(1).min(self.snapshots.len() - 1);
        self.snap_to_frame(&self.snapshots[idx])
    }

    /// Advance cursor by one and return the new frame.
    /// Returns None if no new snapshot is available yet.
    pub fn step(&mut self) -> Option<DebugFrame> {
        if self.cursor >= self.snapshots.len() {
            return None;
        }
        let frame = self.snap_to_frame(&self.snapshots[self.cursor]);
        self.cursor += 1;
        if frame.finished {
            self.finished = true;
        }
        Some(frame)
    }

    pub fn toggle_collapsed(&mut self, node_id: usize) {
        if let Some(n) = self.tree.get_mut(node_id) {
            n.collapsed = !n.collapsed;
        }
    }

    pub fn collapse_all(&mut self, node_id: usize) {
        let children: Vec<usize> = self
            .tree
            .get(node_id)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        if let Some(n) = self.tree.get_mut(node_id) {
            n.collapsed = true;
        }
        for c in children {
            self.collapse_all(c);
        }
    }

    pub fn reveal_node(&mut self, node_id: usize) {
        let mut cur = Some(node_id);
        while let Some(id) = cur {
            if let Some(n) = self.tree.get_mut(id) {
                n.collapsed = false;
                cur = n.parent;
            } else {
                break;
            }
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Convert a snapshot into a DebugFrame, resolving the active AST node.
    fn snap_to_frame(&self, snap: &DebugSnapshot) -> DebugFrame {
        let active_node_id = self.find_node_for_label(&snap.label, snap.step);
        DebugFrame {
            active_node_id,
            step_label: snap.label.clone(),
            source_line: snap.source_line,
            scopes: snap.scopes.clone(),
            call_stack: snap.call_stack.clone(),
            finished: snap.finished,
            error: snap.error.clone(),
            buffered_output: snap.output_since_last.clone(),
        }
    }

    /// Find the best-matching tree node id for a snapshot label.
    ///
    /// Strategy: the snapshot labels match the tree node labels exactly
    /// (both are produced by the same `stmt_debug_label` / `node_label`
    /// logic).  When there are multiple nodes with the same label (e.g. two
    /// "If" statements) we pick the one whose *position* in the flat tree
    /// array is closest to `step` — a cheap heuristic that works well for
    /// sequential programs.
    fn find_node_for_label(&self, label: &str, step: usize) -> usize {
        // Collect all node ids whose label matches.
        let candidates: Vec<usize> = self
            .tree
            .iter()
            .filter(|n| n.label == label)
            .map(|n| n.id)
            .collect();

        match candidates.len() {
            0 => {
                // No exact match — fall back to the first-match map.
                self.label_to_node.get(label).copied().unwrap_or(0)
            }
            1 => candidates[0],
            _ => {
                // Multiple matches: pick the id numerically closest to `step`.
                candidates
                    .into_iter()
                    .min_by_key(|&id| (id as isize - step as isize).unsigned_abs())
                    .unwrap_or(0)
            }
        }
    }
}

fn placeholder_frame() -> DebugFrame {
    DebugFrame {
        active_node_id: 0,
        step_label: "Waiting for debug output…".into(),
        source_line: 0,
        scopes: vec![],
        call_stack: vec!["main".into()],
        finished: false,
        error: None,
        buffered_output: String::new(),
    }
}

// ─── JSON snapshot parser ─────────────────────────────────────────────────────

fn parse_snapshot_line(line: &str) -> Option<DebugSnapshot> {
    let step = extract_u64(line, "\"step\"")?;
    let label = extract_str(line, "\"label\"").unwrap_or_default();
    let source_line = extract_u64(line, "\"line\"").unwrap_or(0) as usize;
    let output = extract_str(line, "\"output\"").unwrap_or_default();
    let finished = extract_bool(line, "\"finished\"").unwrap_or(false);
    let error = extract_nullable_str(line, "\"error\"");
    let call_stack = extract_str_array(line, "\"stack\"");
    let scopes = extract_scopes(line);

    Some(DebugSnapshot {
        step: step as usize,
        label,
        source_line,
        scopes,
        call_stack,
        output_since_last: output,
        finished,
        error,
    })
}

fn extract_u64(s: &str, key: &str) -> Option<u64> {
    let pos = s.find(key)?;
    let after = &s[pos + key.len()..];
    let colon = after.find(':')? + 1;
    let digits: String = after[colon..]
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

fn extract_bool(s: &str, key: &str) -> Option<bool> {
    let pos = s.find(key)?;
    let after = &s[pos + key.len()..];
    let colon = after.find(':')? + 1;
    let val: String = after[colon..]
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_alphabetic())
        .collect();
    match val.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn extract_str(s: &str, key: &str) -> Option<String> {
    let pos = s.find(key)?;
    let after = &s[pos + key.len()..];
    let colon = after.find(':')? + 1;
    let rest = after[colon..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    Some(parse_json_string(&rest[1..]))
}

fn extract_nullable_str(s: &str, key: &str) -> Option<String> {
    let pos = s.find(key)?;
    let after = &s[pos + key.len()..];
    let colon = after.find(':')? + 1;
    let rest = after[colon..].trim_start();
    if rest.starts_with("null") {
        return None;
    }
    if rest.starts_with('"') {
        return Some(parse_json_string(&rest[1..]));
    }
    None
}

fn parse_json_string(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '"' {
            break;
        }
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => break,
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn extract_str_array(s: &str, key: &str) -> Vec<String> {
    let Some(pos) = s.find(key) else {
        return vec![];
    };
    let after = &s[pos + key.len()..];
    let Some(colon) = after.find(':') else {
        return vec![];
    };
    let rest = after[colon + 1..].trim_start();
    if !rest.starts_with('[') {
        return vec![];
    }
    let Some(end) = rest.find(']') else {
        return vec![];
    };
    let inner = &rest[1..end];
    let mut result = Vec::new();
    for part in inner.split(',') {
        let p = part.trim();
        if p.starts_with('"') && p.ends_with('"') && p.len() >= 2 {
            result.push(p[1..p.len() - 1].to_string());
        }
    }
    result
}

fn extract_scopes(s: &str) -> Vec<ScopeSnapshot> {
    let Some(pos) = s.find("\"scopes\"") else {
        return vec![];
    };
    let after = &s[pos + 8..];
    let Some(colon) = after.find(':') else {
        return vec![];
    };
    let rest = after[colon + 1..].trim_start();
    if !rest.starts_with('[') {
        return vec![];
    }

    let mut scopes = Vec::new();
    let bytes = rest.as_bytes();
    let mut i = 1usize;
    let mut depth = 0i32;
    let mut obj_start = None;

    while i < bytes.len() {
        match bytes[i] as char {
            '{' => {
                if depth == 0 {
                    obj_start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = obj_start {
                        let obj = &rest[start..=i];
                        if let Some(sc) = parse_scope_object(obj) {
                            scopes.push(sc);
                        }
                        obj_start = None;
                    }
                }
            }
            ']' if depth == 0 => break,
            _ => {}
        }
        i += 1;
    }
    scopes
}

fn parse_scope_object(obj: &str) -> Option<ScopeSnapshot> {
    let label = extract_str(obj, "\"label\"").unwrap_or_default();
    let vars = extract_vars(obj);
    Some(ScopeSnapshot { label, vars })
}

fn extract_vars(obj: &str) -> Vec<VarRow> {
    let Some(pos) = obj.find("\"vars\"") else {
        return vec![];
    };
    let after = &obj[pos + 6..];
    let Some(colon) = after.find(':') else {
        return vec![];
    };
    let rest = after[colon + 1..].trim_start();
    if !rest.starts_with('[') {
        return vec![];
    }

    let mut vars = Vec::new();
    let bytes = rest.as_bytes();
    let mut i = 1usize;
    let mut depth = 0i32;
    let mut obj_start = None;

    while i < bytes.len() {
        match bytes[i] as char {
            '{' => {
                if depth == 0 {
                    obj_start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = obj_start {
                        let var_obj = &rest[start..=i];
                        if let Some(row) = parse_var_object(var_obj) {
                            vars.push(row);
                        }
                        obj_start = None;
                    }
                }
            }
            ']' if depth == 0 => break,
            _ => {}
        }
        i += 1;
    }
    vars
}

fn parse_var_object(obj: &str) -> Option<VarRow> {
    let name = extract_str(obj, "\"name\"")?;
    let type_label = extract_str(obj, "\"type\"").unwrap_or_default();
    let value = extract_str(obj, "\"value\"").unwrap_or_default();
    let changed = extract_bool(obj, "\"changed\"").unwrap_or(false);
    Some(VarRow {
        name,
        type_label,
        value,
        changed,
    })
}