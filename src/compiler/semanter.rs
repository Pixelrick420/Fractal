#![allow(unused)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

use crate::compiler::parser::{AccessStep, AddOp, AssignOp, CmpOp, MulOp, ParseNode, UnOp};

pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "print", "input", "starts", "ends", "append", "pop", "insert", "find", "delete", "len", "pow",
    "abs", "sqrt",
];

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

#[derive(Debug, Clone)]
pub enum SymbolValue {
    Int(i64),
    Float(f64),
    Char(char),
    Boolean(bool),
    Null,
    Array(Vec<SymbolValue>),
    List(Vec<SymbolValue>),
    StructInstance {
        fields: HashMap<String, SymbolValue>,
    },
    Unknown,
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
                let mut pairs: Vec<String> =
                    fields.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                pairs.sort();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}

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
        write!(
            f,
            "{:<30} : {:<22} = {:<25} [scope={}] [{}]",
            self.name,
            format!("{}", self.ty),
            format!("{}", self.value),
            self.scope_depth,
            self.origin
        )
    }
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, FrType)>,
}

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<(String, FrType)>,
    pub return_type: FrType,
}

struct ScopeStack {
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
                "'{}' is already declared in this scope — \
                 rename one of the declarations or move it to a different scope",
                sym.name
            )));
        }
        frame.insert(sym.name.clone(), sym);
        Ok(())
    }
    fn inject(&mut self, sym: Symbol) {
        self.frames
            .last_mut()
            .unwrap()
            .insert(sym.name.clone(), sym);
    }
    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for frame in self.frames.iter().rev() {
            if let Some(s) = frame.get(name) {
                return Some(s);
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
    fn depth(&self) -> usize {
        self.depth
    }
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
    pub context: String,
}
impl SemanticError {
    fn new(m: impl Into<String>) -> Self {
        SemanticError {
            message: m.into(),
            context: String::new(),
        }
    }
    fn with_context(m: impl Into<String>, ctx: impl Into<String>) -> Self {
        SemanticError {
            message: m.into(),
            context: ctx.into(),
        }
    }
}
impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (desc, hint) = match self.message.split_once(" — ") {
            Some((d, h)) => (d.trim(), Some(h.trim())),
            None => (self.message.trim(), None),
        };

        writeln!(f, "\x1b[1;31merror[S000]\x1b[0m\x1b[1m: {desc}\x1b[0m")?;

        if !self.context.is_empty() {
            writeln!(
                f,
                " \x1b[1;34m       =\x1b[0m \x1b[1;36mnote\x1b[0m: {}",
                self.context
            )?;
        }

        if let Some(hint_text) = hint {
            for line in hint_text.split('\n') {
                let line = line.trim();
                if !line.is_empty() {
                    writeln!(
                        f,
                        " \x1b[1;34m       =\x1b[0m \x1b[1;32mhint\x1b[0m: {line}"
                    )?;
                }
            }
        }

        Ok(())
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
        println!(
            "\x1b[1m{:<30}   {:<22}   {:<25}   SCOPE   ORIGIN\x1b[0m",
            "NAME", "TYPE", "VALUE"
        );
        println!("{}", "─".repeat(95));
        for sym in &self.symbol_table {
            println!("{}", sym);
        }
        println!("{}", "─".repeat(95));
        println!("  {} symbol(s)\n", self.symbol_table.len());
    }

    pub fn print_errors(&self) {
        if self.errors.is_empty() {
            println!("\x1b[1;32m✓  No semantic errors.\x1b[0m\n");
        } else {
            let n = self.errors.len();
            eprintln!(
                "\x1b[1;31m✗  {} semantic error{} found:\x1b[0m\n",
                n,
                if n == 1 { "" } else { "s" }
            );
            for e in &self.errors {
                eprintln!("{}", e);
            }
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

fn types_compatible(declared: &FrType, actual: &FrType) -> bool {
    if declared == actual {
        return true;
    }
    match (declared, actual) {
        (FrType::List { elem: de }, FrType::Array { elem: ae, .. }) => de == ae,

        (
            FrType::Array { elem: de, size: ds },
            FrType::Array {
                elem: ae,
                size: as_,
            },
        ) => de == ae && ds == as_,

        (FrType::Char, FrType::Array { elem, size: 1 }) => **elem == FrType::Char,

        _ => false,
    }
}

pub struct Analyzer {
    scopes: ScopeStack,
    structs: HashMap<String, StructDef>,
    functions: HashMap<String, FuncDef>,
    errors: Vec<SemanticError>,
    all_symbols: Vec<Symbol>,
    loop_depth: usize,
    current_fn_return: Option<FrType>,
    current_fn: Option<String>,
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
            current_fn: None,
            current_origin: "global".to_string(),
        }
    }

    fn err(&mut self, msg: impl Into<String>) {
        let ctx = match &self.current_fn {
            Some(f) => format!("inside function '{}'", f),
            None => "at global scope".to_string(),
        };
        self.errors.push(SemanticError::with_context(msg, ctx));
    }

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
            if s.name == name {
                s.value = value;
                break;
            }
        }
    }

    fn infer_expr_with_hint(&mut self, node: &ParseNode, hint: &FrType) -> (FrType, SymbolValue) {
        if let ParseNode::ArrayLit(elems) = node {
            if elems.is_empty() {
                let resolved = match hint {
                    FrType::List { elem } => FrType::List { elem: elem.clone() },
                    FrType::Array { elem, size } => FrType::Array {
                        elem: elem.clone(),
                        size: *size,
                    },
                    _ => return self.infer_expr(node),
                };
                return (resolved, SymbolValue::Array(vec![]));
            }
        }
        self.infer_expr(node)
    }

    fn infer_expr(&mut self, node: &ParseNode) -> (FrType, SymbolValue) {
        match node {
            ParseNode::IntLit(n) => (FrType::Int, SymbolValue::Int(*n)),
            ParseNode::FloatLit(f) => (FrType::Float, SymbolValue::Float(*f)),
            ParseNode::CharLit(c) => (FrType::Char, SymbolValue::Char(*c)),
            ParseNode::BoolLit(b) => (FrType::Boolean, SymbolValue::Boolean(*b)),
            ParseNode::Null => (FrType::Null, SymbolValue::Null),

            ParseNode::StringLit(s) => {
                let chars: Vec<SymbolValue> = s.chars().map(SymbolValue::Char).collect();
                let len = chars.len() as i64;
                (
                    FrType::Array {
                        elem: Box::new(FrType::Char),
                        size: len,
                    },
                    SymbolValue::Array(chars),
                )
            }

            ParseNode::ArrayLit(elems) => {
                if elems.is_empty() {
                    return (
                        FrType::Array {
                            elem: Box::new(FrType::Void),
                            size: 0,
                        },
                        SymbolValue::Array(vec![]),
                    );
                }
                let (first_ty, _) = self.infer_expr(&elems[0]);
                let mut vals = Vec::new();
                for e in elems {
                    let (ty, v) = self.infer_expr(e);
                    if ty != first_ty {
                        self.err(format!(
                            "array/list literal has mixed element types: first element is '{}' \
                             but a later element is '{}' — all elements must be the same type",
                            first_ty, ty
                        ));
                    }
                    vals.push(v);
                }
                let size = vals.len() as i64;

                (
                    FrType::Array {
                        elem: Box::new(first_ty),
                        size,
                    },
                    SymbolValue::Array(vals),
                )
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
                    self.err(format!(
                        "use of undeclared identifier '{}' — \
                         declare it with `:type {} = value;` before using it, \
                         or check for a typo in the name",
                        name, name
                    ));
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
                            self.err(format!(
                                "unary `-` (negation) requires an :int or :float operand, \
                                 but got '{}' — negate only works on numeric types",
                                ty
                            ));
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
                            self.err(format!(
                                "bitwise NOT `~` requires an :int operand, but got '{}' — \
                                 bitwise operations only work on :int",
                                ty
                            ));
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
                    let op_str = match op {
                        AddOp::Add => "+",
                        AddOp::Sub => "-",
                    };
                    self.err(format!(
                        "both sides of `{}` must be the same type, \
                         but the left side is '{}' and the right side is '{}' — \
                         use an explicit cast, e.g. `:float(x) {} y`",
                        op_str, lt, rt, op_str
                    ));
                    return (lt, SymbolValue::Unknown);
                }
                let v = match (op, &lv, &rv) {
                    (AddOp::Add, SymbolValue::Int(a), SymbolValue::Int(b)) => {
                        SymbolValue::Int(a.wrapping_add(*b))
                    }
                    (AddOp::Add, SymbolValue::Float(a), SymbolValue::Float(b)) => {
                        SymbolValue::Float(a + b)
                    }
                    (AddOp::Sub, SymbolValue::Int(a), SymbolValue::Int(b)) => {
                        SymbolValue::Int(a.wrapping_sub(*b))
                    }
                    (AddOp::Sub, SymbolValue::Float(a), SymbolValue::Float(b)) => {
                        SymbolValue::Float(a - b)
                    }
                    _ => SymbolValue::Unknown,
                };
                (lt, v)
            }

            ParseNode::Mul { left, op, right } => {
                let (lt, lv) = self.infer_expr(left);
                let (rt, rv) = self.infer_expr(right);
                if lt != rt {
                    let op_str = match op {
                        MulOp::Mul => "*",
                        MulOp::Div => "/",
                        MulOp::Mod => "%",
                    };
                    self.err(format!(
                        "both sides of `{}` must be the same type, \
                         but the left side is '{}' and the right side is '{}' — \
                         use an explicit cast, e.g. `:float(x) {} y`",
                        op_str, lt, rt, op_str
                    ));
                    return (lt, SymbolValue::Unknown);
                }
                let v = match (op, &lv, &rv) {
                    (MulOp::Mul, SymbolValue::Int(a), SymbolValue::Int(b)) => {
                        SymbolValue::Int(a.wrapping_mul(*b))
                    }
                    (MulOp::Mul, SymbolValue::Float(a), SymbolValue::Float(b)) => {
                        SymbolValue::Float(a * b)
                    }
                    (MulOp::Div, SymbolValue::Int(a), SymbolValue::Int(b)) if *b != 0 => {
                        SymbolValue::Int(a.wrapping_div(*b))
                    }
                    (MulOp::Div, SymbolValue::Float(a), SymbolValue::Float(b)) => {
                        SymbolValue::Float(a / b)
                    }
                    (MulOp::Mod, SymbolValue::Int(a), SymbolValue::Int(b)) if *b != 0 => {
                        SymbolValue::Int(a.wrapping_rem(*b))
                    }
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
                        "bitwise operators (`&`, `|`, `^`) require both operands to be :int, \
                         but got '{}' and '{}' — \
                         cast to :int first if needed, e.g. `:int(x) & :int(y)`",
                        lt, rt
                    ));
                }
                let v = match (node, &lv, &rv) {
                    (ParseNode::BitAnd { .. }, SymbolValue::Int(a), SymbolValue::Int(b)) => {
                        SymbolValue::Int(a & b)
                    }
                    (ParseNode::BitOr { .. }, SymbolValue::Int(a), SymbolValue::Int(b)) => {
                        SymbolValue::Int(a | b)
                    }
                    (ParseNode::BitXor { .. }, SymbolValue::Int(a), SymbolValue::Int(b)) => {
                        SymbolValue::Int(a ^ b)
                    }
                    _ => SymbolValue::Unknown,
                };
                (FrType::Int, v)
            }

            ParseNode::Cmp { left, op: _, right } => {
                let (lt, _) = self.infer_expr(left);
                let (rt, _) = self.infer_expr(right);
                if lt != FrType::Null && rt != FrType::Null && lt != rt {
                    self.err(format!(
                        "comparison requires both sides to be the same type, \
                         but the left side is '{}' and the right side is '{}' — \
                         add an explicit cast so both sides match",
                        lt, rt
                    ));
                }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            ParseNode::LogAnd { left, right } | ParseNode::LogOr { left, right } => {
                let (lt, _) = self.infer_expr(left);
                let (rt, _) = self.infer_expr(right);
                if lt != FrType::Boolean {
                    self.err(format!(
                        "`!and`/`!or` requires :boolean operands, but the left side is '{}' — \
                         use a comparison to produce a :boolean, e.g. `(x > 0) !and (y > 0)`",
                        lt
                    ));
                }
                if rt != FrType::Boolean {
                    self.err(format!(
                        "`!and`/`!or` requires :boolean operands, but the right side is '{}' — \
                         use a comparison to produce a :boolean, e.g. `(x > 0) !and (y > 0)`",
                        rt
                    ));
                }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            ParseNode::LogNot { operand } => {
                let (ty, _) = self.infer_expr(operand);
                if ty != FrType::Boolean {
                    self.err(format!(
                        "`!not` requires a :boolean operand, but got '{}' — \
                         use a comparison to produce a :boolean first, e.g. `!not (x == 0)`",
                        ty
                    ));
                }
                (FrType::Boolean, SymbolValue::Unknown)
            }

            _ => (FrType::Void, SymbolValue::Unknown),
        }
    }

    fn infer_access_chain(&mut self, base: &str, steps: &[AccessStep]) -> (FrType, SymbolValue) {
        if steps.len() == 1 {
            if let AccessStep::Call(args) = &steps[0] {
                if BUILTIN_FUNCTIONS.contains(&base) || self.functions.contains_key(base) {
                    let ret = self.check_call(base, args);
                    return (ret, SymbolValue::Unknown);
                }

                if self.scopes.lookup(base).is_some() {
                    self.err(format!(
                        "'{}' is a variable, not a function — \
                         it cannot be called with `()`",
                        base
                    ));
                    return (FrType::Void, SymbolValue::Unknown);
                }
            }
        }

        if let Some(AccessStep::Field(field)) = steps.first() {
            if steps.len() >= 2 {
                if let AccessStep::Call(args) = &steps[1] {
                    let mod_fn = format!("{}::{}", base, field);
                    if self.functions.contains_key(field.as_str())
                        || self.functions.contains_key(mod_fn.as_str())
                        || BUILTIN_FUNCTIONS.contains(&field.as_str())
                    {
                        let fn_name = if self.functions.contains_key(field.as_str()) {
                            field.clone()
                        } else if self.functions.contains_key(mod_fn.as_str()) {
                            mod_fn
                        } else {
                            field.clone()
                        };
                        let ret = self.check_call(&fn_name, args);

                        let mut cur_ty = ret;
                        let mut cur_val = SymbolValue::Unknown;
                        for step in &steps[2..] {
                            let (t, v) = self.walk_step(cur_ty, cur_val, step, base);
                            cur_ty = t;
                            cur_val = v;
                        }
                        return (cur_ty, cur_val);
                    }
                }
            }

            let qualified = format!("{}::{}", base, field);
            if self.scopes.lookup(&qualified).is_some() {
                let sym = self.scopes.lookup(&qualified).unwrap();
                let mut cur_ty = sym.ty.clone();
                let mut cur_val = sym.value.clone();
                for step in &steps[1..] {
                    let (t, v) = self.walk_step(cur_ty, cur_val, step, &qualified);
                    cur_ty = t;
                    cur_val = v;
                }
                return (cur_ty, cur_val);
            }
        }

        let (mut cur_ty, mut cur_val) = match self.scopes.lookup(base) {
            Some(s) => (s.ty.clone(), s.value.clone()),
            None => {
                self.err(format!(
                    "use of undeclared identifier '{}' — \
                     declare it with `:type {} = value;` before using it, \
                     or check for a typo in the name",
                    base, base
                ));
                return (FrType::Void, SymbolValue::Unknown);
            }
        };

        for step in steps {
            let (t, v) = self.walk_step(cur_ty, cur_val, step, base);
            cur_ty = t;
            cur_val = v;
        }
        (cur_ty, cur_val)
    }

    fn walk_step(
        &mut self,
        cur_ty: FrType,
        cur_val: SymbolValue,
        step: &AccessStep,
        base: &str,
    ) -> (FrType, SymbolValue) {
        match step {
            AccessStep::Field(field) => {
                if let FrType::Struct { name: sname } = &cur_ty {
                    let sname = sname.clone();
                    if let Some(sdef) = self.structs.get(&sname).cloned() {
                        if let Some((_, fty)) = sdef.fields.iter().find(|(n, _)| n == field) {
                            let fval = if let SymbolValue::StructInstance { fields } = &cur_val {
                                fields.get(field).cloned().unwrap_or(SymbolValue::Unknown)
                            } else {
                                SymbolValue::Unknown
                            };
                            return (fty.clone(), fval);
                        }
                        self.err(format!(
                            "struct '{}' has no field named '{}' — \
                             check the struct definition for the correct field names",
                            sname, field
                        ));
                    } else {
                        self.err(format!(
                            "use of undefined struct type '{}' — \
                             declare it with `:struct<{}> {{ ... }};` before using it",
                            sname, sname
                        ));
                    }
                } else {
                    self.err(format!(
                        "cannot use `::{}` field access on a value of type '{}', \
                         which is not a struct — `::` is only valid on :struct types",
                        field, cur_ty
                    ));
                }
                (FrType::Void, SymbolValue::Unknown)
            }
            AccessStep::Index(idx_node) => {
                let (idx_ty, _) = self.infer_expr(idx_node);
                if idx_ty != FrType::Int {
                    self.err(format!(
                        "array/list index must be :int, but got '{}' — \
                         use :int(expr) to cast the index if needed",
                        idx_ty
                    ));
                }
                match cur_ty {
                    FrType::Array { elem, .. } | FrType::List { elem } => {
                        (*elem, SymbolValue::Unknown)
                    }
                    _ => {
                        self.err(format!(
                            "cannot index into a value of type '{}' — \
                             indexing with `[i]` is only valid on :array and :list",
                            cur_ty
                        ));
                        (FrType::Void, SymbolValue::Unknown)
                    }
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
            "print" | "input" => {
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
                                "append: cannot add a '{}' value to a list<{}> — \
                                 the value type must match the list's element type",
                                elem_ty, elem
                            ));
                        }
                    } else {
                        self.err(format!(
                            "append: first argument must be a :list, but got '{}' — \
                             append only works on :list, not :array (arrays are fixed-size)",
                            list_ty
                        ));
                    }
                } else {
                    self.err(format!(
                        "append expects exactly 2 arguments (list, value), but got {} — \
                         usage: `append(my_list, value);`",
                        args.len()
                    ));
                }
                return FrType::Void;
            }
            "pop" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    return match ty {
                        FrType::List { elem } => *elem,
                        _ => {
                            self.err(format!(
                                "pop: argument must be a :list, but got '{}' — \
                                 pop only works on :list (use indexing for :array elements)",
                                ty
                            ));
                            FrType::Void
                        }
                    };
                }
                self.err(format!(
                    "pop expects exactly 1 argument (a :list), but got {} — \
                     usage: `:type val = pop(my_list);`",
                    args.len()
                ));
                return FrType::Void;
            }
            "sqrt" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    if ty != FrType::Float {
                        self.err(format!(
                            "sqrt expects a :float argument, but got '{}' — \
                             cast with :float(x) if needed",
                            ty
                        ));
                    }
                } else {
                    self.err(format!(
                        "sqrt expects exactly 1 :float argument, but got {}",
                        args.len()
                    ));
                }
                return FrType::Float;
            }
            "abs" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    if ty != FrType::Int && ty != FrType::Float {
                        self.err(format!(
                            "abs expects an :int or :float argument, but got '{}' — \
                             abs only works on numeric types",
                            ty
                        ));
                    }
                    return ty;
                } else {
                    self.err(format!(
                        "abs expects exactly 1 argument (:int or :float), but got {}",
                        args.len()
                    ));
                }
                return FrType::Int;
            }
            "pow" => {
                if args.len() == 2 {
                    for a in args {
                        let (ty, _) = self.infer_expr(a);
                        if ty != FrType::Float {
                            self.err(format!(
                                "pow expects :float arguments, but got '{}' — \
                                 cast with :float(x) if needed",
                                ty
                            ));
                        }
                    }
                } else {
                    self.err(format!(
                        "pow expects exactly 2 :float arguments (base, exponent), but got {}",
                        args.len()
                    ));
                }
                return FrType::Float;
            }
            "len" => {
                if args.len() == 1 {
                    let (ty, _) = self.infer_expr(&args[0]);
                    if !matches!(ty, FrType::List { .. } | FrType::Array { .. }) {
                        self.err(format!(
                            "len expects an :array or :list argument, but got '{}' — \
                             len only works on iterable types",
                            ty
                        ));
                    }
                } else {
                    self.err(format!(
                        "len expects exactly 1 argument (an :array or :list), but got {}",
                        args.len()
                    ));
                }
                return FrType::Int;
            }
            "find" => {
                for a in args {
                    self.infer_expr(a);
                }
                return FrType::Int;
            }
            "starts" | "ends" => {
                for a in args {
                    self.infer_expr(a);
                }
                return FrType::Boolean;
            }
            "insert" | "delete" => {
                for a in args {
                    self.infer_expr(a);
                }
                return FrType::Void;
            }
            _ => {}
        }

        if let Some(fdef) = self.functions.get(name).cloned() {
            let params = fdef.params.clone();
            let ret = fdef.return_type.clone();
            if args.len() != params.len() {
                self.err(format!(
                    "function '{}' expects {} argument(s) but was called with {} — \
                     check the function signature and make sure each argument is passed separately",
                    name,
                    params.len(),
                    args.len()
                ));
                return ret;
            }
            for (i, (a, (pname, pty))) in args.iter().zip(params.iter()).enumerate() {
                let (aty, _) = self.infer_expr(a);
                if aty != *pty && aty != FrType::Null {
                    self.err(format!(
                        "function '{}': argument {} ('{}') expects type '{}' but got '{}' — \
                         add an explicit cast if a conversion is intended",
                        name,
                        i + 1,
                        pname,
                        pty,
                        aty
                    ));
                }
            }
            return ret;
        }

        if let Some(sym) = self.scopes.lookup(name) {
            return sym.ty.clone();
        }

        self.err(format!(
            "call to undeclared function '{}' — \
             define it with `!func {}(...) -> :type {{ ... }}` before calling it, \
             or check for a typo in the name",
            name, name
        ));
        FrType::Void
    }

    fn apply_cast(&mut self, dest: &FrType, src: &FrType, val: &SymbolValue) -> SymbolValue {
        let legal = src == dest
            || matches!(
                (src, dest),
                (FrType::Int, FrType::Float)
                    | (FrType::Float, FrType::Int)
                    | (FrType::Int, FrType::Char)
                    | (FrType::Char, FrType::Int)
                    | (FrType::Int, FrType::Boolean)
                    | (FrType::Boolean, FrType::Int)
                    | (FrType::Float, FrType::Boolean)
                    | (FrType::Boolean, FrType::Float)
            );
        if !legal {
            self.err(format!(
                "cannot cast '{}' to '{}' — \
                 legal casts are: int↔float, int↔char, int↔boolean, float↔boolean; \
                 for a multi-step cast (e.g. float→char) go via :int first: `:char(:int(x))`",
                src, dest
            ));
            return SymbolValue::Unknown;
        }
        match (dest, val) {
            (FrType::Int, SymbolValue::Int(n)) => SymbolValue::Int(*n),
            (FrType::Float, SymbolValue::Float(f)) => SymbolValue::Float(*f),
            (FrType::Char, SymbolValue::Char(c)) => SymbolValue::Char(*c),
            (FrType::Boolean, SymbolValue::Boolean(b)) => SymbolValue::Boolean(*b),

            (FrType::Float, SymbolValue::Int(n)) => SymbolValue::Float(*n as f64),

            (FrType::Int, SymbolValue::Float(f)) => SymbolValue::Int(*f as i64),

            (FrType::Char, SymbolValue::Int(n)) => {
                SymbolValue::Char(char::from_u32(*n as u32).unwrap_or('\0'))
            }

            (FrType::Int, SymbolValue::Char(c)) => SymbolValue::Int(*c as i64),

            (FrType::Boolean, SymbolValue::Int(n)) => SymbolValue::Boolean(*n != 0),

            (FrType::Int, SymbolValue::Boolean(b)) => SymbolValue::Int(*b as i64),

            (FrType::Boolean, SymbolValue::Float(f)) => SymbolValue::Boolean(*f != 0.0),

            (FrType::Float, SymbolValue::Boolean(b)) => {
                SymbolValue::Float(if *b { 1.0 } else { 0.0 })
            }

            _ => SymbolValue::Unknown,
        }
    }

    fn check_stmt(&mut self, node: &ParseNode) {
        match node {
            ParseNode::Decl {
                data_type,
                name,
                init,
            } => {
                let declared_ty = self.node_to_frtype(data_type);
                let (init_ty, init_val) = if let Some(expr) = init {
                    self.infer_expr_with_hint(expr, &declared_ty)
                } else {
                    (declared_ty.clone(), SymbolValue::Unknown)
                };
                let value = if let Some(_) = init {
                    match (&declared_ty, &init_val) {
                        (FrType::Struct { .. }, SymbolValue::StructInstance { .. }) => init_val,
                        _ => {
                            let compatible = types_compatible(&declared_ty, &init_ty);
                            if !compatible
                                && !matches!(init_ty, FrType::Null)
                                && init_ty != FrType::Void
                            {
                                let hint = match (&declared_ty, &init_ty) {

                                    (FrType::Int | FrType::Float | FrType::Char | FrType::Boolean,
                                     FrType::Int | FrType::Float | FrType::Char | FrType::Boolean) => {
                                        format!(
                                            "the types must match exactly; this language has no implicit casting — \
                                             use an explicit cast, e.g. `:{}({})` if a conversion is intended",
                                            declared_ty, name
                                        )
                                    }

                                    (FrType::Array { size: ds, .. }, FrType::Array { size: as_, .. }) => {
                                        format!(
                                            "the array literal has {} element{} but the declared type requires {} — \
                                             fix the literal to have exactly {} element{}",
                                            as_, if *as_ == 1 { "" } else { "s" },
                                            ds,
                                            ds, if *ds == 1 { "" } else { "s" }
                                        )
                                    }

                                    _ => {
                                        "the types must match exactly; this language has no implicit casting".to_string()
                                    }
                                };
                                self.err(format!(
                                    "cannot initialise '{}' (declared as '{}') with a value of type '{}' — {}",
                                    name, declared_ty, init_ty, hint
                                ));
                            }
                            init_val
                        }
                    }
                } else {
                    SymbolValue::Unknown
                };

                self.declare_sym(Symbol {
                    name: name.clone(),
                    ty: declared_ty,
                    value,
                    scope_depth: self.scopes.depth(),
                    origin: self.current_origin.clone(),
                });
            }

            ParseNode::StructDecl {
                struct_name,
                var_name,
                init,
            } => {
                if !self.structs.contains_key(struct_name) {
                    self.err(format!(
                        "use of undefined struct type '{}' — \
                         define it with `:struct<{}> {{ ... }};` before declaring a variable of that type",
                        struct_name, struct_name
                    ));
                }
                let value = if let Some(expr) = init {
                    let (_, v) = self.infer_expr(expr);
                    v
                } else {
                    SymbolValue::Unknown
                };
                self.declare_sym(Symbol {
                    name: var_name.clone(),
                    ty: FrType::Struct {
                        name: struct_name.clone(),
                    },
                    value,
                    scope_depth: self.scopes.depth(),
                    origin: self.current_origin.clone(),
                });
            }

            ParseNode::StructDef { name, fields } => {
                let mut fl = Vec::new();
                for f in fields {
                    if let ParseNode::Field {
                        data_type,
                        name: fname,
                    } = f
                    {
                        fl.push((fname.clone(), self.node_to_frtype(data_type)));
                    }
                }
                self.structs.insert(
                    name.clone(),
                    StructDef {
                        name: name.clone(),
                        fields: fl,
                    },
                );
            }

            ParseNode::Assign { lvalue, op, expr } => {
                let (lty, _) = self.infer_expr(lvalue);
                let (rty, rval) = self.infer_expr_with_hint(expr, &lty);
                if !types_compatible(&lty, &rty)
                    && !matches!(rty, FrType::Null)
                    && !matches!(lty, FrType::Null | FrType::Void)
                {
                    self.err(format!(
                        "cannot assign a '{}' value to a variable of type '{}' — \
                         types must match exactly; use an explicit cast if a conversion is intended",
                        rty, lty
                    ));
                }
                if matches!(op, AssignOp::AmpEq | AssignOp::PipeEq | AssignOp::CaretEq)
                    && lty != FrType::Int
                {
                    self.err(format!(
                        "bitwise compound assignment (`&=`, `|=`, `^=`) requires an :int variable, \
                         but '{}' is of type '{}' — bitwise operations only work on :int",
                        if let ParseNode::AccessChain { base, .. } = lvalue.as_ref() { base.as_str() } else { "lhs" },
                        lty
                    ));
                }
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
                if let Some(expected) = self.current_fn_return.clone() {
                    if ty != expected && !matches!(ty, FrType::Null) {
                        let fn_name = self.current_fn.clone().unwrap_or_else(|| "?".to_string());
                        self.err(format!(
                            "!return expression has type '{}' but function '{}' is declared to return '{}' — \
                             either change the return type annotation or cast the return value",
                            ty, fn_name, expected
                        ));
                    }
                }
            }

            ParseNode::Exit(e) => {
                let (ty, _) = self.infer_expr(e);
                if ty != FrType::Int && !matches!(ty, FrType::Null) {
                    self.err(format!(
                        "!exit requires an :int exit code, but got '{}' — \
                         use an integer literal, e.g. `!exit 0;` for success or `!exit 1;` for failure",
                        ty
                    ));
                }
            }

            ParseNode::Break | ParseNode::Continue => {
                if self.loop_depth == 0 {
                    self.err(
                        "!break and !continue can only be used inside a loop — \
                         they are not valid at the top level or inside a function outside a loop",
                    );
                }
            }

            ParseNode::If {
                condition,
                then_block,
                else_block,
            } => {
                let (cty, _) = self.infer_expr(condition);
                if cty != FrType::Boolean {
                    self.err(format!(
                        "!if condition must be :boolean, but got '{}' — \
                         use a comparison operator to produce a :boolean, \
                         e.g. `!if (x > 0)` instead of `!if (x)`",
                        cty
                    ));
                }
                let saved = self.current_origin.clone();
                self.scopes.push();
                self.current_origin = "local".to_string();
                for s in then_block {
                    self.check_stmt(s);
                }
                self.scopes.pop();
                self.current_origin = saved.clone();
                if let Some(eb) = else_block {
                    self.scopes.push();
                    self.current_origin = "local".to_string();
                    for s in eb {
                        self.check_stmt(s);
                    }
                    self.scopes.pop();
                    self.current_origin = saved;
                }
            }

            ParseNode::While { condition, body } => {
                let (cty, _) = self.infer_expr(condition);
                if cty != FrType::Boolean {
                    self.err(format!(
                        "!while condition must be :boolean, but got '{}' — \
                         use a comparison operator to produce a :boolean, \
                         e.g. `!while (x > 0)` instead of `!while (x)`",
                        cty
                    ));
                }
                self.loop_depth += 1;
                let saved = self.current_origin.clone();
                self.scopes.push();
                self.current_origin = "local".to_string();
                for s in body {
                    self.check_stmt(s);
                }
                self.scopes.pop();
                self.current_origin = saved;
                self.loop_depth -= 1;
            }

            ParseNode::For {
                var_type,
                var_name,
                start,
                stop,
                step,
                body,
            } => {
                let declared_ty = self.node_to_frtype(var_type);
                let (sty, _) = self.infer_expr(start);
                let (ety, _) = self.infer_expr(stop);
                let (step_ty, _) = self.infer_expr(step);

                if sty != declared_ty && sty != FrType::Void {
                    self.err(format!(
                        "!for loop variable '{}' is declared as '{}' but the start expression \
                         has type '{}' — the start value must match the loop variable's type",
                        var_name, declared_ty, sty
                    ));
                }
                if ety != declared_ty && ety != FrType::Void {
                    self.err(format!(
                        "!for loop variable '{}' is declared as '{}' but the stop expression \
                         has type '{}' — the stop value must match the loop variable's type",
                        var_name, declared_ty, ety
                    ));
                }
                if step_ty != declared_ty && step_ty != FrType::Void {
                    self.err(format!(
                        "!for loop variable '{}' is declared as '{}' but the step expression \
                         has type '{}' — the step value must match the loop variable's type",
                        var_name, declared_ty, step_ty
                    ));
                }
                self.loop_depth += 1;
                let saved = self.current_origin.clone();
                self.scopes.push();
                self.current_origin = "loop".to_string();
                self.declare_sym(Symbol {
                    name: var_name.clone(),
                    ty: declared_ty,
                    value: SymbolValue::Unknown,
                    scope_depth: self.scopes.depth(),
                    origin: "loop".to_string(),
                });
                self.current_origin = "local".to_string();
                for s in body {
                    self.check_stmt(s);
                }
                self.scopes.pop();
                self.current_origin = saved;
                self.loop_depth -= 1;
            }

            ParseNode::FuncDef {
                name,
                params,
                return_type,
                body,
            } => {
                let ret_ty = self.node_to_frtype(return_type);
                let mut param_list: Vec<(String, FrType)> = Vec::new();
                for p in params {
                    if let ParseNode::Param {
                        data_type,
                        name: pname,
                    } = p
                    {
                        param_list.push((pname.clone(), self.node_to_frtype(data_type)));
                    }
                }

                let is_inside_module = self.current_origin.starts_with("module:");
                if BUILTIN_FUNCTIONS.contains(&name.as_str()) && !is_inside_module {
                    self.err(format!(
                        "cannot define function '{}': that name is a built-in function — \
                         choose a different name to avoid shadowing the built-in",
                        name
                    ));
                } else if self.functions.contains_key(name) {
                    self.err(format!(
                        "function '{}' is already defined — \
                         each function name must be unique; rename one of the definitions",
                        name
                    ));
                } else {
                    self.functions.insert(
                        name.clone(),
                        FuncDef {
                            name: name.clone(),
                            params: param_list.clone(),
                            return_type: ret_ty.clone(),
                        },
                    );

                    self.declare_sym(Symbol {
                        name: name.clone(),
                        ty: ret_ty.clone(),
                        value: SymbolValue::Unknown,
                        scope_depth: self.scopes.depth(),
                        origin: format!("func:{}", name),
                    });
                }

                let saved_ret = self.current_fn_return.take();
                let saved_fn = self.current_fn.take();
                let saved_origin = self.current_origin.clone();
                self.current_fn_return = Some(ret_ty);
                self.current_fn = Some(name.clone());
                self.scopes.push();
                self.current_origin = format!("fn:{}", name);
                for (pname, pty) in &param_list {
                    self.declare_sym(Symbol {
                        name: pname.clone(),
                        ty: pty.clone(),
                        value: SymbolValue::Unknown,
                        scope_depth: self.scopes.depth(),
                        origin: format!("param:{}", name),
                    });
                }
                self.current_origin = format!("fn:{}", name);
                for s in body {
                    self.check_stmt(s);
                }
                self.scopes.pop();
                self.current_origin = saved_origin;
                self.current_fn = saved_fn;
                self.current_fn_return = saved_ret;
            }

            ParseNode::Module { name, items } => {
                let saved = self.current_origin.clone();
                self.current_origin = format!("module:{}", name);
                self.scopes.push();
                for item in items {
                    self.check_stmt(item);
                }
                let frame = self.scopes.pop();

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

                let bare_names: Vec<String> = frame.keys().cloned().collect();
                for bare in bare_names {
                    if let Some(fd) = self.functions.remove(&bare) {
                        let qualified_fn = format!("{}::{}", name, bare);
                        let qualified_fd = FuncDef {
                            name: qualified_fn.clone(),
                            params: fd.params,
                            return_type: fd.return_type,
                        };
                        self.functions.insert(qualified_fn, qualified_fd);
                    }
                }

                self.current_origin = saved;
            }

            ParseNode::Program(items) => {
                for item in items {
                    self.check_stmt(item);
                }
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
