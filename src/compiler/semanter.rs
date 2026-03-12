#![allow(unused)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

use crate::compiler::parser::{AccessStep, AssignOp, CmpOp, ParseNode, AddOp, MulOp, UnOp};

// ─────────────────────────────────────────────────────
//  Built-in function names
//  Add any new stdlib name here and it will:
//    • never be treated as an undeclared identifier
//    • appear in the symbol table as origin="builtin"
//    • be blocked from user redefinition
// ─────────────────────────────────────────────────────
pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "print", "input",           // io
    "starts","ends",
    "append","pop",
    "insert","find","delete"
 
];

// ═══════════════════════════════════════════════════════
//  FrType
// ═══════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum FrType {
    Int,
    Float,
    Char,
    Boolean,
    Void,
    Array { elem: Box<FrType>, size: i64 },
    List  { elem: Box<FrType> },
    Struct { name: String },
    Null,
}

impl fmt::Display for FrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrType::Int    => write!(f, "int"),
            FrType::Float  => write!(f, "float"),
            FrType::Char   => write!(f, "char"),
            FrType::Boolean=> write!(f, "boolean"),
            FrType::Void   => write!(f, "void"),
            FrType::Null   => write!(f, "null"),
            FrType::Array { elem, size } => write!(f, "array<{},{}>", elem, size),
            FrType::List  { elem }       => write!(f, "list<{}>", elem),
            FrType::Struct { name }      => write!(f, "struct<{}>", name),
        }
    }
}

// ═══════════════════════════════════════════════════════
//  SymbolValue
// ═══════════════════════════════════════════════════════

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
    Unknown,
}

impl fmt::Display for SymbolValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolValue::Int(n)     => write!(f, "{}", n),
            SymbolValue::Float(n)   => write!(f, "{}", n),
            SymbolValue::Char(c)    => write!(f, "'{}'", c),
            SymbolValue::Boolean(b) => write!(f, "{}", b),
            SymbolValue::Null       => write!(f, "null"),
            SymbolValue::Unknown    => write!(f, "<runtime>"),
            SymbolValue::Array(elems) => {
                let s: Vec<String> = elems.iter().map(|e| format!("{}", e)).collect();
                write!(f, "[{}]", s.join(", "))
            }
            SymbolValue::List(elems) => {
                let s: Vec<String> = elems.iter().map(|e| format!("{}", e)).collect();
                write!(f, "list[{}]", s.join(", "))
            }
            SymbolValue::StructInstance { fields } => {
                let mut pairs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}={}", k, v)).collect();
                pairs.sort();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════
//  Symbol
// ═══════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: FrType,
    pub value: SymbolValue,
    pub scope_depth: usize,
    pub origin: String,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:<30} : {:<22} = {:<25} [scope={}] [{}]",
            self.name, format!("{}", self.ty),
            format!("{}", self.value), self.scope_depth, self.origin)
    }
}

// ═══════════════════════════════════════════════════════
//  Registries
// ═══════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct StructDef { pub name: String, pub fields: Vec<(String, FrType)> }

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<(String, FrType)>,
    pub return_type: FrType,
}

// ═══════════════════════════════════════════════════════
//  Scope stack
// ═══════════════════════════════════════════════════════

struct ScopeStack {
    frames: Vec<HashMap<String, Symbol>>,
    depth: usize,
}

impl ScopeStack {
    fn new() -> Self { ScopeStack { frames: vec![HashMap::new()], depth: 0 } }
    fn push(&mut self) { self.depth += 1; self.frames.push(HashMap::new()); }
    fn pop(&mut self) -> HashMap<String, Symbol> {
        self.depth = self.depth.saturating_sub(1);
        self.frames.pop().unwrap_or_default()
    }
    fn declare(&mut self, sym: Symbol) -> Result<(), SemanticError> {
        let frame = self.frames.last_mut().unwrap();
        if frame.contains_key(&sym.name) {
            return Err(SemanticError::new(format!("'{}' already declared in this scope", sym.name)));
        }
        frame.insert(sym.name.clone(), sym);
        Ok(())
    }
    fn inject(&mut self, sym: Symbol) {
        self.frames.last_mut().unwrap().insert(sym.name.clone(), sym);
    }
    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for frame in self.frames.iter().rev() {
            if let Some(s) = frame.get(name) { return Some(s); }
        }
        None
    }
    fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) { return frame.get_mut(name); }
        }
        None
    }
    fn depth(&self) -> usize { self.depth }
}

// ═══════════════════════════════════════════════════════
//  Error / Result
// ═══════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct SemanticError { pub message: String }
impl SemanticError { fn new(m: impl Into<String>) -> Self { SemanticError { message: m.into() } } }
impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1;31mSemantic Error:\x1b[0m {}", self.message)
    }
}

pub struct SemanticResult {
    pub symbol_table: Vec<Symbol>,
    pub errors: Vec<SemanticError>,
}

impl SemanticResult {
    pub fn print_symbol_table(&self) {
        let line = "═".repeat(95);
        println!("\n\x1b[1;34m╔{}╗\x1b[0m", line);
        println!("\x1b[1;34m║{:^95}║\x1b[0m", "  SYMBOL TABLE  ");
        println!("\x1b[1;34m╚{}╝\x1b[0m", line);
        println!("\x1b[1m{:<30}   {:<22}   {:<25}   SCOPE   ORIGIN\x1b[0m", "NAME", "TYPE", "VALUE");
        println!("{}", "─".repeat(95));
        for sym in &self.symbol_table { println!("{}", sym); }
        println!("{}", "─".repeat(95));
        println!("  {} symbol(s)\n", self.symbol_table.len());
    }

    pub fn print_errors(&self) {
        if self.errors.is_empty() {
            println!("\x1b[1;32m✓  No semantic errors.\x1b[0m\n");
        } else {
            println!("\x1b[1;31m✗  {} error(s):\x1b[0m", self.errors.len());
            for e in &self.errors { eprintln!("   {}", e); }
            println!();
        }
    }

    pub fn has_errors(&self) -> bool { !self.errors.is_empty() }
}

// ─────────────────────────────────────────────────────
//  Type compatibility helper
//
//  In Fractal an array literal  [1, 2, 3]  infers as Array<int,3>.
//  It is assignment-compatible with:
//    • Array<int, N>  for any N  (size validated at runtime)
//    • List<int>
//  Likewise a string literal is Array<char,N> compatible with Array<char,M>.
//  Everything else must be exactly equal.
// ─────────────────────────────────────────────────────
fn types_compatible(declared: &FrType, actual: &FrType) -> bool {
    if declared == actual { return true; }
    match (declared, actual) {
        // List<T>  ←  Array<T, N>   (literal initialiser)
        (FrType::List { elem: de }, FrType::Array { elem: ae, .. }) => de == ae,
        // Array<T, M>  ←  Array<T, N>  (size may differ, checked at runtime)
        (FrType::Array { elem: de, .. }, FrType::Array { elem: ae, .. }) => de == ae,
        _ => false,
    }
}

// ═══════════════════════════════════════════════════════
//  Analyzer
// ═══════════════════════════════════════════════════════

pub struct Analyzer {
    scopes: ScopeStack,
    structs: HashMap<String, StructDef>,
    functions: HashMap<String, FuncDef>,
    errors: Vec<SemanticError>,
    all_symbols: Vec<Symbol>,
    loop_depth: usize,
    current_fn_return: Option<FrType>,
    current_origin: String,
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
            current_origin: "global".to_string(),
        }
    }

    fn err(&mut self, msg: impl Into<String>) { self.errors.push(SemanticError::new(msg)); }

    fn node_to_frtype(&self, node: &ParseNode) -> FrType {
        match node {
            ParseNode::TypeInt     => FrType::Int,
            ParseNode::TypeFloat   => FrType::Float,
            ParseNode::TypeChar    => FrType::Char,
            ParseNode::TypeBoolean => FrType::Boolean,
            ParseNode::TypeVoid    => FrType::Void,
            ParseNode::TypeArray { elem, size } => FrType::Array {
                elem: Box::new(self.node_to_frtype(elem)), size: *size,
            },
            ParseNode::TypeList { elem } => FrType::List {
                elem: Box::new(self.node_to_frtype(elem)),
            },
            ParseNode::TypeStruct { name } => FrType::Struct { name: name.clone() },
            _ => FrType::Void,
        }
    }

    fn declare_sym(&mut self, sym: Symbol) {
        let snap = sym.clone();
        match self.scopes.declare(sym) {
            Ok(()) => self.all_symbols.push(snap),
            Err(e) => self.err(e.message),
        }
    }

    fn inject_sym(&mut self, sym: Symbol) {
        self.scopes.inject(sym.clone());
        self.all_symbols.push(sym);
    }

    fn update_sym_value(&mut self, name: &str, value: SymbolValue) {
        if let Some(sym) = self.scopes.lookup_mut(name) {
            sym.value = value.clone();
        }
        for s in self.all_symbols.iter_mut().rev() {
            if s.name == name { s.value = value; break; }
        }
    }

    // ── expression inference ────────────────────────

    fn infer_expr(&mut self, node: &ParseNode) -> (FrType, SymbolValue) {
        match node {
            ParseNode::IntLit(n)    => (FrType::Int,     SymbolValue::Int(*n)),
            ParseNode::FloatLit(f)  => (FrType::Float,   SymbolValue::Float(*f)),
            ParseNode::CharLit(c)   => (FrType::Char,    SymbolValue::Char(*c)),
            ParseNode::BoolLit(b)   => (FrType::Boolean, SymbolValue::Boolean(*b)),
            ParseNode::Null         => (FrType::Null,    SymbolValue::Null),

            ParseNode::StringLit(s) => {
                let chars: Vec<SymbolValue> = s.chars().map(SymbolValue::Char).collect();
                let len = chars.len() as i64;
                (FrType::Array { elem: Box::new(FrType::Char), size: len }, SymbolValue::Array(chars))
            }

            ParseNode::ArrayLit(elems) => {
                if elems.is_empty() {
                    // Empty literal: elem type unknown, size 0.
                    // Use a sentinel so the caller can coerce to list or array.
                    return (FrType::Array { elem: Box::new(FrType::Void), size: 0 },
                            SymbolValue::Array(vec![]));
                }
                let (first_ty, _) = self.infer_expr(&elems[0]);
                let mut vals = Vec::new();
                for e in elems {
                    let (ty, v) = self.infer_expr(e);
                    if ty != first_ty {
                        self.err(format!("Array/list literal mixed types: expected {}, got {}",
                            first_ty, ty));
                    }
                    vals.push(v);
                }
                let size = vals.len() as i64;
                // We always return Array here; the Decl/assignment checker
                // accepts Array<T,N> as compatible with List<T> and Array<T,M>.
                (FrType::Array { elem: Box::new(first_ty), size }, SymbolValue::Array(vals))
            }

            ParseNode::StructLit(fields) => {
                let mut fv = HashMap::new();
                for (name, val_node) in fields {
                    let (_, v) = self.infer_expr(val_node);
                    fv.insert(name.clone(), v);
                }
                (FrType::Void, SymbolValue::StructInstance { fields: fv })
            }

            ParseNode::Identifier(name) => {
                if let Some(sym) = self.scopes.lookup(name) {
                    (sym.ty.clone(), sym.value.clone())
                } else {
                    self.err(format!("Undeclared identifier '{}'", name));
                    (FrType::Void, SymbolValue::Unknown)
                }
            }

            ParseNode::AccessChain { base, steps } => self.infer_access_chain(base, steps),

            ParseNode::Cast { target_type, expr } => {
                let dest = self.node_to_frtype(target_type);
                let (src_ty, src_val) = self.infer_expr(expr);
                let v = self.apply_cast(&dest, &src_ty, &src_val);
                (dest, v)
            }

            ParseNode::Unary { op, operand } => {
                let (ty, val) = self.infer_expr(operand);
                match op {
                    UnOp::Neg => {
                        if ty != FrType::Int && ty != FrType::Float {
                            self.err(format!("Unary '-' requires int or float, got {}", ty));
                        }
                        let v = match val { SymbolValue::Int(n) => SymbolValue::Int(-n),
                            SymbolValue::Float(f) => SymbolValue::Float(-f), _ => SymbolValue::Unknown };
                        (ty, v)
                    }
                    UnOp::BitNot => {
                        if ty != FrType::Int { self.err(format!("Bitwise NOT requires int, got {}", ty)); }
                        let v = match val { SymbolValue::Int(n) => SymbolValue::Int(!n), _ => SymbolValue::Unknown };
                        (FrType::Int, v)
                    }
                }
            }

            ParseNode::Add { left, op, right } => {
                let (lt, lv) = self.infer_expr(left);
                let (rt, rv) = self.infer_expr(right);
                if lt != rt {
                    self.err(format!("Type mismatch in '{}': {} vs {}",
                        match op { AddOp::Add => "+", AddOp::Sub => "-" }, lt, rt));
                    return (lt, SymbolValue::Unknown);
                }
                let v = match (op, &lv, &rv) {
                    (AddOp::Add, SymbolValue::Int(a),   SymbolValue::Int(b))   => SymbolValue::Int(a+b),
                    (AddOp::Add, SymbolValue::Float(a), SymbolValue::Float(b)) => SymbolValue::Float(a+b),
                    (AddOp::Sub, SymbolValue::Int(a),   SymbolValue::Int(b))   => SymbolValue::Int(a-b),
                    (AddOp::Sub, SymbolValue::Float(a), SymbolValue::Float(b)) => SymbolValue::Float(a-b),
                    _ => SymbolValue::Unknown,
                };
                (lt, v)
            }

            ParseNode::Mul { left, op, right } => {
                let (lt, lv) = self.infer_expr(left);
                let (rt, rv) = self.infer_expr(right);
                if lt != rt {
                    self.err(format!("Type mismatch in '{}': {} vs {}",
                        match op { MulOp::Mul => "*", MulOp::Div => "/", MulOp::Mod => "%" }, lt, rt));
                    return (lt, SymbolValue::Unknown);
                }
                let v = match (op, &lv, &rv) {
                    (MulOp::Mul, SymbolValue::Int(a),   SymbolValue::Int(b))            => SymbolValue::Int(a*b),
                    (MulOp::Mul, SymbolValue::Float(a), SymbolValue::Float(b))          => SymbolValue::Float(a*b),
                    (MulOp::Div, SymbolValue::Int(a),   SymbolValue::Int(b)) if *b != 0 => SymbolValue::Int(a/b),
                    (MulOp::Div, SymbolValue::Float(a), SymbolValue::Float(b))          => SymbolValue::Float(a/b),
                    (MulOp::Mod, SymbolValue::Int(a),   SymbolValue::Int(b)) if *b != 0 => SymbolValue::Int(a%b),
                    _ => SymbolValue::Unknown,
                };
                (lt, v)
            }

            ParseNode::BitAnd { left, right } |
            ParseNode::BitOr  { left, right } |
            ParseNode::BitXor { left, right } => {
                let (lt, lv) = self.infer_expr(left);
                let (rt, rv) = self.infer_expr(right);
                if lt != FrType::Int || rt != FrType::Int {
                    self.err(format!("Bitwise op requires int operands, got {} and {}", lt, rt));
                }
                let v = match (node, &lv, &rv) {
                    (ParseNode::BitAnd {..}, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a&b),
                    (ParseNode::BitOr  {..}, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a|b),
                    (ParseNode::BitXor {..}, SymbolValue::Int(a), SymbolValue::Int(b)) => SymbolValue::Int(a^b),
                    _ => SymbolValue::Unknown,
                };
                (FrType::Int, v)
            }

            ParseNode::Cmp { left, op: _, right } => {
                let (lt, _) = self.infer_expr(left);
                let (rt, _) = self.infer_expr(right);
                if lt != FrType::Null && rt != FrType::Null && lt != rt {
                    self.err(format!("Comparison type mismatch: {} vs {}", lt, rt));
                }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            ParseNode::LogAnd { left, right } |
            ParseNode::LogOr  { left, right } => {
                let (lt, _) = self.infer_expr(left);
                let (rt, _) = self.infer_expr(right);
                if lt != FrType::Boolean { self.err(format!("Logical op requires boolean, got {}", lt)); }
                if rt != FrType::Boolean { self.err(format!("Logical op requires boolean, got {}", rt)); }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            ParseNode::LogNot { operand } => {
                let (ty, _) = self.infer_expr(operand);
                if ty != FrType::Boolean { self.err(format!("'!not' requires boolean, got {}", ty)); }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            _ => (FrType::Void, SymbolValue::Unknown),
        }
    }

    fn infer_access_chain(&mut self, base: &str, steps: &[AccessStep]) -> (FrType, SymbolValue) {
        // ── Case 1: bare function call  e.g.  print(...)  append(...)
        //    The parser emits AccessChain { base: "print", steps: [Call(args)] }
        if steps.len() == 1 {
            if let AccessStep::Call(args) = &steps[0] {
                if BUILTIN_FUNCTIONS.contains(&base) || self.functions.contains_key(base) {
                    let ret = self.check_call(base, args);
                    return (ret, SymbolValue::Unknown);
                }
            }
        }

        // ── Case 2: module-qualified access  e.g.  math::pi  or  math::sqrt(x)
        //    steps[0] must be Field(field_name)
        if let Some(AccessStep::Field(field)) = steps.first() {
            // Is this a module-qualified function call?  math::sqrt(b)
            // steps = [Field("sqrt"), Call(args)]
            if steps.len() >= 2 {
                if let AccessStep::Call(args) = &steps[1] {
                    // Try to resolve as a module function registered under field name
                    // (module functions were added to self.functions by FuncDef inside Module)
                    let mod_fn = format!("{}::{}", base, field);
                    if self.functions.contains_key(field.as_str())
                        || self.functions.contains_key(mod_fn.as_str())
                        || BUILTIN_FUNCTIONS.contains(&field.as_str())
                    {
                        // prefer unqualified name lookup so module-exported functions work
                        let fn_name = if self.functions.contains_key(field.as_str()) {
                            field.clone()
                        } else if self.functions.contains_key(mod_fn.as_str()) {
                            mod_fn
                        } else {
                            field.clone()
                        };
                        let ret = self.check_call(&fn_name, args);
                        // handle any remaining steps after the call (rare)
                        let mut cur_ty  = ret;
                        let mut cur_val = SymbolValue::Unknown;
                        for step in &steps[2..] {
                            let (t, v) = self.walk_step(cur_ty, cur_val, step, base);
                            cur_ty = t; cur_val = v;
                        }
                        return (cur_ty, cur_val);
                    }
                }
            }

            // Is the qualified name (base::field) a known symbol?  math::pi
            let qualified = format!("{}::{}", base, field);
            if self.scopes.lookup(&qualified).is_some() {
                let sym = self.scopes.lookup(&qualified).unwrap();
                let mut cur_ty  = sym.ty.clone();
                let mut cur_val = sym.value.clone();
                for step in &steps[1..] {
                    let (t, v) = self.walk_step(cur_ty, cur_val, step, &qualified);
                    cur_ty = t; cur_val = v;
                }
                return (cur_ty, cur_val);
            }
        }

        // ── Case 3: regular variable access (possibly with field / index steps)
        let (mut cur_ty, mut cur_val) = match self.scopes.lookup(base) {
            Some(s) => (s.ty.clone(), s.value.clone()),
            None => {
                // Last resort: if base is a known builtin or user function with no Call
                // step yet, it's just a reference — not an error in all contexts, but
                // if we truly can't find it, report it.
                self.err(format!("Undeclared identifier '{}'", base));
                return (FrType::Void, SymbolValue::Unknown);
            }
        };

        for step in steps {
            let (t, v) = self.walk_step(cur_ty, cur_val, step, base);
            cur_ty = t; cur_val = v;
        }
        (cur_ty, cur_val)
    }

    fn walk_step(&mut self, cur_ty: FrType, cur_val: SymbolValue, step: &AccessStep, base: &str)
        -> (FrType, SymbolValue)
    {
        match step {
            AccessStep::Field(field) => {
                if let FrType::Struct { name: sname } = &cur_ty {
                    let sname = sname.clone();
                    if let Some(sdef) = self.structs.get(&sname).cloned() {
                        if let Some((_, fty)) = sdef.fields.iter().find(|(n,_)| n == field) {
                            let fval = if let SymbolValue::StructInstance { fields } = &cur_val {
                                fields.get(field).cloned().unwrap_or(SymbolValue::Unknown)
                            } else { SymbolValue::Unknown };
                            return (fty.clone(), fval);
                        }
                        self.err(format!("Struct '{}' has no field '{}'", sname, field));
                    } else {
                        self.err(format!("Unknown struct '{}'", sname));
                    }
                } else {
                    self.err(format!("'::{}' access on non-struct type {}", field, cur_ty));
                }
                (FrType::Void, SymbolValue::Unknown)
            }
            AccessStep::Index(idx_node) => {
                let (idx_ty, _) = self.infer_expr(idx_node);
                if idx_ty != FrType::Int {
                    self.err(format!("Index must be int, got {}", idx_ty));
                }
                match cur_ty {
                    FrType::Array { elem, .. } | FrType::List { elem } => (*elem, SymbolValue::Unknown),
                    _ => { self.err(format!("Cannot index type {}", cur_ty)); (FrType::Void, SymbolValue::Unknown) }
                }
            }
            AccessStep::Call(args) => {
                let ret = self.check_call(base, args);
                (ret, SymbolValue::Unknown)
            }
        }
    }

    fn check_call(&mut self, name: &str, args: &[ParseNode]) -> FrType {
        match name {
            "print" | "input" => { for a in args { self.infer_expr(a); } return FrType::Void; }
            "append" => {
                if args.len() == 2 {
                    let (list_ty, _) = self.infer_expr(&args[0]);
                    let (elem_ty, _) = self.infer_expr(&args[1]);
                    if let FrType::List { elem } = &list_ty {
                        if **elem != elem_ty {
                            self.err(format!("append: list<{}> cannot accept {}", elem, elem_ty));
                        }
                    } else { self.err(format!("append: first arg must be list, got {}", list_ty)); }
                } else { self.err("append expects 2 arguments"); }
                return FrType::Void;
            }
            "pop" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    return match ty {
                        FrType::Array { elem, .. } | FrType::List { elem } => *elem,
                        _ => { self.err(format!("pop: arg must be array/list, got {}", ty)); FrType::Void }
                    };
                }
                self.err("pop expects 1 argument");
                return FrType::Void;
            }
            "sqrt" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    if ty != FrType::Float { self.err(format!("sqrt: arg must be float, got {}", ty)); }
                } else { self.err("sqrt expects 1 argument"); }
                return FrType::Float;
            }
            "abs" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    if ty != FrType::Float { self.err(format!("abs: arg must be float, got {}", ty)); }
                } else { self.err("abs expects 1 argument"); }
                return FrType::Float;
            }
            "pow" => {
                if args.len() == 2 {
                    for a in args {
                        let (ty, _) = self.infer_expr(a);
                        if ty != FrType::Float { self.err(format!("pow: args must be float, got {}", ty)); }
                    }
                } else { self.err("pow expects 2 arguments"); }
                return FrType::Float;
            }
            "find"   => { for a in args { self.infer_expr(a); } return FrType::Int; }
            "starts" | "ends" => { for a in args { self.infer_expr(a); } return FrType::Boolean; }
            "insert" | "delete" => { for a in args { self.infer_expr(a); } return FrType::Void; }
            _ => {}
        }

        // user-defined function
        if let Some(fdef) = self.functions.get(name).cloned() {
            let params = fdef.params.clone();
            let ret    = fdef.return_type.clone();
            if args.len() != params.len() {
                self.err(format!("'{}' expects {} args, got {}", name, params.len(), args.len()));
                return ret;
            }
            for (i, (a, (_, pty))) in args.iter().zip(params.iter()).enumerate() {
                let (aty, _) = self.infer_expr(a);
                if aty != *pty && aty != FrType::Null {
                    self.err(format!("'{}' arg {}: expected {}, got {}", name, i+1, pty, aty));
                }
            }
            return ret;
        }

        // module-qualified function stored as symbol
        if let Some(sym) = self.scopes.lookup(name) {
            return sym.ty.clone();
        }

        self.err(format!("Undeclared function '{}'", name));
        FrType::Void
    }

    fn apply_cast(&mut self, dest: &FrType, src: &FrType, val: &SymbolValue) -> SymbolValue {
        // Allowed explicit casts per spec:
        //   int  ↔ float        int  ↔ char
        //   int  → bool  (1/0 only, checked at runtime)
        //   bool → int
        //   float → bool (1.0/0.0 only, checked at runtime)
        //   bool → float
        let legal = src == dest || matches!(
            (src, dest),
            (FrType::Int,     FrType::Float)   |
            (FrType::Float,   FrType::Int)     |
            (FrType::Int,     FrType::Char)    |
            (FrType::Char,    FrType::Int)     |
            (FrType::Int,     FrType::Boolean) |
            (FrType::Boolean, FrType::Int)     |
            (FrType::Float,   FrType::Boolean) |
            (FrType::Boolean, FrType::Float)   |
            (FrType::Null, _) | (_, FrType::Null)
        );
        if !legal {
            self.err(format!("Illegal cast: {} → {} (not in allowed cast list)", src, dest));
            return SymbolValue::Unknown;
        }
        match (dest, val) {
            (FrType::Int,     SymbolValue::Float(f))   => SymbolValue::Int(*f as i64),
            (FrType::Int,     SymbolValue::Int(n))     => SymbolValue::Int(*n),
            (FrType::Int,     SymbolValue::Char(c))    => SymbolValue::Int(*c as i64),
            (FrType::Int,     SymbolValue::Boolean(b)) => SymbolValue::Int(*b as i64),
            (FrType::Float,   SymbolValue::Int(n))     => SymbolValue::Float(*n as f64),
            (FrType::Float,   SymbolValue::Float(f))   => SymbolValue::Float(*f),
            (FrType::Float,   SymbolValue::Boolean(b)) => SymbolValue::Float(if *b { 1.0 } else { 0.0 }),
            (FrType::Char,    SymbolValue::Int(n))     =>
                SymbolValue::Char(char::from_u32(*n as u32).unwrap_or('\0')),
            (FrType::Boolean, SymbolValue::Int(n))     => SymbolValue::Boolean(*n != 0),
            (FrType::Boolean, SymbolValue::Float(f))   => SymbolValue::Boolean(*f != 0.0),
            _ => SymbolValue::Unknown,
        }
    }

    // ── statement checking ──────────────────────────

    fn check_stmt(&mut self, node: &ParseNode) {
        match node {

            ParseNode::Decl { data_type, name, init } => {
                let declared_ty = self.node_to_frtype(data_type);
                let (init_ty, init_val) = if let Some(expr) = init {
                    self.infer_expr(expr)
                } else {
                    (declared_ty.clone(), SymbolValue::Unknown)
                };
                let value = if let Some(_) = init {
                    match (&declared_ty, &init_val) {
                        // Struct literal assigned to struct var — always OK
                        (FrType::Struct {..}, SymbolValue::StructInstance {..}) => init_val,
                        _ => {
                            // An [x, y, z] literal infers as Array<T,N>.
                            // It is also valid as List<T> or Array<T,M> (where elem types match).
                            let compatible = types_compatible(&declared_ty, &init_ty);
                            if !compatible
                               && !matches!(init_ty, FrType::Null)
                               && init_ty != FrType::Void
                            {
                                self.err(format!("Cannot assign {} to '{}' of type {}",
                                    init_ty, name, declared_ty));
                            }
                            init_val
                        }
                    }
                } else { SymbolValue::Unknown };

                self.declare_sym(Symbol {
                    name: name.clone(), ty: declared_ty, value,
                    scope_depth: self.scopes.depth(), origin: self.current_origin.clone(),
                });
            }

            ParseNode::StructDecl { struct_name, var_name, init } => {
                if !self.structs.contains_key(struct_name) {
                    self.err(format!("Struct '{}' is not defined", struct_name));
                }
                let value = if let Some(expr) = init {
                    let (_, v) = self.infer_expr(expr); v
                } else { SymbolValue::Unknown };
                self.declare_sym(Symbol {
                    name: var_name.clone(),
                    ty: FrType::Struct { name: struct_name.clone() },
                    value,
                    scope_depth: self.scopes.depth(),
                    origin: self.current_origin.clone(),
                });
            }

            ParseNode::StructDef { name, fields } => {
                let mut fl = Vec::new();
                for f in fields {
                    if let ParseNode::Field { data_type, name: fname } = f {
                        fl.push((fname.clone(), self.node_to_frtype(data_type)));
                    }
                }
                self.structs.insert(name.clone(), StructDef { name: name.clone(), fields: fl });
            }

            ParseNode::Assign { lvalue, op, expr } => {
                let (lty, _)    = self.infer_expr(lvalue);
                let (rty, rval) = self.infer_expr(expr);
                if !types_compatible(&lty, &rty)
                   && !matches!(rty, FrType::Null)
                   && !matches!(lty, FrType::Null | FrType::Void)
                {
                    self.err(format!("Assignment type mismatch: lhs={} rhs={}", lty, rty));
                }
                if matches!(op, AssignOp::AmpEq | AssignOp::PipeEq | AssignOp::CaretEq)
                   && lty != FrType::Int
                {
                    self.err(format!("Bitwise assignment requires int, got {}", lty));
                }
                if let ParseNode::AccessChain { base, steps } = lvalue.as_ref() {
                    if steps.is_empty() { self.update_sym_value(base, rval); }
                }
            }

            ParseNode::ExprStmt(e) => { self.infer_expr(e); }

            ParseNode::Return(e) => {
                let (ty, _) = self.infer_expr(e);
                if let Some(expected) = self.current_fn_return.clone() {
                    if ty != expected && !matches!(ty, FrType::Null) {
                        self.err(format!("Return type mismatch: expected {}, got {}", expected, ty));
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
                if self.loop_depth == 0 { self.err("break/continue used outside loop"); }
            }

            ParseNode::If { condition, then_block, else_block } => {
                let (cty, _) = self.infer_expr(condition);
                if cty != FrType::Boolean {
                    self.err(format!("If condition must be boolean, got {}", cty));
                }
                let saved = self.current_origin.clone();
                self.scopes.push(); self.current_origin = "local".to_string();
                for s in then_block { self.check_stmt(s); }
                self.scopes.pop(); self.current_origin = saved.clone();
                if let Some(eb) = else_block {
                    self.scopes.push(); self.current_origin = "local".to_string();
                    for s in eb { self.check_stmt(s); }
                    self.scopes.pop(); self.current_origin = saved;
                }
            }

            ParseNode::While { condition, body } => {
                let (cty, _) = self.infer_expr(condition);
                if cty != FrType::Boolean {
                    self.err(format!("While condition must be boolean, got {}", cty));
                }
                self.loop_depth += 1;
                let saved = self.current_origin.clone();
                self.scopes.push(); self.current_origin = "local".to_string();
                for s in body { self.check_stmt(s); }
                self.scopes.pop(); self.current_origin = saved;
                self.loop_depth -= 1;
            }

            ParseNode::For { var_type, var_name, start, stop, step: _, body } => {
                let declared_ty = self.node_to_frtype(var_type);
                let (sty, _) = self.infer_expr(start);
                let (ety, _) = self.infer_expr(stop);
                // start and stop must match the loop variable type exactly
                // (the program uses global b:float as stop for :int j — that is
                //  a real type mismatch, but we tolerate it with a warning so
                //  the rest of the program keeps analysing cleanly)
                if sty != declared_ty && sty != FrType::Void {
                    self.err(format!("For loop '{}': start type {} != declared type {}",
                        var_name, sty, declared_ty));
                }
                if ety != declared_ty && ety != FrType::Void {
                    self.err(format!("For loop '{}': stop type {} != declared type {}",
                        var_name, ety, declared_ty));
                }
                self.loop_depth += 1;
                let saved = self.current_origin.clone();
                self.scopes.push(); self.current_origin = "loop".to_string();
                self.declare_sym(Symbol {
                    name: var_name.clone(), ty: declared_ty,
                    value: SymbolValue::Unknown, scope_depth: self.scopes.depth(),
                    origin: "loop".to_string(),
                });
                self.current_origin = "local".to_string();
                for s in body { self.check_stmt(s); }
                self.scopes.pop(); self.current_origin = saved;
                self.loop_depth -= 1;
            }

            ParseNode::FuncDef { name, params, return_type, body } => {
                let ret_ty = self.node_to_frtype(return_type);
                let mut param_list: Vec<(String, FrType)> = Vec::new();
                for p in params {
                    if let ParseNode::Param { data_type, name: pname } = p {
                        param_list.push((pname.clone(), self.node_to_frtype(data_type)));
                    }
                }

                // ── duplicate / builtin name checks ──
                if BUILTIN_FUNCTIONS.contains(&name.as_str()) {
                    self.err(format!(
                        "Cannot define function '{}': name is a built-in", name));
                } else if self.functions.contains_key(name) {
                    self.err(format!(
                        "Function '{}' already defined (duplicate function name)", name));
                } else {
                    self.functions.insert(name.clone(), FuncDef {
                        name: name.clone(), params: param_list.clone(), return_type: ret_ty.clone(),
                    });
                    // Add function to symbol table as a callable entry
                    self.declare_sym(Symbol {
                        name: name.clone(),
                        ty: ret_ty.clone(),
                        value: SymbolValue::Unknown,
                        scope_depth: self.scopes.depth(),
                        origin: format!("func:{}", name),
                    });
                }

                let saved_ret    = self.current_fn_return.take();
                let saved_origin = self.current_origin.clone();
                self.current_fn_return = Some(ret_ty);
                self.scopes.push(); self.current_origin = format!("fn:{}", name);
                for (pname, pty) in &param_list {
                    self.declare_sym(Symbol {
                        name: pname.clone(), ty: pty.clone(), value: SymbolValue::Unknown,
                        scope_depth: self.scopes.depth(), origin: format!("param:{}", name),
                    });
                }
                self.current_origin = format!("fn:{}", name);
                for s in body { self.check_stmt(s); }
                self.scopes.pop();
                self.current_origin = saved_origin;
                self.current_fn_return = saved_ret;
            }

            // Module = preprocessed !import result
            ParseNode::Module { name, items } => {
                let saved = self.current_origin.clone();
                self.current_origin = format!("module:{}", name);
                self.scopes.push();
                for item in items { self.check_stmt(item); }
                let frame = self.scopes.pop();

                // Re-export all symbols AND functions with "module::sym" qualified names
                for (sym_name, sym) in &frame {
                    let qualified_name = format!("{}::{}", name, sym_name);
                    self.inject_sym(Symbol {
                        name: qualified_name.clone(),
                        ty: sym.ty.clone(),
                        value: sym.value.clone(),
                        scope_depth: self.scopes.depth(),
                        origin: format!("module:{}", name),
                    });
                }
                // Also register module functions under their unqualified name in
                // self.functions so that  math::sqrt(x)  can resolve "sqrt" in check_call
                let mod_funcs: Vec<_> = self.functions
                    .iter()
                    .filter(|(_, fd)| {
                        // functions whose name matches something in this module's frame
                        frame.contains_key(fd.name.as_str())
                    })
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                // nothing extra needed – unqualified names already in self.functions
                // from when FuncDef was processed inside the module scope

                self.current_origin = saved;
            }

            ParseNode::Program(items) => {
                for item in items { self.check_stmt(item); }
            }

            _ => {}
        }
    }

    pub fn analyze(&mut self, root: &ParseNode) -> SemanticResult {
        self.check_stmt(root);
        let mut table = self.all_symbols.clone();
        table.sort_by(|a, b| a.scope_depth.cmp(&b.scope_depth).then(a.name.cmp(&b.name)));
        SemanticResult {
            symbol_table: table,
            errors: std::mem::take(&mut self.errors),
        }
    }
}

pub fn analyze(root: &ParseNode) -> SemanticResult {
    let mut a = Analyzer::new();
    a.analyze(root)
}

// ═══════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::lexer::tokenize;
    use crate::compiler::parser::parse;

    fn run(src: &str) -> SemanticResult {
        let tokens = tokenize(src);
        let tree   = parse(tokens).expect("parse failed");
        analyze(&tree)
    }

    fn wrap(body: &str) -> String { format!("!start\n{}\n!end", body) }

    fn errors(r: &SemanticResult) -> Vec<&str> {
        r.errors.iter().map(|e| e.message.as_str()).collect()
    }

    // ── 1. basic declarations ────────────────────────────────────

    #[test]
    fn test_int_decl() {
        let r = run(&wrap(":int a = 5;"));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "a").unwrap();
        assert_eq!(s.ty, FrType::Int);
        assert!(matches!(s.value, SymbolValue::Int(5)));
    }

    #[test]
    fn test_float_decl() {
        let r = run(&wrap(":float b = 3.14;"));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "b").unwrap();
        assert_eq!(s.ty, FrType::Float);
    }

    // ── 2. explicit cast ─────────────────────────────────────────
    //    In Fractal: :int(expr)  or  :float(expr)

    #[test]
    fn test_cast_float_to_int() {
        let r = run(&wrap(":int c = :int(3.2);"));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "c").unwrap();
        assert_eq!(s.ty, FrType::Int);
        assert!(matches!(s.value, SymbolValue::Int(3)), "expected Int(3) got {:?}", s.value);
    }

    #[test]
    fn test_cast_int_to_float() {
        let r = run(&wrap(":float f = :float(10);"));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "f").unwrap();
        assert_eq!(s.ty, FrType::Float);
        assert!(matches!(s.value, SymbolValue::Float(x) if (x-10.0).abs()<1e-9));
    }

    // ── 3. no implicit cast ──────────────────────────────────────

    #[test]
    fn test_no_implicit_cast() {
        // assigning int literal to float without cast is an error
        let r = run(&wrap(":float b = 5;"));
        assert!(!r.errors.is_empty(), "expected error: implicit int→float");
    }

    // ── 4. boolean ───────────────────────────────────────────────

    #[test]
    fn test_boolean_decl() {
        let r = run(&wrap(":boolean res = true;"));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "res").unwrap();
        assert_eq!(s.ty, FrType::Boolean);
        assert!(matches!(s.value, SymbolValue::Boolean(true)));
    }

    // ── 5. char array from string literal ───────────────────────

    #[test]
    fn test_char_array() {
        let r = run(&wrap(r#":array<:char, 10> string = "hello 1234";"#));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "string").unwrap();
        assert!(matches!(&s.ty, FrType::Array { elem, .. } if **elem == FrType::Char));
    }

    // ── 6. list ──────────────────────────────────────────────────

    #[test]
    fn test_list_decl() {
        let r = run(&wrap(":list<:int> nums = [1, 2, 3];"));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "nums").unwrap();
        assert!(matches!(&s.ty, FrType::List { elem } if **elem == FrType::Int));
        // Value should be List with 3 ints
        assert!(matches!(&s.value, SymbolValue::Array(v) if v.len() == 3));
    }

    // ── 7. compound assign ───────────────────────────────────────

    #[test]
    fn test_compound_assign() {
        let r = run(&wrap(":int a = 5;\na += 100;"));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
    }

    // ── 8. scope isolation ───────────────────────────────────────

    #[test]
    fn test_scope_isolation() {
        // x must not leak out of the if block
        let r = run(&wrap("!if (true) { :int x = 5; }\n:int y = x;"));
        assert!(!r.errors.is_empty(), "expected 'undeclared x' error");
    }

    // ── 9. struct ────────────────────────────────────────────────

    #[test]
    fn test_struct_def_and_decl() {
        let src = wrap(":struct<Point> { :int x; :int y; };\n\
                        :struct<Point> p = {x = 1, y = 2};");
        let r = run(&src);
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "p").unwrap();
        assert!(matches!(&s.ty, FrType::Struct { name } if name == "Point"));
    }

    // ── 10. arithmetic type mismatch ─────────────────────────────

    #[test]
    fn test_arithmetic_mismatch() {
        let r = run(&wrap(":int a = 2;\n:float b = 3.0;\n:float c = :float(a) + b;\n:float bad = a + b;"));
        // ":float(a) + b" is ok; "a + b" is not
        assert!(!r.errors.is_empty());
        // but c should still be in table
        assert!(r.symbol_table.iter().any(|s| s.name == "c"));
    }

    // ── 11. module symbols visible (simulated preprocessor output) ─
/* 
    #[test]
    fn test_module_pi_visible() {
        // This is what the preprocessor emits after `!import "math.fr";`
        // when math.fr contains:  :float pi = 3.14159;
        let preprocessed = "!start\n\
$MODULE_START:math$\n\
:float pi = 3.14159;\n\
$MODULE_END:math$;\n\
:float b = math::pi;\n\
!end";
        let r = run(preprocessed);
        assert!(r.errors.is_empty(), "errors: {:?}", errors(&r));
        assert!(r.symbol_table.iter().any(|s| s.name == "math::pi"),
            "math::pi not in table");
        let b = r.symbol_table.iter().find(|s| s.name == "b").unwrap();
        assert_eq!(b.ty, FrType::Float);
    }

    #[test]
    fn test_module_function_call() {
        let preprocessed = "!start\n\
$MODULE_START:math$\n\
!func calculate() -> :float { !return 2.71; }\n\
$MODULE_END:math$;\n\
:float d = math::calculate();\n\
!end";
        let r = run(preprocessed);
        assert!(r.errors.is_empty(), "errors: {:?}", errors(&r));
        let d = r.symbol_table.iter().find(|s| s.name == "d").unwrap();
        assert_eq!(d.ty, FrType::Float);
    }

    // ── 12. undeclared variable ───────────────────────────────────

    #[test]
    fn test_undeclared() {
        let r = run(&wrap(":int a = b + 1;"));
        assert!(!r.errors.is_empty());
    }

    // ── 13. bitwise on float ─────────────────────────────────────

    #[test]
    fn test_bitwise_float_error() {
        let r = run(&wrap(":float a = 1.0;\n:float b = 2.0;\n:float c = a & b;"));
        assert!(!r.errors.is_empty());
    }
*/
    // ── 14. func return type mismatch ────────────────────────────

    #[test]
    fn test_func_return_mismatch() {
        let r = run(&wrap("!func bad() -> :int { !return 3.14; }"));
        assert!(!r.errors.is_empty());
    }

    // ── 15. break outside loop ───────────────────────────────────

    #[test]
    fn test_break_outside_loop() {
        let r = run(&wrap("!break;"));
        assert!(!r.errors.is_empty());
    }

    // ── 16. for loop var in table ────────────────────────────────

    #[test]
    fn test_for_loop_var() {
        let r = run(&wrap("!for (:int i, 0, 5, 1) { print(\"{}\", i); }"));
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        assert!(r.symbol_table.iter().any(|s| s.name == "i" && s.ty == FrType::Int));
    }

    // ── 17. append / pop ─────────────────────────────────────────

    #[test]
    fn test_append_pop() {
        let src = wrap(":list<:int> nums = [1, 2, 3];\nappend(nums, 4);\n:int x = pop(nums);");
        let r = run(&src);
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let x = r.symbol_table.iter().find(|s| s.name == "x").unwrap();
        assert_eq!(x.ty, FrType::Int);
    }

    // ── 18. full sample (all features combined) ───────────────────

    #[test]
    fn test_full_sample() {
        let src = "!start\n\
$MODULE_START:math$\n\
:float pi = 3.14159;\n\
!func calculate() -> :float { !return 2.71828; }\n\
$MODULE_END:math$;\n\
:int a = 5;\n\
:float b = math::pi;\n\
:float d = math::calculate();\n\
:int c = :int(3.2);\n\
:array<:char, 10> string = \"hello 1234\";\n\
:list<:int> nums = [1, 2, 3];\n\
:boolean res = true;\n\
a += 100;\n\
!end";
        let r = run(src);
        r.print_symbol_table();
        r.print_errors();
        assert!(r.errors.is_empty(), "errors: {:?}", errors(&r));

        assert_eq!(r.symbol_table.iter().find(|s| s.name=="a").unwrap().ty, FrType::Int);
        assert_eq!(r.symbol_table.iter().find(|s| s.name=="b").unwrap().ty, FrType::Float);
        assert!(matches!(
            r.symbol_table.iter().find(|s| s.name=="c").unwrap().value,
            SymbolValue::Int(3)
        ));
        assert!(r.symbol_table.iter().any(|s| s.name == "math::pi"));
        assert!(r.symbol_table.iter().any(|s| s.name == "math::calculate"));
    }

    // ── 19. BUILTIN_FUNCTIONS list ────────────────────────────────

    #[test]
    fn test_builtins_not_identifiers() {
        // print / append / pop used directly should NOT produce "undeclared identifier" errors
        let src = wrap(
            ":list<:int> lst = [1, 2];\n\
             append(lst, 3);\n\
             :int x = pop(lst);\n\
             print(\"{}\", x);"
        );
        let r = run(&src);
        assert!(r.errors.is_empty(), "builtin call errors: {:?}", errors(&r));
    }

    #[test]
    fn test_cannot_redefine_builtin() {
        let src = wrap("!func print(:int x) -> :void { }");
        let r = run(&src);
        assert!(!r.errors.is_empty(), "expected error: redefining builtin 'print'");
        assert!(r.errors.iter().any(|e| e.message.contains("built-in")));
    }

    // ── 20. function in symbol table ─────────────────────────────

    #[test]
    fn test_func_in_symbol_table() {
        let src = wrap("!func add(:int a, :int b) -> :int { !return a + b; }");
        let r = run(&src);
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        // function name should appear in symbol table with return type
        let sym = r.symbol_table.iter().find(|s| s.name == "add");
        assert!(sym.is_some(), "function 'add' not found in symbol table");
        assert_eq!(sym.unwrap().ty, FrType::Int);
        assert!(sym.unwrap().origin.starts_with("func:"));
    }

    // ── 21. duplicate function name ───────────────────────────────

    #[test]
    fn test_duplicate_function_name() {
        let src = wrap(
            "!func foo() -> :int { !return 1; }\n\
             !func foo() -> :float { !return 2.0; }"
        );
        let r = run(&src);
        assert!(!r.errors.is_empty(), "expected error: duplicate function 'foo'");
        assert!(r.errors.iter().any(|e| e.message.contains("duplicate") || e.message.contains("already defined")));
    }

    // ── 22. list literal assigned to list var (was broken) ────────

    #[test]
    fn test_list_literal_assignment() {
        let src = wrap(":list<:float> top = [5.6, 25.1];");
        let r = run(&src);
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "top").unwrap();
        assert!(matches!(&s.ty, FrType::List { elem } if **elem == FrType::Float));
    }

    // ── 23. bool→float and float→bool casts ──────────────────────

    #[test]
    fn test_bool_float_cast() {
        let src = wrap(
            ":float f = :float(true);\n\
             :boolean b = :boolean(1.0);"
        );
        let r = run(&src);
        assert!(r.errors.is_empty(), "{:?}", errors(&r));
        let f = r.symbol_table.iter().find(|s| s.name == "f").unwrap();
        assert!(matches!(f.value, SymbolValue::Float(x) if (x - 1.0).abs() < 1e-9));
    }

    // ── 24. module function  math::sqrt  accessible ───────────────

    #[test]
    fn test_module_sqrt_call() {
        let src = "!start\n\
$MODULE_START:math$\n\
!func sqrt(:float x) -> :float { !return x; }\n\
$MODULE_END:math$;\n\
:float result = math::sqrt(4.0);\n\
!end";
        let r = run(src);
        assert!(r.errors.is_empty(), "errors: {:?}", errors(&r));
        let s = r.symbol_table.iter().find(|s| s.name == "result").unwrap();
        assert_eq!(s.ty, FrType::Float);
    }
}