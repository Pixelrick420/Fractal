#![allow(unused)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

use crate::compiler::parser::{AccessStep, AssignOp, CmpOp, ParseNode, AddOp, MulOp, UnOp};

// ─────────────────────────────────────────────
//  Types
// ─────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FrType {
    Int,
    Float,
    Char,
    Boolean,
    Void,
    Array { elem: Box<FrType>, size: i64 },
    List { elem: Box<FrType> },
    Struct { name: String },
    Null,
}

impl fmt::Display for FrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrType::Int => write!(f, "int"),
            FrType::Float => write!(f, "float"),
            FrType::Char => write!(f, "char"),
            FrType::Boolean => write!(f, "boolean"),
            FrType::Void => write!(f, "void"),
            FrType::Null => write!(f, "null"),
            FrType::Array { elem, size } => write!(f, "array<{},{}>", elem, size),
            FrType::List { elem } => write!(f, "list<{}>", elem),
            FrType::Struct { name } => write!(f, "struct<{}>", name),
        }
    }
}

// ─────────────────────────────────────────────
//  Symbol value (what we know at analysis time)
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum SymbolValue {
    Int(i64),
    Float(f64),
    Char(char),
    Boolean(bool),
    Null,
    Array(Vec<SymbolValue>),
    List(Vec<SymbolValue>),
    StructInstance { fields: HashMap<String, SymbolValue> },
    Unknown, // value not statically known
}

impl fmt::Display for SymbolValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolValue::Int(n) => write!(f, "{}", n),
            SymbolValue::Float(n) => write!(f, "{}", n),
            SymbolValue::Char(c) => write!(f, "'{}'", c),
            SymbolValue::Boolean(b) => write!(f, "{}", b),
            SymbolValue::Null => write!(f, "null"),
            SymbolValue::Unknown => write!(f, "<runtime>"),
            SymbolValue::Array(elems) => {
                let s: Vec<String> = elems.iter().map(|e| format!("{}", e)).collect();
                write!(f, "[{}]", s.join(", "))
            }
            SymbolValue::List(elems) => {
                let s: Vec<String> = elems.iter().map(|e| format!("{}", e)).collect();
                write!(f, "list[{}]", s.join(", "))
            }
            SymbolValue::StructInstance { fields } => {
                let mut pairs: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                pairs.sort();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}

// ─────────────────────────────────────────────
//  Symbol entry
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: FrType,
    pub value: SymbolValue,
    pub scope_depth: usize,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<20} : {:<25} = {}  [scope={}]",
            self.name, format!("{}", self.ty), format!("{}", self.value), self.scope_depth
        )
    }
}

// ─────────────────────────────────────────────
//  Struct definition registry
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, FrType)>,
}

// ─────────────────────────────────────────────
//  Function definition registry
// ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<(String, FrType)>,
    pub return_type: FrType,
}

// ─────────────────────────────────────────────
//  Scope stack
// ─────────────────────────────────────────────

struct ScopeStack {
    /// Each frame: (depth, map of name -> Symbol)
    frames: Vec<HashMap<String, Symbol>>,
    depth: usize,
}

impl ScopeStack {
    fn new() -> Self {
        ScopeStack {
            frames: vec![HashMap::new()],
            depth: 0,
        }
    }

    fn push(&mut self) {
        self.depth += 1;
        self.frames.push(HashMap::new());
    }

    fn pop(&mut self) -> HashMap<String, Symbol> {
        self.depth = self.depth.saturating_sub(1);
        self.frames.pop().unwrap_or_default()
    }

    fn declare(&mut self, sym: Symbol) -> Result<(), SemanticError> {
        let frame = self.frames.last_mut().unwrap();
        if frame.contains_key(&sym.name) {
            return Err(SemanticError::new(format!(
                "Variable '{}' already declared in this scope",
                sym.name
            )));
        }
        frame.insert(sym.name.clone(), sym);
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for frame in self.frames.iter().rev() {
            if let Some(sym) = frame.get(name) {
                return Some(sym);
            }
        }
        None
    }

    fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) {
                return frame.get_mut(name);
            }
        }
        None
    }

    fn current_depth(&self) -> usize {
        self.depth
    }

    /// Collect all symbols from all frames (for display)
    fn all_symbols(&self) -> Vec<Symbol> {
        let mut out = Vec::new();
        for frame in &self.frames {
            let mut syms: Vec<_> = frame.values().cloned().collect();
            syms.sort_by(|a, b| a.name.cmp(&b.name));
            out.extend(syms);
        }
        out
    }
}

// ─────────────────────────────────────────────
//  Semantic error
// ─────────────────────────────────────────────

#[derive(Debug)]
pub struct SemanticError {
    pub message: String,
}

impl SemanticError {
    fn new(msg: impl Into<String>) -> Self {
        SemanticError { message: msg.into() }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1;31mSemantic Error:\x1b[0m {}", self.message)
    }
}

// ─────────────────────────────────────────────
//  Semantic result
// ─────────────────────────────────────────────

pub struct SemanticResult {
    pub symbol_table: Vec<Symbol>,
    pub errors: Vec<SemanticError>,
}

impl SemanticResult {
    pub fn print_symbol_table(&self) {
        println!("\n\x1b[1;34m╔══════════════════════════════════════════════════════════════════════╗\x1b[0m");
        println!("\x1b[1;34m║                         SYMBOL TABLE                                ║\x1b[0m");
        println!("\x1b[1;34m╚══════════════════════════════════════════════════════════════════════╝\x1b[0m");
        println!(
            "\x1b[1m{:<20}   {:<25}   {:<25}   {}\x1b[0m",
            "NAME", "TYPE", "VALUE", "SCOPE"
        );
        println!("{}", "─".repeat(85));
        for sym in &self.symbol_table {
            println!("{}", sym);
        }
        println!("{}", "─".repeat(85));
    }

    pub fn print_errors(&self) {
        if self.errors.is_empty() {
            println!("\x1b[1;32m✓ No semantic errors.\x1b[0m");
        } else {
            for e in &self.errors {
                eprintln!("{}", e);
            }
        }
    }
}

// ─────────────────────────────────────────────
//  Analyzer
// ─────────────────────────────────────────────

pub struct Analyzer {
    scopes: ScopeStack,
    structs: HashMap<String, StructDef>,
    functions: HashMap<String, FuncDef>,
    errors: Vec<SemanticError>,
    /// Snapshot of every symbol ever declared (for final table)
    all_symbols: Vec<Symbol>,
    /// Whether we are inside a loop (for break/continue)
    loop_depth: usize,
    /// Current function return type
    current_fn_return: Option<FrType>,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            scopes: ScopeStack::new(),
            structs: HashMap::new(),
            functions: HashMap::new(),
            errors: Vec::new(),
            all_symbols: Vec::new(),
            loop_depth: 0,
            current_fn_return: None,
        }
    }

    fn err(&mut self, msg: impl Into<String>) {
        self.errors.push(SemanticError::new(msg));
    }

    // ── helpers ──────────────────────────────

    fn node_to_frtype(&self, node: &ParseNode) -> FrType {
        match node {
            ParseNode::TypeInt => FrType::Int,
            ParseNode::TypeFloat => FrType::Float,
            ParseNode::TypeChar => FrType::Char,
            ParseNode::TypeBoolean => FrType::Boolean,
            ParseNode::TypeVoid => FrType::Void,
            ParseNode::TypeArray { elem, size } => FrType::Array {
                elem: Box::new(self.node_to_frtype(elem)),
                size: *size,
            },
            ParseNode::TypeList { elem } => FrType::List {
                elem: Box::new(self.node_to_frtype(elem)),
            },
            ParseNode::TypeStruct { name } => FrType::Struct { name: name.clone() },
            _ => FrType::Void,
        }
    }

    fn declare_sym(&mut self, sym: Symbol) {
        // snapshot before declaring (so we capture even if duplicate error)
        let snap = sym.clone();
        match self.scopes.declare(sym) {
            Ok(()) => self.all_symbols.push(snap),
            Err(e) => self.err(e.message),
        }
    }

    fn update_sym_value(&mut self, name: &str, value: SymbolValue) {
        if let Some(sym) = self.scopes.lookup_mut(name) {
            sym.value = value.clone();
            // update snapshot
            for s in self.all_symbols.iter_mut().rev() {
                if s.name == name {
                    s.value = value;
                    break;
                }
            }
        }
    }

    // ── type inference for expressions ───────

    /// Returns (type, optional_constant_value)
    fn infer_expr(&mut self, node: &ParseNode) -> (FrType, SymbolValue) {
        match node {
            ParseNode::IntLit(n) => (FrType::Int, SymbolValue::Int(*n)),
            ParseNode::FloatLit(f) => (FrType::Float, SymbolValue::Float(*f)),
            ParseNode::CharLit(c) => (FrType::Char, SymbolValue::Char(*c)),
            ParseNode::BoolLit(b) => (FrType::Boolean, SymbolValue::Boolean(*b)),
            ParseNode::Null => (FrType::Null, SymbolValue::Null),

            ParseNode::StringLit(s) => {
                let chars: Vec<SymbolValue> =
                    s.chars().map(SymbolValue::Char).collect();
                let len = chars.len() as i64;
                (
                    FrType::Array { elem: Box::new(FrType::Char), size: len },
                    SymbolValue::Array(chars),
                )
            }

            ParseNode::ArrayLit(elems) => {
                if elems.is_empty() {
                    return (FrType::Array { elem: Box::new(FrType::Void), size: 0 }, SymbolValue::Array(vec![]));
                }
                let (first_ty, _) = self.infer_expr(&elems[0]);
                let mut vals = Vec::new();
                for e in elems {
                    let (ty, v) = self.infer_expr(e);
                    if ty != first_ty {
                        self.err(format!(
                            "Array literal has mixed types: expected {}, got {}",
                            first_ty, ty
                        ));
                    }
                    vals.push(v);
                }
                let size = vals.len() as i64;
                (FrType::Array { elem: Box::new(first_ty), size }, SymbolValue::Array(vals))
            }

            ParseNode::StructLit(fields) => {
                let mut field_vals = HashMap::new();
                for (name, val_node) in fields {
                    let (_, v) = self.infer_expr(val_node);
                    field_vals.insert(name.clone(), v);
                }
                // We can't know the struct type without context here; caller resolves
                (FrType::Void, SymbolValue::StructInstance { fields: field_vals })
            }

            ParseNode::Identifier(name) => {
                if let Some(sym) = self.scopes.lookup(name) {
                    (sym.ty.clone(), sym.value.clone())
                } else {
                    self.err(format!("Undeclared identifier '{}'", name));
                    (FrType::Void, SymbolValue::Unknown)
                }
            }

            ParseNode::AccessChain { base, steps } => {
                self.infer_access_chain(base, steps)
            }

            ParseNode::Cast { target_type, expr } => {
                let dest = self.node_to_frtype(target_type);
                let (src_ty, src_val) = self.infer_expr(expr);
                let cast_val = self.apply_cast(&dest, &src_ty, &src_val);
                (dest, cast_val)
            }

            ParseNode::Unary { op, operand } => {
                let (ty, val) = self.infer_expr(operand);
                match op {
                    UnOp::Neg => {
                        if ty != FrType::Int && ty != FrType::Float {
                            self.err(format!("Unary '-' requires int or float, got {}", ty));
                        }
                        let v = match val {
                            SymbolValue::Int(n) => SymbolValue::Int(-n),
                            SymbolValue::Float(f) => SymbolValue::Float(-f),
                            _ => SymbolValue::Unknown,
                        };
                        (ty, v)
                    }
                    UnOp::BitNot => {
                        if ty != FrType::Int {
                            self.err(format!("Bitwise NOT requires int, got {}", ty));
                        }
                        let v = match val {
                            SymbolValue::Int(n) => SymbolValue::Int(!n),
                            _ => SymbolValue::Unknown,
                        };
                        (FrType::Int, v)
                    }
                }
            }

            ParseNode::Add { left, op, right } => {
                let (lt, lv) = self.infer_expr(left);
                let (rt, rv) = self.infer_expr(right);
                if lt != rt {
                    self.err(format!(
                        "Type mismatch in '{}' operation: {} vs {}",
                        match op { AddOp::Add => "+", AddOp::Sub => "-" },
                        lt, rt
                    ));
                    return (lt, SymbolValue::Unknown);
                }
                let v = match (op, &lv, &rv) {
                    (AddOp::Add, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a + b),
                    (AddOp::Add, SymbolValue::Float(a), SymbolValue::Float(b)) => SymbolValue::Float(a + b),
                    (AddOp::Sub, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a - b),
                    (AddOp::Sub, SymbolValue::Float(a), SymbolValue::Float(b)) => SymbolValue::Float(a - b),
                    _ => SymbolValue::Unknown,
                };
                (lt, v)
            }

            ParseNode::Mul { left, op, right } => {
                let (lt, lv) = self.infer_expr(left);
                let (rt, rv) = self.infer_expr(right);
                if lt != rt {
                    self.err(format!(
                        "Type mismatch in '{}' operation: {} vs {}",
                        match op { MulOp::Mul => "*", MulOp::Div => "/", MulOp::Mod => "%" },
                        lt, rt
                    ));
                    return (lt, SymbolValue::Unknown);
                }
                let v = match (op, &lv, &rv) {
                    (MulOp::Mul, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a * b),
                    (MulOp::Mul, SymbolValue::Float(a), SymbolValue::Float(b)) => SymbolValue::Float(a * b),
                    (MulOp::Div, SymbolValue::Int(a), SymbolValue::Int(b)) if *b != 0 => SymbolValue::Int(a / b),
                    (MulOp::Div, SymbolValue::Float(a), SymbolValue::Float(b)) => SymbolValue::Float(a / b),
                    (MulOp::Mod, SymbolValue::Int(a), SymbolValue::Int(b)) if *b != 0 => SymbolValue::Int(a % b),
                    _ => SymbolValue::Unknown,
                };
                (lt, v)
            }

            ParseNode::BitAnd { left, right }
            | ParseNode::BitOr { left, right }
            | ParseNode::BitXor { left, right } => {
                let (lt, lv) = self.infer_expr(left);
                let (rt, rv) = self.infer_expr(right);
                if lt != FrType::Int || rt != FrType::Int {
                    self.err(format!(
                        "Bitwise operation requires int operands, got {} and {}",
                        lt, rt
                    ));
                }
                let v = match (node, &lv, &rv) {
                    (ParseNode::BitAnd { .. }, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a & b),
                    (ParseNode::BitOr { .. }, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a | b),
                    (ParseNode::BitXor { .. }, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a ^ b),
                    _ => SymbolValue::Unknown,
                };
                (FrType::Int, v)
            }

            ParseNode::Cmp { left, op, right } => {
                let (lt, lv) = self.infer_expr(left);
                let (rt, rv) = self.infer_expr(right);
                // null comparisons allowed with struct
                let is_null_cmp = lt == FrType::Null || rt == FrType::Null;
                if !is_null_cmp && lt != rt {
                    self.err(format!(
                        "Type mismatch in comparison: {} vs {}",
                        lt, rt
                    ));
                }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            ParseNode::LogAnd { left, right } | ParseNode::LogOr { left, right } => {
                let (lt, _) = self.infer_expr(left);
                let (rt, _) = self.infer_expr(right);
                if lt != FrType::Boolean {
                    self.err(format!("Logical operator requires boolean, got {}", lt));
                }
                if rt != FrType::Boolean {
                    self.err(format!("Logical operator requires boolean, got {}", rt));
                }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            ParseNode::LogNot { operand } => {
                let (ty, _) = self.infer_expr(operand);
                if ty != FrType::Boolean {
                    self.err(format!("'!not' requires boolean, got {}", ty));
                }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            _ => (FrType::Void, SymbolValue::Unknown),
        }
    }

    fn infer_access_chain(&mut self, base: &str, steps: &[AccessStep]) -> (FrType, SymbolValue) {
        // Lookup base in scope
        let base_sym = match self.scopes.lookup(base) {
            Some(s) => (s.ty.clone(), s.value.clone()),
            None => {
                // Could be a module name – try first step as field
                if let Some(AccessStep::Field(field)) = steps.first() {
                    let qualified = format!("{}::{}", base, field);
                    if let Some(sym) = self.scopes.lookup(&qualified) {
                        return (sym.ty.clone(), sym.value.clone());
                    }
                }
                self.err(format!("Undeclared identifier '{}'", base));
                return (FrType::Void, SymbolValue::Unknown);
            }
        };

        let mut cur_ty = base_sym.0;
        let mut cur_val = base_sym.1;

        for (i, step) in steps.iter().enumerate() {
            match step {
                AccessStep::Field(field) => {
                    // If it's a module-qualified name, look up as "module::field"
                    // Otherwise it's a struct field
                    match &cur_ty {
                        FrType::Struct { name: sname } => {
                            if let Some(sdef) = self.structs.get(sname).cloned() {
                                if let Some((_, fty)) = sdef.fields.iter().find(|(n, _)| n == field) {
                                    cur_ty = fty.clone();
                                    cur_val = match &cur_val {
                                        SymbolValue::StructInstance { fields } => {
                                            fields.get(field).cloned().unwrap_or(SymbolValue::Unknown)
                                        }
                                        _ => SymbolValue::Unknown,
                                    };
                                } else {
                                    self.err(format!(
                                        "Struct '{}' has no field '{}'",
                                        sname, field
                                    ));
                                    cur_ty = FrType::Void;
                                    cur_val = SymbolValue::Unknown;
                                }
                            } else {
                                // struct def not found (may be from import)
                                cur_ty = FrType::Void;
                                cur_val = SymbolValue::Unknown;
                            }
                        }
                        _ => {
                            // Could be module::something — look in scope
                            let qualified = format!("{}::{}", base, field);
                            if let Some(sym) = self.scopes.lookup(&qualified) {
                                cur_ty = sym.ty.clone();
                                cur_val = sym.value.clone();
                            } else {
                                self.err(format!(
                                    "'::{}' access on non-struct type {} (or unknown module)",
                                    field, cur_ty
                                ));
                                cur_ty = FrType::Void;
                                cur_val = SymbolValue::Unknown;
                            }
                        }
                    }
                }
                AccessStep::Index(idx_node) => {
                    let (idx_ty, _) = self.infer_expr(idx_node);
                    if idx_ty != FrType::Int {
                        self.err(format!("Array/list index must be int, got {}", idx_ty));
                    }
                    match cur_ty.clone() {
                        FrType::Array { elem, .. } | FrType::List { elem } => {
                            cur_ty = *elem;
                            cur_val = SymbolValue::Unknown;
                        }
                        _ => {
                            self.err(format!(
                                "Cannot index into type {}",
                                cur_ty
                            ));
                            cur_ty = FrType::Void;
                            cur_val = SymbolValue::Unknown;
                        }
                    }
                }
                AccessStep::Call(args) => {
                    // function call: base is the function name (with possible module prefix)
                    let fn_name = if i == 0 {
                        base.to_string()
                    } else {
                        // module::func pattern — already resolved above
                        base.to_string()
                    };
                    let ret = self.check_call(&fn_name, args);
                    cur_ty = ret;
                    cur_val = SymbolValue::Unknown;
                }
            }
        }

        (cur_ty, cur_val)
    }

    fn check_call(&mut self, name: &str, args: &[ParseNode]) -> FrType {
        // built-in functions
        match name {
            "print" => {
                // first arg is string, rest are any
                for a in args {
                    self.infer_expr(a);
                }
                return FrType::Void;
            }
            "input" => {
                for a in args {
                    self.infer_expr(a);
                }
                return FrType::Void;
            }
            "append" => {
                if args.len() == 2 {
                    let (list_ty, _) = self.infer_expr(&args[0]);
                    let (elem_ty, _) = self.infer_expr(&args[1]);
                    if let FrType::List { elem } = &list_ty {
                        if **elem != elem_ty {
                            self.err(format!(
                                "append: list element type {} does not match {}",
                                elem, elem_ty
                            ));
                        }
                    } else {
                        self.err(format!("append: first argument must be a list, got {}", list_ty));
                    }
                } else {
                    self.err("append expects 2 arguments");
                }
                return FrType::Void;
            }
            "pop" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    match ty {
                        FrType::Array { elem, .. } | FrType::List { elem } => return *elem,
                        _ => self.err(format!("pop: argument must be array or list, got {}", ty)),
                    }
                } else {
                    self.err("pop expects 1 argument");
                }
                return FrType::Void;
            }
            "sqrt" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    if ty != FrType::Float {
                        self.err(format!("sqrt: argument must be float, got {}", ty));
                    }
                } else {
                    self.err("sqrt expects 1 argument");
                }
                return FrType::Float;
            }
            "find" => { for a in args { self.infer_expr(a); } return FrType::Int; }
            "starts" | "ends" => { for a in args { self.infer_expr(a); } return FrType::Boolean; }
            "insert" | "delete" => { for a in args { self.infer_expr(a); } return FrType::Void; }
            _ => {}
        }

        // user-defined function
        if let Some(fdef) = self.functions.get(name).cloned() {
            let params = fdef.params.clone();
            let ret = fdef.return_type.clone();
            // check arg count
            if args.len() != params.len() {
                self.err(format!(
                    "Function '{}' expects {} arguments, got {}",
                    name, params.len(), args.len()
                ));
                return ret;
            }
            for (i, (a, (pname, pty))) in args.iter().zip(params.iter()).enumerate() {
                let (aty, _) = self.infer_expr(a);
                if aty != *pty && aty != FrType::Null {
                    self.err(format!(
                        "Argument {} of '{}': expected {}, got {}",
                        i + 1, name, pty, aty
                    ));
                }
            }
            return ret;
        }

        // unknown – may be from stdlib or not yet seen
        self.err(format!("Undeclared function '{}'", name));
        FrType::Void
    }

    fn apply_cast(&mut self, dest: &FrType, src: &FrType, val: &SymbolValue) -> SymbolValue {
        // validate cast is legal
        let legal = matches!(
            (src, dest),
            (FrType::Int, FrType::Float)
                | (FrType::Float, FrType::Int)
                | (FrType::Int, FrType::Char)
                | (FrType::Char, FrType::Int)
                | (FrType::Float, FrType::Boolean)
                | (FrType::Int, FrType::Boolean)
                | (FrType::Boolean, FrType::Int)
                | (FrType::Null, _)
                | (_, FrType::Null)
        );
        if !legal && src != dest {
            self.err(format!("Cannot cast {} to {}", src, dest));
            return SymbolValue::Unknown;
        }
        match (dest, val) {
            (FrType::Int, SymbolValue::Float(f)) => SymbolValue::Int(*f as i64),
            (FrType::Int, SymbolValue::Int(n)) => SymbolValue::Int(*n),
            (FrType::Int, SymbolValue::Char(c)) => SymbolValue::Int(*c as i64),
            (FrType::Int, SymbolValue::Boolean(b)) => SymbolValue::Int(*b as i64),
            (FrType::Float, SymbolValue::Int(n)) => SymbolValue::Float(*n as f64),
            (FrType::Float, SymbolValue::Float(f)) => SymbolValue::Float(*f),
            (FrType::Char, SymbolValue::Int(n)) => {
                SymbolValue::Char(char::from_u32(*n as u32).unwrap_or('\0'))
            }
            (FrType::Boolean, SymbolValue::Int(n)) => SymbolValue::Boolean(*n != 0),
            _ => SymbolValue::Unknown,
        }
    }

    // ── statement analysis ────────────────────

    fn check_stmt(&mut self, node: &ParseNode) {
        match node {
            ParseNode::Decl { data_type, name, init } => {
                let declared_ty = self.node_to_frtype(data_type);
                let (init_ty, init_val) = if let Some(init_expr) = init {
                    self.infer_expr(init_expr)
                } else {
                    (declared_ty.clone(), SymbolValue::Unknown)
                };

                // type check
                let value = if let Some(init_expr) = init {
                    // struct literal special case
                    let (ity, ival) = self.infer_expr(init_expr);
                    let resolved_val = match (&declared_ty, &ival) {
                        (FrType::Struct { .. }, SymbolValue::StructInstance { .. }) => ival,
                        _ => {
                            if ity != declared_ty && !matches!(ity, FrType::Null) {
                                self.err(format!(
                                    "Cannot assign {} to variable '{}' of type {}",
                                    ity, name, declared_ty
                                ));
                            }
                            ival
                        }
                    };
                    resolved_val
                } else {
                    SymbolValue::Unknown
                };

                let sym = Symbol {
                    name: name.clone(),
                    ty: declared_ty,
                    value,
                    scope_depth: self.scopes.current_depth(),
                };
                self.declare_sym(sym);
            }

            ParseNode::StructDecl { struct_name, var_name, init } => {
                let declared_ty = FrType::Struct { name: struct_name.clone() };
                if !self.structs.contains_key(struct_name) {
                    self.err(format!("Struct '{}' is not defined", struct_name));
                }
                let value = if let Some(init_expr) = init {
                    let (_, v) = self.infer_expr(init_expr);
                    v
                } else {
                    SymbolValue::Unknown
                };
                let sym = Symbol {
                    name: var_name.clone(),
                    ty: declared_ty,
                    value,
                    scope_depth: self.scopes.current_depth(),
                };
                self.declare_sym(sym);
            }

            ParseNode::StructDef { name, fields } => {
                let mut field_list = Vec::new();
                for f in fields {
                    match f {
                        ParseNode::Field { data_type, name: fname } => {
                            field_list.push((fname.clone(), self.node_to_frtype(data_type)));
                        }
                        _ => {}
                    }
                }
                self.structs.insert(
                    name.clone(),
                    StructDef { name: name.clone(), fields: field_list },
                );
            }

            ParseNode::Assign { lvalue, op, expr } => {
                let (lty, _) = self.infer_expr(lvalue);
                let (rty, rval) = self.infer_expr(expr);

                // check types match for the op
                if lty != rty && !matches!(rty, FrType::Null) && !matches!(lty, FrType::Null) {
                    self.err(format!(
                        "Assignment type mismatch: {} := {}",
                        lty, rty
                    ));
                }

                // bitwise assign only on int
                if matches!(op, AssignOp::AmpEq | AssignOp::PipeEq | AssignOp::CaretEq)
                    && lty != FrType::Int
                {
                    self.err(format!("Bitwise assignment requires int, got {}", lty));
                }

                // update value if lvalue is a simple identifier
                if let ParseNode::AccessChain { base, steps } = lvalue.as_ref() {
                    if steps.is_empty() {
                        self.update_sym_value(base, rval);
                    }
                }
            }

            ParseNode::ExprStmt(e) => {
                self.infer_expr(e);
            }

            ParseNode::Return(e) => {
                let (ty, _) = self.infer_expr(e);
                if let Some(expected) = &self.current_fn_return.clone() {
                    if ty != *expected && !matches!(ty, FrType::Null) {
                        self.err(format!(
                            "Return type mismatch: expected {}, got {}",
                            expected, ty
                        ));
                    }
                }
            }

            ParseNode::Exit(e) => {
                let (ty, _) = self.infer_expr(e);
                if ty != FrType::Int && !matches!(ty, FrType::Null) {
                    self.err(format!("exit() requires int, got {}", ty));
                }
            }

            ParseNode::Break | ParseNode::Continue => {
                if self.loop_depth == 0 {
                    self.err("break/continue outside loop");
                }
            }

            ParseNode::If { condition, then_block, else_block } => {
                let (cty, _) = self.infer_expr(condition);
                if cty != FrType::Boolean {
                    self.err(format!("If condition must be boolean, got {}", cty));
                }
                self.scopes.push();
                for s in then_block { self.check_stmt(s); }
                let popped = self.scopes.pop();
                self.absorb_scope(popped);

                if let Some(eb) = else_block {
                    self.scopes.push();
                    for s in eb { self.check_stmt(s); }
                    let popped = self.scopes.pop();
                    self.absorb_scope(popped);
                }
            }

            ParseNode::While { condition, body } => {
                let (cty, _) = self.infer_expr(condition);
                if cty != FrType::Boolean {
                    self.err(format!("While condition must be boolean, got {}", cty));
                }
                self.loop_depth += 1;
                self.scopes.push();
                for s in body { self.check_stmt(s); }
                let popped = self.scopes.pop();
                self.absorb_scope(popped);
                self.loop_depth -= 1;
            }

            ParseNode::For { var_type, var_name, start, stop, step, body } => {
                let declared_ty = self.node_to_frtype(var_type);
                let (sty, _) = self.infer_expr(start);
                let (ety, _) = self.infer_expr(stop);
                let (stepy, _) = self.infer_expr(step);

                if sty != declared_ty {
                    self.err(format!("For loop start type {} doesn't match {} {}", sty, declared_ty, var_name));
                }
                if ety != declared_ty {
                    self.err(format!("For loop stop type {} doesn't match {} {}", ety, declared_ty, var_name));
                }

                self.loop_depth += 1;
                self.scopes.push();
                let loop_sym = Symbol {
                    name: var_name.clone(),
                    ty: declared_ty,
                    value: SymbolValue::Unknown,
                    scope_depth: self.scopes.current_depth(),
                };
                self.declare_sym(loop_sym);
                for s in body { self.check_stmt(s); }
                let popped = self.scopes.pop();
                self.absorb_scope(popped);
                self.loop_depth -= 1;
            }

            ParseNode::FuncDef { name, params, return_type, body } => {
                let ret_ty = self.node_to_frtype(return_type);
                let mut param_list = Vec::new();
                for p in params {
                    if let ParseNode::Param { data_type, name: pname } = p {
                        param_list.push((pname.clone(), self.node_to_frtype(data_type)));
                    }
                }
                self.functions.insert(
                    name.clone(),
                    FuncDef {
                        name: name.clone(),
                        params: param_list.clone(),
                        return_type: ret_ty.clone(),
                    },
                );

                let saved_return = self.current_fn_return.take();
                self.current_fn_return = Some(ret_ty.clone());
                self.scopes.push();
                for (pname, pty) in &param_list {
                    let sym = Symbol {
                        name: pname.clone(),
                        ty: pty.clone(),
                        value: SymbolValue::Unknown,
                        scope_depth: self.scopes.current_depth(),
                    };
                    self.declare_sym(sym);
                }
                for s in body { self.check_stmt(s); }
                let popped = self.scopes.pop();
                self.absorb_scope(popped);
                self.current_fn_return = saved_return;
            }

            ParseNode::Module { name, items } => {
                // process module items, injecting them with module prefix
                self.scopes.push();
                for item in items {
                    self.check_stmt(item);
                }
                let popped = self.scopes.pop();
                // Re-declare symbols with module:: prefix in current scope
                for (_, sym) in popped {
                    let qualified = Symbol {
                        name: format!("{}::{}", name, sym.name),
                        ty: sym.ty,
                        value: sym.value,
                        scope_depth: self.scopes.current_depth(),
                    };
                    let _ = self.scopes.declare(qualified.clone());
                    self.all_symbols.push(qualified);
                }
            }

            ParseNode::Program(items) => {
                for item in items { self.check_stmt(item); }
            }

            _ => {}
        }
    }

    fn absorb_scope(&mut self, popped: HashMap<String, Symbol>) {
        // symbols from inner scope go into the global snapshot (for display)
        // but are not re-added to current scope
        for (_, sym) in popped {
            // Already added in declare_sym; just make sure we don't duplicate
        }
    }

    // ── public entry ─────────────────────────

    pub fn analyze(&mut self, root: &ParseNode) -> SemanticResult {
        self.check_stmt(root);

        // Sort symbol table by scope depth then name
        let mut table = self.all_symbols.clone();
        table.sort_by(|a, b| a.scope_depth.cmp(&b.scope_depth).then(a.name.cmp(&b.name)));

        SemanticResult {
            symbol_table: table,
            errors: std::mem::take(&mut self.errors),
        }
    }
}

// ─────────────────────────────────────────────
//  Public API
// ─────────────────────────────────────────────

pub fn analyze(root: &ParseNode) -> SemanticResult {
    let mut analyzer = Analyzer::new();
    analyzer.analyze(root)
}

// ─────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::lexer::tokenize;
    use crate::compiler::parser::parse;

    fn run(src: &str) -> SemanticResult {
        let tokens = tokenize(src);
        let tree = parse(tokens).expect("parse failed");
        analyze(&tree)
    }

    fn wrap(body: &str) -> String {
        format!("!start\n{}\n!end", body)
    }

    #[test]
    fn test_basic_decl() {
        let r = run(&wrap(":int a = 42;"));
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        let sym = r.symbol_table.iter().find(|s| s.name == "a").unwrap();
        assert_eq!(sym.ty, FrType::Int);
        assert!(matches!(sym.value, SymbolValue::Int(42)));
    }

    #[test]
    fn test_type_mismatch() {
        let r = run(&wrap(":int a = 3.14;"));
        assert!(!r.errors.is_empty());
    }

    #[test]
    fn test_explicit_cast() {
        let r = run(&wrap(":int a = :int(3.7);"));
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        let sym = r.symbol_table.iter().find(|s| s.name == "a").unwrap();
        assert!(matches!(sym.value, SymbolValue::Int(3)));
    }

    #[test]
    fn test_undeclared_var() {
        let r = run(&wrap(":int a = b + 1;"));
        assert!(!r.errors.is_empty());
    }

    #[test]
    fn test_scope_isolation() {
        let r = run(&wrap(
            "!if (true) { :int x = 5; }\n:int y = x;"
        ));
        // 'x' should not be visible outside the if block
        assert!(!r.errors.is_empty());
    }

    #[test]
    fn test_array_decl() {
        let r = run(&wrap(":array<:int, 3> arr = [1, 2, 3];"));
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        let sym = r.symbol_table.iter().find(|s| s.name == "arr").unwrap();
        assert!(matches!(sym.ty, FrType::Array { .. }));
    }

    #[test]
    fn test_struct_def_and_decl() {
        let src = wrap(
            ":struct<Point> { :int x; :int y; };\n\
             :struct<Point> p = {x = 1, y = 2};",
        );
        let r = run(&src);
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        let sym = r.symbol_table.iter().find(|s| s.name == "p").unwrap();
        assert!(matches!(sym.ty, FrType::Struct { .. }));
    }

    #[test]
    fn test_func_return_type_check() {
        let src = wrap("!func bad() -> :int { !return 3.14; }");
        let r = run(&src);
        assert!(!r.errors.is_empty());
    }

    #[test]
    fn test_bitwise_on_float_error() {
        let r = run(&wrap(":float a = 1.0;\n:float b = 2.0;\n:int c = a & b;"));
        assert!(!r.errors.is_empty());
    }
}