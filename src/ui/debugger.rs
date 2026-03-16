// src/ui/debugger.rs
//
// Tree-walking step debugger for Fractal.
//
// Public surface:
//   DebugSession::new(root)  → builds flat step list + tree table
//   session.step()           → executes one step, returns DebugFrame
//   session.current_frame()  → peek without advancing
//   session.tree             → Vec<TreeNode> consumed by tree_view.rs
//   session.finished         → true once program ends

use std::collections::HashMap;
use crate::compiler::parser::{
    AccessStep, AddOp, AssignOp, CmpOp, MulOp, ParseNode, ShiftOp, UnOp,
};

// ─────────────────────────────────────────────────────────────────────────────
// Runtime value
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FractalValue {
    Int(i64), Float(f64), Char(char), Bool(bool),
    Str(String), Array(Vec<FractalValue>), List(Vec<FractalValue>),
    Struct(HashMap<String, FractalValue>), Null, Void,
}

impl FractalValue {
    pub fn display(&self) -> String {
        match self {
            Self::Int(n)   => n.to_string(),
            Self::Float(f) => format!("{:.4}", f),
            Self::Char(c)  => format!("'{}'", c),
            Self::Bool(b)  => b.to_string(),
            Self::Str(s)   => format!("\"{}\"", s),
            Self::Null     => "null".into(),
            Self::Void     => "void".into(),
            Self::Array(v) | Self::List(v) => {
                let inner: Vec<String> = v.iter().map(|x| x.display()).collect();
                format!("[{}]", inner.join(", "))
            }
            Self::Struct(f) => {
                let p: Vec<String> = f.iter().map(|(k,v)| format!("{}: {}", k, v.display())).collect();
                format!("{{{}}}", p.join(", "))
            }
        }
    }
    pub fn type_label(&self) -> &'static str {
        match self {
            Self::Int(_) => ":int", Self::Float(_) => ":float",
            Self::Char(_) => ":char", Self::Bool(_) => ":bool",
            Self::Str(_) => ":str", Self::Array(_) => ":array",
            Self::List(_) => ":list", Self::Struct(_) => ":struct",
            Self::Null => "null", Self::Void => "void",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Flat tree-node table  (consumed by tree_view.rs)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: usize, pub label: String,
    pub depth: usize, pub children: Vec<usize>, pub parent: Option<usize>,
}

pub fn build_tree_table(root: &ParseNode) -> Vec<TreeNode> {
    let mut t = Vec::new();
    visit_node(root, None, 0, &mut t);
    t
}

fn visit_node(node: &ParseNode, parent: Option<usize>, depth: usize, t: &mut Vec<TreeNode>) -> usize {
    let id = t.len();
    t.push(TreeNode { id, label: node_label(node), depth, children: vec![], parent });
    let kids: Vec<usize> = children_of(node).into_iter()
        .map(|c| visit_node(c, Some(id), depth + 1, t)).collect();
    t[id].children = kids;
    id
}

fn children_of(n: &ParseNode) -> Vec<&ParseNode> {
    match n {
        ParseNode::Program(items)    => items.iter().collect(),
        ParseNode::FuncDef { params, body, return_type, .. } => {
            let mut c: Vec<&ParseNode> = params.iter().collect();
            c.push(return_type); c.extend(body); c
        }
        ParseNode::StructDef { fields, .. }    => fields.iter().collect(),
        ParseNode::StructDecl { init, .. }     => init.as_deref().into_iter().collect(),
        ParseNode::Decl { init, .. }           => init.as_deref().into_iter().collect(),
        ParseNode::Assign { lvalue, expr, .. } => vec![lvalue.as_ref(), expr.as_ref()],
        ParseNode::If { condition, then_block, else_block } => {
            let mut c = vec![condition.as_ref()]; c.extend(then_block);
            if let Some(eb) = else_block { c.extend(eb); } c
        }
        ParseNode::For { start, stop, step, body, .. } => {
            let mut c = vec![start.as_ref(), stop.as_ref(), step.as_ref()]; c.extend(body); c
        }
        ParseNode::While { condition, body } => {
            let mut c = vec![condition.as_ref()]; c.extend(body); c
        }
        ParseNode::Return(e) | ParseNode::Exit(e) => vec![e.as_ref()],
        ParseNode::ExprStmt(e)        => vec![e.as_ref()],
        ParseNode::LogOr  { left, right } | ParseNode::LogAnd { left, right }
        | ParseNode::BitOr { left, right } | ParseNode::BitXor { left, right }
        | ParseNode::BitAnd { left, right }
        | ParseNode::BitShift { left, right, .. }
        | ParseNode::Add { left, right, .. } | ParseNode::Mul { left, right, .. }
        | ParseNode::Cmp { left, right, .. } => vec![left.as_ref(), right.as_ref()],
        ParseNode::LogNot { operand } | ParseNode::Unary { operand, .. } => vec![operand.as_ref()],
        ParseNode::Cast { expr, .. }  => vec![expr.as_ref()],
        ParseNode::ArrayLit(elems)    => elems.iter().collect(),
        ParseNode::AccessChain { steps, .. } => steps.iter().flat_map(|s| match s {
            AccessStep::Index(i) => vec![i.as_ref()],
            AccessStep::Call(a)  => a.iter().collect(),
            _                    => vec![],
        }).collect(),
        _ => vec![],
    }
}

fn node_label(n: &ParseNode) -> String {
    match n {
        ParseNode::Program(_)     => "Program".into(),
        ParseNode::FuncDef { name, return_type, .. }
            => format!("FuncDef {}  → {}", name, type_str(return_type)),
        ParseNode::Param { data_type, name }
            => format!("Param {} : {}", name, type_str(data_type)),
        ParseNode::StructDef { name, .. }  => format!("StructDef {}", name),
        ParseNode::StructDecl { var_name, struct_name, .. }
            => format!("StructDecl {} : {}", var_name, struct_name),
        ParseNode::Decl { data_type, name, init }
            => format!("Decl {} : {}{}", name, type_str(data_type), if init.is_some() { " =" } else { "" }),
        ParseNode::Assign { op, .. }  => format!("Assign {:?}", op),
        ParseNode::If { .. }          => "If".into(),
        ParseNode::For { var_name, .. } => format!("For {}", var_name),
        ParseNode::While { .. }       => "While".into(),
        ParseNode::Return(_)          => "Return".into(),
        ParseNode::Exit(_)            => "Exit".into(),
        ParseNode::Break              => "Break".into(),
        ParseNode::Continue           => "Continue".into(),
        ParseNode::ExprStmt(_)        => "ExprStmt".into(),
        ParseNode::AccessChain { base, steps } => {
            let chain: String = steps.iter().map(|s| match s {
                AccessStep::Field(f) => format!("::{}", f),
                AccessStep::Index(_) => "[…]".into(),
                AccessStep::Call(a)  => format!("({})", a.len()),
            }).collect();
            format!("Chain {}{}", base, chain)
        }
        ParseNode::LogOr  { .. }  => "LogOr".into(),
        ParseNode::LogAnd { .. }  => "LogAnd".into(),
        ParseNode::LogNot { .. }  => "LogNot".into(),
        ParseNode::Cmp { op, .. } => format!("Cmp {:?}", op),
        ParseNode::BitOr  { .. }  => "BitOr".into(),
        ParseNode::BitXor { .. }  => "BitXor".into(),
        ParseNode::BitAnd { .. }  => "BitAnd".into(),
        ParseNode::BitShift { op, .. } => format!("Shift {:?}", op),
        ParseNode::Add { op, .. } => format!("Add {:?}", op),
        ParseNode::Mul { op, .. } => format!("Mul {:?}", op),
        ParseNode::Unary { op, .. } => format!("Unary {:?}", op),
        ParseNode::Cast { target_type, .. } => format!("Cast → {}", type_str(target_type)),
        ParseNode::ArrayLit(e)    => format!("ArrayLit [{}]", e.len()),
        ParseNode::StructLit(_)   => "StructLit".into(),
        ParseNode::Identifier(s)  => format!("Ident {}", s),
        ParseNode::IntLit(n)      => format!("Int {}", n),
        ParseNode::FloatLit(f)    => format!("Float {}", f),
        ParseNode::CharLit(c)     => format!("Char '{}'", c),
        ParseNode::StringLit(s)   => format!("Str \"{}\"", &s[..s.len().min(20)]),
        ParseNode::BoolLit(b)     => format!("Bool {}", b),
        ParseNode::Null            => "Null".into(),
        ParseNode::TypeInt         => ":int".into(),
        ParseNode::TypeFloat       => ":float".into(),
        ParseNode::TypeChar        => ":char".into(),
        ParseNode::TypeBoolean     => ":bool".into(),
        ParseNode::TypeVoid        => ":void".into(),
        ParseNode::TypeArray { elem, size } => format!(":array<{},{}>", type_str(elem), size),
        ParseNode::TypeList { elem } => format!(":list<{}>", type_str(elem)),
        ParseNode::TypeStruct { name } => format!(":struct<{}>", name),
        _ => "Node".into(),
    }
}

fn type_str(n: &ParseNode) -> String {
    match n {
        ParseNode::TypeInt     => ":int".into(),
        ParseNode::TypeFloat   => ":float".into(),
        ParseNode::TypeChar    => ":char".into(),
        ParseNode::TypeBoolean => ":bool".into(),
        ParseNode::TypeVoid    => ":void".into(),
        ParseNode::TypeArray { elem, size } => format!(":array<{},{}>", type_str(elem), size),
        ParseNode::TypeList  { elem }   => format!(":list<{}>", type_str(elem)),
        ParseNode::TypeStruct { name }  => format!(":struct<{}>", name),
        _ => "?".into(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Step list
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StepEntry {
    pub node_id: usize, pub label: String, pub source_line: usize,
    pub action: StepAction,
}

#[derive(Debug, Clone)]
pub enum StepAction {
    Decl    { name: String, init: Option<ParseNode> },
    Assign  { lvalue: ParseNode, op: AssignOp, rhs: ParseNode },
    IfCheck { condition: ParseNode, after_index: usize },
    Jump    { target_index: usize },
    ForInit { var_name: String, start: ParseNode },
    ForCheck { var_name: String, stop: ParseNode, step_expr: ParseNode, body_end_index: usize },
    ExprStmt { expr: ParseNode },
    FuncReturn { expr: Option<ParseNode> },
    Exit,
}

// ─────────────────────────────────────────────────────────────────────────────
// Debug frame
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DebugFrame {
    pub active_node_id: usize,
    pub step_label:     String,
    pub source_line:    usize,
    pub scopes:         Vec<ScopeSnapshot>,
    pub call_stack:     Vec<String>,
    pub finished:       bool,
    pub error:          Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScopeSnapshot {
    pub label: String,
    pub vars:  Vec<VarRow>,
}

#[derive(Debug, Clone)]
pub struct VarRow {
    pub name: String, pub type_label: String, pub value: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Interpreter call frame
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct CallFrame {
    func_name: String,
    scopes: Vec<HashMap<String, FractalValue>>,
}

impl CallFrame {
    fn new(name: impl Into<String>) -> Self {
        Self { func_name: name.into(), scopes: vec![HashMap::new()] }
    }
    fn push_scope(&mut self) { self.scopes.push(HashMap::new()); }
    fn pop_scope(&mut self)  { if self.scopes.len() > 1 { self.scopes.pop(); } }
    fn set(&mut self, name: &str, val: FractalValue) {
        for s in self.scopes.iter_mut().rev() {
            if s.contains_key(name) { s.insert(name.into(), val); return; }
        }
        self.scopes.last_mut().unwrap().insert(name.into(), val);
    }
    fn declare(&mut self, name: &str, val: FractalValue) {
        self.scopes.last_mut().unwrap().insert(name.into(), val);
    }
    fn get(&self, name: &str) -> Option<&FractalValue> {
        for s in self.scopes.iter().rev() { if let Some(v) = s.get(name) { return Some(v); } }
        None
    }
    fn all_vars(&self) -> Vec<(String, FractalValue)> {
        let mut seen = std::collections::HashSet::new();
        let mut out  = Vec::new();
        for scope in self.scopes.iter().rev() {
            for (k, v) in scope {
                if seen.insert(k.clone()) { out.push((k.clone(), v.clone())); }
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// DebugSession
// ─────────────────────────────────────────────────────────────────────────────

pub struct DebugSession {
    steps:      Vec<StepEntry>,
    cursor:     usize,
    call_stack: Vec<CallFrame>,
    pub tree:   Vec<TreeNode>,
    pub finished: bool,
}

impl DebugSession {
    pub fn new(root: &ParseNode) -> Self {
        let tree = build_tree_table(root);
        let mut steps = Vec::new();
        if let ParseNode::Program(items) = root {
            for item in items {
                match item {
                    ParseNode::FuncDef { .. } | ParseNode::StructDef { .. } => {}
                    _ => flatten_node(item, &mut steps),
                }
            }
        }
        Self { steps, cursor: 0, call_stack: vec![CallFrame::new("global")], tree, finished: false }
    }

    pub fn total_steps(&self) -> usize { self.steps.len() }
    pub fn cursor(&self)      -> usize { self.cursor }

    pub fn current_frame(&self) -> DebugFrame {
        if self.cursor >= self.steps.len() || self.finished {
            return self.make_frame(0, "Program finished".into(), 0, None);
        }
        let e = &self.steps[self.cursor];
        self.make_frame(e.node_id, e.label.clone(), e.source_line, None)
    }

    pub fn step(&mut self) -> DebugFrame {
        if self.finished || self.cursor >= self.steps.len() {
            self.finished = true;
            return self.make_frame(0, "Program finished".into(), 0, None);
        }
        let entry = self.steps[self.cursor].clone();
        self.cursor += 1;
        let mut error: Option<String> = None;

        match entry.action {
            StepAction::Decl { name, init } => {
                let val = init.map(|e| self.eval_node(&e)).unwrap_or(FractalValue::Null);
                if let Some(f) = self.call_stack.last_mut() { f.declare(&name, val); }
            }
            StepAction::Assign { lvalue, op, rhs } => {
                let rval = self.eval_node(&rhs);
                error = self.do_assign(&lvalue, op, rval);
            }
            StepAction::IfCheck { condition, after_index } => {
                if !truthy(&self.eval_node(&condition)) { self.cursor = after_index; }
            }
            StepAction::Jump { target_index } => { self.cursor = target_index; }
            StepAction::ForInit { var_name, start } => {
                let val = self.eval_node(&start);
                if let Some(f) = self.call_stack.last_mut() { f.push_scope(); f.declare(&var_name, val); }
            }
            StepAction::ForCheck { var_name, stop, step_expr, body_end_index } => {
                let cur    = self.get_var(&var_name).cloned().unwrap_or(FractalValue::Int(0));
                let stop_v = self.eval_node(&stop);
                if int_ge(&cur, &stop_v) {
                    if let Some(f) = self.call_stack.last_mut() { f.pop_scope(); }
                    self.cursor = body_end_index;
                } else {
                    let step_v = self.eval_node(&step_expr);
                    let next   = int_add(&cur, &step_v);
                    if let Some(f) = self.call_stack.last_mut() { f.set(&var_name, next); }
                }
            }
            StepAction::ExprStmt { expr }       => { self.eval_node(&expr); }
            StepAction::FuncReturn { expr }      => {
                let _ = expr.map(|e| self.eval_node(&e));
                self.call_stack.pop();
                if self.call_stack.is_empty() { self.call_stack.push(CallFrame::new("global")); }
            }
            StepAction::Exit => { self.finished = true; }
        }

        if self.cursor >= self.steps.len() { self.finished = true; }
        self.make_frame(entry.node_id, entry.label, entry.source_line, error)
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_frame(&self, nid: usize, label: String, line: usize, error: Option<String>) -> DebugFrame {
        DebugFrame {
            active_node_id: nid, step_label: label, source_line: line,
            scopes: self.snapshot(), call_stack: self.call_stack.iter().map(|f| f.func_name.clone()).collect(),
            finished: self.finished || self.cursor >= self.steps.len(), error,
        }
    }

    fn snapshot(&self) -> Vec<ScopeSnapshot> {
        self.call_stack.iter().map(|frame| {
            let vars = frame.all_vars().into_iter().map(|(name, val)| VarRow {
                name, type_label: val.type_label().into(), value: val.display(),
            }).collect();
            ScopeSnapshot { label: frame.func_name.clone(), vars }
        }).collect()
    }

    fn get_var(&self, name: &str) -> Option<&FractalValue> {
        for f in self.call_stack.iter().rev() { if let Some(v) = f.get(name) { return Some(v); } }
        None
    }

    fn do_assign(&mut self, lvalue: &ParseNode, op: AssignOp, rval: FractalValue) -> Option<String> {
        if let ParseNode::AccessChain { base, steps } = lvalue {
            if steps.is_empty() {
                let cur  = self.get_var(base).cloned().unwrap_or(FractalValue::Null);
                let next = apply_op(cur, op, rval);
                if let Some(f) = self.call_stack.last_mut() { f.set(base, next); }
            }
        }
        None
    }

    pub fn eval_node(&mut self, node: &ParseNode) -> FractalValue {
        match node {
            ParseNode::IntLit(n)    => FractalValue::Int(*n),
            ParseNode::FloatLit(f)  => FractalValue::Float(*f),
            ParseNode::CharLit(c)   => FractalValue::Char(*c),
            ParseNode::BoolLit(b)   => FractalValue::Bool(*b),
            ParseNode::StringLit(s) => FractalValue::Str(s.clone()),
            ParseNode::Null         => FractalValue::Null,
            ParseNode::Identifier(name) => self.get_var(name).cloned().unwrap_or(FractalValue::Null),

            ParseNode::AccessChain { base, steps } => {
                let steps_owned = steps.clone();
                let mut val = self.get_var(base).cloned().unwrap_or(FractalValue::Null);
                for step in &steps_owned {
                    val = match step {
                        AccessStep::Field(f) => match &val {
                            FractalValue::Struct(map) => map.get(f).cloned().unwrap_or(FractalValue::Null),
                            _ => FractalValue::Null,
                        },
                        AccessStep::Index(idx_node) => {
                            let idx = match self.eval_node(idx_node) {
                                FractalValue::Int(i) => i as usize, _ => 0,
                            };
                            match &val {
                                FractalValue::Array(a) | FractalValue::List(a) =>
                                    a.get(idx).cloned().unwrap_or(FractalValue::Null),
                                _ => FractalValue::Null,
                            }
                        }
                        AccessStep::Call(args) => {
                            let arg_vals: Vec<FractalValue> = args.iter().map(|a| self.eval_node(a)).collect();
                            self.call_builtin(base, &arg_vals)
                        }
                    };
                }
                val
            }

            ParseNode::Cast { target_type, expr } => cast_value(self.eval_node(expr), target_type),

            ParseNode::ArrayLit(elems) => {
                FractalValue::Array(elems.iter().map(|e| self.eval_node(e)).collect())
            }

            ParseNode::LogOr { left, right } => {
                let l = self.eval_node(left);
                if truthy(&l) { return FractalValue::Bool(true); }
                FractalValue::Bool(truthy(&self.eval_node(right)))
            }
            ParseNode::LogAnd { left, right } => {
                let l = self.eval_node(left);
                if !truthy(&l) { return FractalValue::Bool(false); }
                FractalValue::Bool(truthy(&self.eval_node(right)))
            }
            ParseNode::LogNot { operand } =>
                FractalValue::Bool(!truthy(&self.eval_node(operand))),

            ParseNode::Cmp { left, right, op } => {
                let (l, r) = (self.eval_node(left), self.eval_node(right));
                FractalValue::Bool(cmp_vals(&l, &r, op))
            }

            ParseNode::Add { left, right, op } => {
                let (l, r) = (self.eval_node(left), self.eval_node(right));
                match (l, r) {
                    (FractalValue::Int(a),   FractalValue::Int(b))   =>
                        FractalValue::Int(if matches!(op, AddOp::Add) { a+b } else { a-b }),
                    (FractalValue::Float(a), FractalValue::Float(b)) =>
                        FractalValue::Float(if matches!(op, AddOp::Add) { a+b } else { a-b }),
                    _ => FractalValue::Null,
                }
            }

            ParseNode::Mul { left, right, op } => {
                let (l, r) = (self.eval_node(left), self.eval_node(right));
                match (l, r) {
                    (FractalValue::Int(a), FractalValue::Int(b)) => match op {
                        MulOp::Mul => FractalValue::Int(a * b),
                        MulOp::Div => FractalValue::Int(if b == 0 { 0 } else { a / b }),
                        MulOp::Mod => FractalValue::Int(if b == 0 { 0 } else { a % b }),
                    },
                    (FractalValue::Float(a), FractalValue::Float(b)) => match op {
                        MulOp::Mul => FractalValue::Float(a * b),
                        MulOp::Div => FractalValue::Float(a / b),
                        _ => FractalValue::Null,
                    },
                    _ => FractalValue::Null,
                }
            }

            ParseNode::Unary { op, operand } => {
                let v = self.eval_node(operand);
                match (v, op) {
                    (FractalValue::Int(n),   UnOp::Neg)    => FractalValue::Int(-n),
                    (FractalValue::Float(f), UnOp::Neg)    => FractalValue::Float(-f),
                    (FractalValue::Int(n),   UnOp::BitNot) => FractalValue::Int(!n),
                    _ => FractalValue::Null,
                }
            }

            ParseNode::BitAnd { left, right } => match (self.eval_node(left), self.eval_node(right)) {
                (FractalValue::Int(a), FractalValue::Int(b)) => FractalValue::Int(a & b), _ => FractalValue::Null },
            ParseNode::BitOr  { left, right } => match (self.eval_node(left), self.eval_node(right)) {
                (FractalValue::Int(a), FractalValue::Int(b)) => FractalValue::Int(a | b), _ => FractalValue::Null },
            ParseNode::BitXor { left, right } => match (self.eval_node(left), self.eval_node(right)) {
                (FractalValue::Int(a), FractalValue::Int(b)) => FractalValue::Int(a ^ b), _ => FractalValue::Null },
            ParseNode::BitShift { left, right, op } => match (self.eval_node(left), self.eval_node(right)) {
                (FractalValue::Int(a), FractalValue::Int(b)) => FractalValue::Int(
                    if matches!(op, ShiftOp::Left) { a << b } else { a >> b }), _ => FractalValue::Null },

            ParseNode::ExprStmt(e) => self.eval_node(e),
            _ => FractalValue::Null,
        }
    }

    fn call_builtin(&mut self, name: &str, args: &[FractalValue]) -> FractalValue {
        match name {
            "len" => match args.first() {
                Some(FractalValue::Array(v) | FractalValue::List(v)) => FractalValue::Int(v.len() as i64),
                _ => FractalValue::Int(0),
            },
            "abs" => match args.first() {
                Some(FractalValue::Int(n))   => FractalValue::Int(n.abs()),
                Some(FractalValue::Float(f)) => FractalValue::Float(f.abs()),
                _ => FractalValue::Null,
            },
            "sqrt" => match args.first() {
                Some(FractalValue::Float(f)) => FractalValue::Float(f.sqrt()),
                _ => FractalValue::Null,
            },
            _ => FractalValue::Void,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Flatten AST → step list
// ─────────────────────────────────────────────────────────────────────────────

fn flatten_node(node: &ParseNode, steps: &mut Vec<StepEntry>) {
    let node_id = steps.len();
    match node {
        ParseNode::Decl { name, init, .. } => push(steps, node_id,
            format!("Decl  {}", name),
            StepAction::Decl { name: name.clone(), init: init.as_deref().cloned() }),

        ParseNode::StructDecl { var_name, init, .. } => push(steps, node_id,
            format!("StructDecl  {}", var_name),
            StepAction::Decl { name: var_name.clone(), init: init.as_deref().cloned() }),

        ParseNode::Assign { lvalue, op, expr } => push(steps, node_id,
            format!("Assign  {:?}", op),
            StepAction::Assign { lvalue: *lvalue.clone(), op: op.clone(), rhs: *expr.clone() }),

        ParseNode::If { condition, then_block, else_block } => {
            let check_idx = steps.len();
            push(steps, node_id, "If  (condition)".into(),
                StepAction::IfCheck { condition: *condition.clone(), after_index: 0 });
            for s in then_block { flatten_node(s, steps); }
            if let Some(eb) = else_block {
                let jump_idx   = steps.len();
                push(steps, node_id, "Jump (skip else)".into(), StepAction::Jump { target_index: 0 });
                let else_start = steps.len();
                patch_if(steps, check_idx, else_start);
                for s in eb { flatten_node(s, steps); }
                patch_jump(steps, jump_idx, steps.len());
            } else {
                patch_if(steps, check_idx, steps.len());
            }
        }

        ParseNode::For { var_name, start, stop, step, body, .. } => {
            push(steps, node_id, format!("ForInit  {}", var_name),
                StepAction::ForInit { var_name: var_name.clone(), start: *start.clone() });
            let check_idx = steps.len();
            push(steps, node_id, format!("ForCheck  {}", var_name),
                StepAction::ForCheck { var_name: var_name.clone(), stop: *stop.clone(),
                    step_expr: *step.clone(), body_end_index: 0 });
            for s in body { flatten_node(s, steps); }
            let back = steps.len();
            push(steps, node_id, "ForBack".into(), StepAction::Jump { target_index: check_idx });
            patch_for(steps, check_idx, back + 1);
        }

        ParseNode::While { condition, body } => {
            let check_idx = steps.len();
            push(steps, node_id, "While  (condition)".into(),
                StepAction::IfCheck { condition: *condition.clone(), after_index: 0 });
            for s in body { flatten_node(s, steps); }
            let back = steps.len();
            push(steps, node_id, "WhileBack".into(), StepAction::Jump { target_index: check_idx });
            patch_if(steps, check_idx, back + 1);
        }

        ParseNode::Return(e) => push(steps, node_id, "Return".into(),
            StepAction::FuncReturn { expr: Some(*e.clone()) }),

        ParseNode::Exit(_) => push(steps, node_id, "Exit".into(), StepAction::Exit),

        ParseNode::ExprStmt(e) => push(steps, node_id, "ExprStmt".into(),
            StepAction::ExprStmt { expr: *e.clone() }),

        _ => {}
    }
}

fn push(steps: &mut Vec<StepEntry>, nid: usize, label: String, action: StepAction) {
    let line = steps.len() + 1;
    steps.push(StepEntry { node_id: nid, label, source_line: line, action });
}
fn patch_if(steps: &mut Vec<StepEntry>, idx: usize, target: usize) {
    if let Some(StepAction::IfCheck { after_index, .. }) = steps.get_mut(idx).map(|e| &mut e.action) {
        *after_index = target; } }
fn patch_jump(steps: &mut Vec<StepEntry>, idx: usize, target: usize) {
    if let Some(StepAction::Jump { target_index }) = steps.get_mut(idx).map(|e| &mut e.action) {
        *target_index = target; } }
fn patch_for(steps: &mut Vec<StepEntry>, idx: usize, target: usize) {
    if let Some(StepAction::ForCheck { body_end_index, .. }) = steps.get_mut(idx).map(|e| &mut e.action) {
        *body_end_index = target; } }

// ─────────────────────────────────────────────────────────────────────────────
// Pure helpers
// ─────────────────────────────────────────────────────────────────────────────

fn truthy(v: &FractalValue) -> bool {
    match v {
        FractalValue::Bool(b)  => *b,
        FractalValue::Int(n)   => *n != 0,
        FractalValue::Float(f) => *f != 0.0,
        FractalValue::Null     => false,
        _                      => true,
    }
}
fn int_ge(a: &FractalValue, b: &FractalValue) -> bool {
    matches!((a, b), (FractalValue::Int(x), FractalValue::Int(y)) if x >= y)
}
fn int_add(a: &FractalValue, b: &FractalValue) -> FractalValue {
    match (a, b) { (FractalValue::Int(x), FractalValue::Int(y)) => FractalValue::Int(x + y), _ => a.clone() }
}
fn cmp_vals(l: &FractalValue, r: &FractalValue, op: &CmpOp) -> bool {
    match (l, r) {
        (FractalValue::Int(a),   FractalValue::Int(b))   => cmp_ord(a, b, op),
        (FractalValue::Float(a), FractalValue::Float(b)) => cmp_ord_f(a, b, op),
        (FractalValue::Char(a),  FractalValue::Char(b))  => cmp_ord(a, b, op),
        (FractalValue::Bool(a),  FractalValue::Bool(b))  =>
            matches!(op, CmpOp::EqEq) && a == b || matches!(op, CmpOp::Ne) && a != b,
        _ => false,
    }
}
fn cmp_ord<T: PartialOrd>(a: &T, b: &T, op: &CmpOp) -> bool {
    match op { CmpOp::Gt => a>b, CmpOp::Lt => a<b, CmpOp::Ge => a>=b,
               CmpOp::Le => a<=b, CmpOp::EqEq => a==b, CmpOp::Ne => a!=b }
}
fn cmp_ord_f(a: &f64, b: &f64, op: &CmpOp) -> bool {
    match op { CmpOp::Gt => a>b, CmpOp::Lt => a<b, CmpOp::Ge => a>=b,
               CmpOp::Le => a<=b,
               CmpOp::EqEq => (a-b).abs() < f64::EPSILON,
               CmpOp::Ne   => (a-b).abs() >= f64::EPSILON }
}
fn apply_op(cur: FractalValue, op: AssignOp, rval: FractalValue) -> FractalValue {
    match op {
        AssignOp::Eq      => rval,
        AssignOp::PlusEq  => int_add(&cur, &rval),
        AssignOp::MinusEq => match (cur, rval) {
            (FractalValue::Int(a),   FractalValue::Int(b))   => FractalValue::Int(a - b),
            (FractalValue::Float(a), FractalValue::Float(b)) => FractalValue::Float(a - b),
            _ => FractalValue::Null },
        AssignOp::StarEq  => match (cur, rval) {
            (FractalValue::Int(a),   FractalValue::Int(b))   => FractalValue::Int(a * b),
            (FractalValue::Float(a), FractalValue::Float(b)) => FractalValue::Float(a * b),
            _ => FractalValue::Null },
        AssignOp::SlashEq => match (cur, rval) {
            (FractalValue::Int(a),   FractalValue::Int(b))   => FractalValue::Int(if b==0 {0} else {a/b}),
            (FractalValue::Float(a), FractalValue::Float(b)) => FractalValue::Float(a / b),
            _ => FractalValue::Null },
        AssignOp::PercentEq => match (cur, rval) {
            (FractalValue::Int(a), FractalValue::Int(b)) => FractalValue::Int(if b==0 {0} else {a%b}),
            _ => FractalValue::Null },
        AssignOp::AmpEq   => match (cur, rval) { (FractalValue::Int(a), FractalValue::Int(b)) => FractalValue::Int(a&b), _ => FractalValue::Null },
        AssignOp::PipeEq  => match (cur, rval) { (FractalValue::Int(a), FractalValue::Int(b)) => FractalValue::Int(a|b), _ => FractalValue::Null },
        AssignOp::CaretEq => match (cur, rval) { (FractalValue::Int(a), FractalValue::Int(b)) => FractalValue::Int(a^b), _ => FractalValue::Null },
    }
}
fn cast_value(v: FractalValue, target: &ParseNode) -> FractalValue {
    match (v, target) {
        (FractalValue::Float(f), ParseNode::TypeInt)     => FractalValue::Int(f as i64),
        (FractalValue::Int(n),   ParseNode::TypeFloat)   => FractalValue::Float(n as f64),
        (FractalValue::Int(n),   ParseNode::TypeChar)    => FractalValue::Char(char::from_u32(n as u32).unwrap_or('\0')),
        (FractalValue::Char(c),  ParseNode::TypeInt)     => FractalValue::Int(c as i64),
        (FractalValue::Int(n),   ParseNode::TypeBoolean) => FractalValue::Bool(n != 0),
        (FractalValue::Bool(b),  ParseNode::TypeInt)     => FractalValue::Int(b as i64),
        (v, _) => v,
    }
}