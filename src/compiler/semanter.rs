#![allow(unused)]
#![allow(dead_code)]

use crate::compiler::parser::{AccessStep, AddOp, AssignOp, CmpOp, MulOp, ParseNode, UnOp};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SemType {
    Int,
    Float,
    Char,
    Boolean,
    Array { elem: Box<SemType>, size: i64 },
    List { elem: Box<SemType> },
    Struct { name: String },
    Void,
    Unknown,
}

impl std::fmt::Display for SemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SemType::Int => write!(f, "int"),
            SemType::Float => write!(f, "float"),
            SemType::Char => write!(f, "char"),
            SemType::Boolean => write!(f, "boolean"),
            SemType::Array { elem, size } => write!(f, "array<{},{}>", elem, size),
            SemType::List { elem } => write!(f, "list<{}>", elem),
            SemType::Struct { name } => write!(f, "struct<{}>", name),
            SemType::Void => write!(f, "void"),
            SemType::Unknown => write!(f, "unknown"),
        }
    }
}

pub fn sem_type_from_parse_node(node: &ParseNode) -> SemType {
    match node {
        ParseNode::TypeInt => SemType::Int,
        ParseNode::TypeFloat => SemType::Float,
        ParseNode::TypeChar => SemType::Char,
        ParseNode::TypeBoolean => SemType::Boolean,
        ParseNode::TypeVoid => SemType::Void,
        ParseNode::TypeArray { elem, size } => SemType::Array {
            elem: Box::new(sem_type_from_parse_node(elem.as_ref())),
            size: *size,
        },
        ParseNode::TypeList { elem } => SemType::List {
            elem: Box::new(sem_type_from_parse_node(elem.as_ref())),
        },
        ParseNode::TypeStruct { name } => SemType::Struct { name: name.clone() },
        _ => SemType::Unknown,
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Char(char),
    Boolean(bool),
    Null,
    Composite,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub sem_type: SemType,
    pub value: Option<Value>,
}

pub struct ScopeStack {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl ScopeStack {
    fn new() -> Self {
        ScopeStack {
            scopes: vec![HashMap::new()],
        }
    }

    fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn declare(&mut self, name: &str, sym: Symbol) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), sym);
        }
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }

    fn update_value(&mut self, name: &str, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.get_mut(name) {
                sym.value = Some(value);
                return;
            }
        }
    }

    pub fn dump(&self) {
        println!("\n\x1b[1;34m=== Symbol Table ===\x1b[0m");
        for (depth, scope) in self.scopes.iter().enumerate() {
            if scope.is_empty() {
                continue;
            }
            println!("  \x1b[2mScope depth {}\x1b[0m", depth);
            let mut names: Vec<&String> = scope.keys().collect();
            names.sort();
            for name in names {
                let sym = &scope[name];
                let val_str = match &sym.value {
                    Some(Value::Int(n)) => format!("{}", n),
                    Some(Value::Float(f)) => format!("{}", f),
                    Some(Value::Char(c)) => format!("{:?}", c),
                    Some(Value::Boolean(b)) => format!("{}", b),
                    Some(Value::Null) => "null".to_string(),
                    Some(Value::Composite) => "<composite>".to_string(),
                    None => "<unset>".to_string(),
                };
                println!(
                    "    \x1b[36m{:<20}\x1b[0m : \x1b[33m{:<20}\x1b[0m = {}",
                    name,
                    sym.sem_type.to_string(),
                    val_str
                );
            }
        }
        println!("\x1b[1;34m====================\x1b[0m\n");
    }
}

#[derive(Debug, Clone)]
pub struct FuncSig {
    pub param_types: Vec<SemType>,
    pub return_type: SemType,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub sem_type: SemType,
}

pub struct Semanter {
    pub scope: ScopeStack,
    pub funcs: HashMap<String, FuncSig>,
    pub structs: HashMap<String, Vec<StructField>>,
    pub errors: Vec<String>,
    current_return_type: Option<SemType>,
}

impl Semanter {
    pub fn new() -> Self {
        let mut s = Semanter {
            scope: ScopeStack::new(),
            funcs: HashMap::new(),
            structs: HashMap::new(),
            errors: Vec::new(),
            current_return_type: None,
        };
        s.register_builtins();
        s
    }

    fn register_builtins(&mut self) {
        self.funcs.insert(
            "print".to_string(),
            FuncSig {
                param_types: vec![],
                return_type: SemType::Void,
            },
        );

        self.funcs.insert(
            "append".to_string(),
            FuncSig {
                param_types: vec![SemType::Unknown, SemType::Unknown],
                return_type: SemType::Void,
            },
        );

        self.funcs.insert(
            "pop".to_string(),
            FuncSig {
                param_types: vec![SemType::Unknown],
                return_type: SemType::Unknown,
            },
        );

        self.funcs.insert(
            "insert".to_string(),
            FuncSig {
                param_types: vec![SemType::Unknown, SemType::Unknown, SemType::Int],
                return_type: SemType::Void,
            },
        );

        self.funcs.insert(
            "find".to_string(),
            FuncSig {
                param_types: vec![SemType::Unknown, SemType::Unknown],
                return_type: SemType::Int,
            },
        );

        self.funcs.insert(
            "delete".to_string(),
            FuncSig {
                param_types: vec![SemType::Unknown, SemType::Int],
                return_type: SemType::Void,
            },
        );

        self.funcs.insert(
            "starts".to_string(),
            FuncSig {
                param_types: vec![SemType::Unknown, SemType::Unknown],
                return_type: SemType::Boolean,
            },
        );

        self.funcs.insert(
            "ends".to_string(),
            FuncSig {
                param_types: vec![SemType::Unknown, SemType::Unknown],
                return_type: SemType::Boolean,
            },
        );
    }

    fn err(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
        eprintln!("\x1b[1;31mSemantic error:\x1b[0m {}", msg);
    }

    pub fn check(&mut self, root: &ParseNode) {
        self.hoist(root);
        self.check_node(root);
        self.scope.dump();
        if self.errors.is_empty() {
            println!("\x1b[1;32mSemantic analysis passed with no errors.\x1b[0m");
        } else {
            eprintln!(
                "\x1b[1;31mSemantic analysis found {} error(s).\x1b[0m",
                self.errors.len()
            );
        }
    }

    fn hoist(&mut self, node: &ParseNode) {
        match node {
            ParseNode::Program(items) | ParseNode::Module { items, .. } => {
                for item in items {
                    self.hoist(item);
                }
            }
            ParseNode::StructDef { name, fields } => {
                let sem_fields: Vec<StructField> = fields
                    .iter()
                    .filter_map(|f| {
                        if let ParseNode::Field {
                            data_type,
                            name: fname,
                        } = f
                        {
                            Some(StructField {
                                name: fname.clone(),
                                sem_type: sem_type_from_parse_node(data_type.as_ref()),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                self.structs.insert(name.clone(), sem_fields);
            }
            ParseNode::FuncDef {
                name,
                params,
                return_type,
                ..
            } => {
                let ret = sem_type_from_parse_node(return_type.as_ref());
                let param_types: Vec<SemType> = params
                    .iter()
                    .map(|p| match p {
                        ParseNode::Param { data_type, .. } => {
                            sem_type_from_parse_node(data_type.as_ref())
                        }
                        _ => SemType::Unknown,
                    })
                    .collect();
                self.funcs.insert(
                    name.clone(),
                    FuncSig {
                        param_types,
                        return_type: ret,
                    },
                );
            }
            _ => {}
        }
    }

    fn check_node(&mut self, node: &ParseNode) {
        match node {
            ParseNode::Program(items) => {
                for item in items {
                    self.check_node(item);
                }
            }

            ParseNode::Module { items, .. } => {
                self.scope.push();
                for item in items {
                    self.check_node(item);
                }
                self.scope.pop();
            }

            ParseNode::FuncDef {
                name,
                params,
                return_type,
                body,
            } => {
                let ret = sem_type_from_parse_node(return_type.as_ref());
                self.scope.push();
                let prev_ret = self.current_return_type.replace(ret);

                for param in params {
                    if let ParseNode::Param {
                        data_type,
                        name: pname,
                    } = param
                    {
                        self.scope.declare(
                            pname,
                            Symbol {
                                sem_type: sem_type_from_parse_node(data_type.as_ref()),
                                value: None,
                            },
                        );
                    }
                }
                for stmt in body {
                    self.check_node(stmt);
                }

                self.current_return_type = prev_ret;
                self.scope.pop();
            }

            ParseNode::StructDef { .. } => {}

            ParseNode::StructDecl {
                struct_name,
                var_name,
                init,
            } => {
                let decl_type = SemType::Struct {
                    name: struct_name.clone(),
                };
                if let Some(init_expr) = init {
                    let init_type = self.infer_type_with_hint(init_expr, &decl_type);
                    if !self.types_compatible(&decl_type, &init_type) {
                        self.err(&format!(
                            "Type mismatch in declaration of '{}': declared as '{}', initialised with '{}'",
                            var_name, decl_type, init_type
                        ));
                    }
                }
                self.scope.declare(
                    var_name,
                    Symbol {
                        sem_type: decl_type,
                        value: Some(Value::Composite),
                    },
                );
            }

            ParseNode::Decl {
                data_type,
                name,
                init,
            } => {
                let decl_type = sem_type_from_parse_node(data_type.as_ref());
                let init_val: Option<Value>;

                if let Some(init_expr) = init {
                    let init_type = self.infer_type_with_hint(init_expr, &decl_type);
                    if !self.types_compatible(&decl_type, &init_type) {
                        self.err(&format!(
                            "Type mismatch in declaration of '{}': declared as '{}', initialised with '{}'",
                            name, decl_type, init_type
                        ));
                    }
                    init_val = self.const_eval(init_expr);
                } else {
                    init_val = None;
                }

                self.scope.declare(
                    name,
                    Symbol {
                        sem_type: decl_type,
                        value: init_val,
                    },
                );
            }

            ParseNode::Assign { lvalue, op, expr } => {
                let lval_type = self.infer_chain_type(lvalue);
                let rval_type = self.infer_type(expr);

                match op {
                    AssignOp::Eq => {
                        if !self.types_compatible(&lval_type, &rval_type) {
                            self.err(&format!(
                                "Type mismatch in assignment: cannot assign '{}' to '{}'",
                                rval_type, lval_type
                            ));
                        }
                    }
                    AssignOp::PlusEq
                    | AssignOp::MinusEq
                    | AssignOp::StarEq
                    | AssignOp::SlashEq
                    | AssignOp::PercentEq => {
                        if !self.is_numeric(&lval_type) {
                            self.err(&format!(
                                "Operator '{:?}' requires numeric left-hand side, got '{}'",
                                op, lval_type
                            ));
                        }
                        if !self.types_compatible(&lval_type, &rval_type) {
                            self.err(&format!(
                                "Type mismatch in compound assignment: cannot apply '{:?}' between '{}' and '{}'",
                                op, lval_type, rval_type
                            ));
                        }
                    }
                    AssignOp::AmpEq | AssignOp::PipeEq | AssignOp::CaretEq => {
                        if lval_type != SemType::Int {
                            self.err(&format!(
                                "Bitwise assignment '{:?}' requires int, got '{}'",
                                op, lval_type
                            ));
                        }
                        if rval_type != SemType::Int {
                            self.err(&format!(
                                "Bitwise assignment '{:?}' requires int rhs, got '{}'",
                                op, rval_type
                            ));
                        }
                    }
                }

                if let ParseNode::AccessChain { base, steps } = lvalue.as_ref() {
                    if steps.is_empty() {
                        let new_val = match op {
                            AssignOp::Eq => self.const_eval(expr),
                            AssignOp::PlusEq
                            | AssignOp::MinusEq
                            | AssignOp::StarEq
                            | AssignOp::SlashEq
                            | AssignOp::PercentEq => {
                                let cur = self.scope.lookup(base).and_then(|s| s.value.clone());
                                let rhs = self.const_eval(expr);
                                match (cur, rhs) {
                                    (Some(Value::Int(a)), Some(Value::Int(b))) => match op {
                                        AssignOp::PlusEq => Some(Value::Int(a + b)),
                                        AssignOp::MinusEq => Some(Value::Int(a - b)),
                                        AssignOp::StarEq => Some(Value::Int(a * b)),
                                        AssignOp::SlashEq if b != 0 => Some(Value::Int(a / b)),
                                        AssignOp::PercentEq if b != 0 => Some(Value::Int(a % b)),
                                        _ => None,
                                    },
                                    (Some(Value::Float(a)), Some(Value::Float(b))) => match op {
                                        AssignOp::PlusEq => Some(Value::Float(a + b)),
                                        AssignOp::MinusEq => Some(Value::Float(a - b)),
                                        AssignOp::StarEq => Some(Value::Float(a * b)),
                                        AssignOp::SlashEq => Some(Value::Float(a / b)),
                                        _ => None,
                                    },
                                    _ => None,
                                }
                            }
                            AssignOp::AmpEq | AssignOp::PipeEq | AssignOp::CaretEq => {
                                let cur = self.scope.lookup(base).and_then(|s| s.value.clone());
                                let rhs = self.const_eval(expr);
                                match (cur, rhs) {
                                    (Some(Value::Int(a)), Some(Value::Int(b))) => match op {
                                        AssignOp::AmpEq => Some(Value::Int(a & b)),
                                        AssignOp::PipeEq => Some(Value::Int(a | b)),
                                        AssignOp::CaretEq => Some(Value::Int(a ^ b)),
                                        _ => None,
                                    },
                                    _ => None,
                                }
                            }
                        };
                        if let Some(val) = new_val {
                            self.scope.update_value(base, val);
                        }
                    }
                }
            }

            ParseNode::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_type = self.infer_type(condition);
                if cond_type != SemType::Boolean && cond_type != SemType::Unknown {
                    self.err(&format!(
                        "If condition must be boolean, got '{}'",
                        cond_type
                    ));
                }
                self.scope.push();
                for s in then_block {
                    self.check_node(s);
                }
                self.scope.pop();
                if let Some(eb) = else_block {
                    self.scope.push();
                    for s in eb {
                        self.check_node(s);
                    }
                    self.scope.pop();
                }
            }

            ParseNode::For {
                var_type,
                var_name,
                start,
                stop,
                step,
                body,
            } => {
                let vtype = sem_type_from_parse_node(var_type.as_ref());
                self.scope.push();
                self.scope.declare(
                    var_name,
                    Symbol {
                        sem_type: vtype.clone(),
                        value: None,
                    },
                );
                for expr in &[start, stop, step] {
                    let t = self.infer_type(expr);
                    if !self.types_compatible(&vtype, &t) {
                        self.err(&format!(
                            "For loop range expression type mismatch: loop var is '{}', expression is '{}'",
                            vtype, t
                        ));
                    }
                }
                for s in body {
                    self.check_node(s);
                }
                self.scope.pop();
            }

            ParseNode::While { condition, body } => {
                let cond_type = self.infer_type(condition);
                if cond_type != SemType::Boolean && cond_type != SemType::Unknown {
                    self.err(&format!(
                        "While condition must be boolean, got '{}'",
                        cond_type
                    ));
                }
                self.scope.push();
                for s in body {
                    self.check_node(s);
                }
                self.scope.pop();
            }

            ParseNode::Return(expr) => {
                let ret_type = self.infer_type(expr);
                if let Some(expected) = &self.current_return_type.clone() {
                    if !self.types_compatible(expected, &ret_type) {
                        self.err(&format!(
                            "Return type mismatch: function expects '{}', got '{}'",
                            expected, ret_type
                        ));
                    }
                }
            }

            ParseNode::Exit(expr) => {
                let t = self.infer_type(expr);
                if t != SemType::Int && t != SemType::Unknown {
                    self.err(&format!("Exit expects int, got '{}'", t));
                }
            }

            ParseNode::ExprStmt(e) => {
                self.infer_type(e);
            }

            ParseNode::Break | ParseNode::Continue => {}

            _ => {}
        }
    }

    pub fn infer_type(&mut self, node: &ParseNode) -> SemType {
        match node {
            ParseNode::IntLit(_) => SemType::Int,
            ParseNode::FloatLit(_) => SemType::Float,
            ParseNode::CharLit(_) => SemType::Char,
            ParseNode::BoolLit(_) => SemType::Boolean,
            ParseNode::Null => SemType::Void,
            ParseNode::StringLit(s) => SemType::Array {
                elem: Box::new(SemType::Char),
                size: s.chars().count() as i64,
            },

            ParseNode::Cast { target_type, .. } => sem_type_from_parse_node(target_type.as_ref()),

            ParseNode::Unary { op, operand } => {
                let t = self.infer_type(operand);
                match op {
                    UnOp::Neg => {
                        if !self.is_numeric(&t) {
                            self.err(&format!(
                                "Unary negation requires numeric type, got '{}'",
                                t
                            ));
                        }
                        t
                    }
                    UnOp::BitNot => {
                        if t != SemType::Int && t != SemType::Unknown {
                            self.err(&format!("Bitwise NOT requires int, got '{}'", t));
                        }
                        SemType::Int
                    }
                }
            }

            ParseNode::Add { left, right, .. } => {
                let lt = self.infer_type(left);
                let rt = self.infer_type(right);
                if !self.types_compatible(&lt, &rt) {
                    self.err(&format!("Type mismatch in addition: '{}' vs '{}'", lt, rt));
                    return SemType::Unknown;
                }
                if !self.is_numeric(&lt) {
                    self.err(&format!("Addition requires numeric operands, got '{}'", lt));
                }
                if lt == SemType::Unknown {
                    rt
                } else {
                    lt
                }
            }

            ParseNode::Mul { left, right, op } => {
                let lt = self.infer_type(left);
                let rt = self.infer_type(right);
                if !self.types_compatible(&lt, &rt) {
                    self.err(&format!(
                        "Type mismatch in mul/div/mod: '{}' vs '{}'",
                        lt, rt
                    ));
                    return SemType::Unknown;
                }
                if matches!(op, MulOp::Mod) && lt != SemType::Int && lt != SemType::Unknown {
                    self.err("Modulo operator requires int operands");
                }
                if lt == SemType::Unknown {
                    rt
                } else {
                    lt
                }
            }

            ParseNode::BitOr { left, right }
            | ParseNode::BitXor { left, right }
            | ParseNode::BitAnd { left, right } => {
                let lt = self.infer_type(left);
                let rt = self.infer_type(right);
                if lt != SemType::Int && lt != SemType::Unknown {
                    self.err(&format!("Bitwise operation requires int, got '{}'", lt));
                }
                if rt != SemType::Int && rt != SemType::Unknown {
                    self.err(&format!("Bitwise operation requires int, got '{}'", rt));
                }
                SemType::Int
            }

            ParseNode::Cmp { left, right, op } => {
                let lt = self.infer_type(left);
                let rt = self.infer_type(right);
                if !self.types_compatible(&lt, &rt) {
                    self.err(&format!(
                        "Type mismatch in comparison '{:?}': '{}' vs '{}'",
                        op, lt, rt
                    ));
                }
                SemType::Boolean
            }

            ParseNode::LogOr { left, right } | ParseNode::LogAnd { left, right } => {
                let lt = self.infer_type(left);
                let rt = self.infer_type(right);
                if lt != SemType::Boolean && lt != SemType::Unknown {
                    self.err(&format!("Logical operator requires boolean, got '{}'", lt));
                }
                if rt != SemType::Boolean && rt != SemType::Unknown {
                    self.err(&format!("Logical operator requires boolean, got '{}'", rt));
                }
                SemType::Boolean
            }

            ParseNode::LogNot { operand } => {
                let t = self.infer_type(operand);
                if t != SemType::Boolean && t != SemType::Unknown {
                    self.err(&format!("Logical NOT requires boolean, got '{}'", t));
                }
                SemType::Boolean
            }

            ParseNode::ArrayLit(elems) => {
                if elems.is_empty() {
                    return SemType::Array {
                        elem: Box::new(SemType::Unknown),
                        size: 0,
                    };
                }
                let first = self.infer_type(&elems[0]);
                for e in elems.iter().skip(1) {
                    let t = self.infer_type(e);
                    if !self.types_compatible(&first, &t) {
                        self.err(&format!(
                            "Array literal element type mismatch: expected '{}', got '{}'",
                            first, t
                        ));
                    }
                }
                SemType::Array {
                    elem: Box::new(first),
                    size: elems.len() as i64,
                }
            }

            ParseNode::StructLit(_) => SemType::Struct {
                name: "unknown".to_string(),
            },

            ParseNode::AccessChain { .. } => self.infer_chain_type(node),

            _ => SemType::Unknown,
        }
    }

    fn infer_chain_type(&mut self, node: &ParseNode) -> SemType {
        let (base, steps) = match node {
            ParseNode::AccessChain { base, steps } => (base, steps),
            _ => return SemType::Unknown,
        };

        let mut current_type = match self.scope.lookup(base) {
            Some(sym) => sym.sem_type.clone(),
            None => {
                if steps.is_empty() {
                    self.err(&format!("Undeclared identifier '{}'", base));
                }
                SemType::Unknown
            }
        };

        for step in steps {
            current_type = match step {
                AccessStep::Field(field_name) => {
                    self.resolve_field(&current_type, field_name, base)
                }

                AccessStep::Index(idx_expr) => {
                    let idx_type = self.infer_type(idx_expr);
                    if idx_type != SemType::Int && idx_type != SemType::Unknown {
                        self.err(&format!("Index expression must be int, got '{}'", idx_type));
                    }

                    match &current_type {
                        SemType::Array { elem, .. } => *elem.clone(),
                        SemType::List { elem } => *elem.clone(),
                        SemType::Unknown => SemType::Unknown,
                        other => {
                            self.err(&format!("Cannot index into type '{}'", other));
                            SemType::Unknown
                        }
                    }
                }

                AccessStep::Call(args) => {
                    let ret = self.check_call(base, args);
                    ret
                }
            };
        }

        current_type
    }

    fn resolve_field(&mut self, on_type: &SemType, field_name: &str, ctx: &str) -> SemType {
        match on_type {
            SemType::Struct { name } => {
                let struct_name = name.clone();

                if let Some(fields) = self.structs.get(&struct_name).cloned() {
                    match fields.iter().find(|f| f.name == field_name) {
                        Some(f) => f.sem_type.clone(),
                        None => {
                            self.err(&format!(
                                "Struct '{}' has no field '{}'",
                                struct_name, field_name
                            ));
                            SemType::Unknown
                        }
                    }
                } else {
                    SemType::Unknown
                }
            }
            SemType::Unknown => SemType::Unknown,
            other => {
                self.err(&format!(
                    "Cannot access field '{}' on non-struct type '{}' (from '{}')",
                    field_name, other, ctx
                ));
                SemType::Unknown
            }
        }
    }

    fn check_call(&mut self, name: &str, args: &[ParseNode]) -> SemType {
        let arg_types: Vec<SemType> = args.iter().map(|a| self.infer_type(a)).collect();

        if let Some(sig) = self.funcs.get(name).cloned() {
            if !sig.param_types.is_empty() && arg_types.len() != sig.param_types.len() {
                self.err(&format!(
                    "Function '{}' expects {} argument(s), got {}",
                    name,
                    sig.param_types.len(),
                    arg_types.len()
                ));
            } else if !sig.param_types.is_empty() {
                for (i, (at, expected)) in arg_types.iter().zip(sig.param_types.iter()).enumerate()
                {
                    if !self.types_compatible(expected, at) {
                        self.err(&format!(
                            "Argument {} of '{}': expected '{}', got '{}'",
                            i + 1,
                            name,
                            expected,
                            at
                        ));
                    }
                }
            }

            if name == "pop" {
                if let Some(at) = arg_types.first() {
                    if let SemType::List { elem } = at {
                        return *elem.clone();
                    }
                }
            }

            sig.return_type.clone()
        } else {
            SemType::Unknown
        }
    }

    fn infer_lvalue_type(&mut self, node: &ParseNode) -> SemType {
        self.infer_chain_type(node)
    }

    fn types_compatible(&self, a: &SemType, b: &SemType) -> bool {
        if a == b {
            return true;
        }
        if matches!(a, SemType::Unknown) || matches!(b, SemType::Unknown) {
            return true;
        }

        if *a == SemType::Float && *b == SemType::Int {
            return true;
        }
        if *a == SemType::Int && *b == SemType::Float {
            return true;
        }

        if let (SemType::Array { elem: ea, .. }, SemType::Array { elem: eb, .. }) = (a, b) {
            if **ea == SemType::Char && **eb == SemType::Char {
                return true;
            }
        }

        if matches!(b, SemType::Void) && matches!(a, SemType::Struct { .. }) {
            return true;
        }
        if matches!(a, SemType::Void) && matches!(b, SemType::Struct { .. }) {
            return true;
        }
        false
    }

    fn is_numeric(&self, t: &SemType) -> bool {
        matches!(t, SemType::Int | SemType::Float | SemType::Unknown)
    }

    fn infer_type_with_hint(&mut self, node: &ParseNode, hint: &SemType) -> SemType {
        match (node, hint) {
            (ParseNode::StructLit(_), SemType::Struct { .. }) => hint.clone(),

            (ParseNode::StringLit(s), SemType::Array { elem, size }) => {
                let len = s.chars().count() as i64;
                if **elem == SemType::Char && len > *size {
                    self.err(&format!(
                        "String literal of length {} does not fit in array<char,{}>",
                        len, size
                    ));
                }
                hint.clone()
            }

            (ParseNode::ArrayLit(elems), SemType::List { elem: hint_elem }) => {
                for e in elems {
                    let t = self.infer_type(e);
                    if !self.types_compatible(hint_elem, &t) {
                        self.err(&format!(
                            "List literal element type mismatch: expected '{}', got '{}'",
                            hint_elem, t
                        ));
                    }
                }
                SemType::List {
                    elem: hint_elem.clone(),
                }
            }

            (
                ParseNode::ArrayLit(elems),
                SemType::Array {
                    elem: hint_elem,
                    size,
                },
            ) => {
                for e in elems {
                    let t = self.infer_type(e);
                    if !self.types_compatible(hint_elem, &t) {
                        self.err(&format!(
                            "Array literal element type mismatch: expected '{}', got '{}'",
                            hint_elem, t
                        ));
                    }
                }
                SemType::Array {
                    elem: hint_elem.clone(),
                    size: *size,
                }
            }

            _ => self.infer_type(node),
        }
    }

    fn const_eval(&self, node: &ParseNode) -> Option<Value> {
        match node {
            ParseNode::IntLit(n) => Some(Value::Int(*n)),
            ParseNode::FloatLit(f) => Some(Value::Float(*f)),
            ParseNode::CharLit(c) => Some(Value::Char(*c)),
            ParseNode::BoolLit(b) => Some(Value::Boolean(*b)),
            ParseNode::Null => Some(Value::Null),
            ParseNode::StringLit(_) => Some(Value::Composite),
            ParseNode::ArrayLit(_) => Some(Value::Composite),
            ParseNode::StructLit(_) => Some(Value::Composite),

            ParseNode::AccessChain { base, steps } if steps.is_empty() => {
                self.scope.lookup(base).and_then(|s| s.value.clone())
            }

            ParseNode::Cast { target_type, expr } => {
                let inner = self.const_eval(expr)?;
                let target = sem_type_from_parse_node(target_type.as_ref());
                match (target, inner) {
                    (SemType::Int, Value::Float(f)) => Some(Value::Int(f as i64)),
                    (SemType::Float, Value::Int(n)) => Some(Value::Float(n as f64)),
                    (SemType::Int, Value::Int(n)) => Some(Value::Int(n)),
                    (SemType::Float, Value::Float(f)) => Some(Value::Float(f)),
                    _ => None,
                }
            }

            ParseNode::Add { left, op, right } => {
                let l = self.const_eval(left)?;
                let r = self.const_eval(right)?;
                match (l, r) {
                    (Value::Int(a), Value::Int(b)) => match op {
                        AddOp::Add => Some(Value::Int(a + b)),
                        AddOp::Sub => Some(Value::Int(a - b)),
                    },
                    (Value::Float(a), Value::Float(b)) => match op {
                        AddOp::Add => Some(Value::Float(a + b)),
                        AddOp::Sub => Some(Value::Float(a - b)),
                    },
                    _ => None,
                }
            }

            ParseNode::Mul { left, op, right } => {
                let l = self.const_eval(left)?;
                let r = self.const_eval(right)?;
                match (l, r) {
                    (Value::Int(a), Value::Int(b)) => match op {
                        MulOp::Mul => Some(Value::Int(a * b)),
                        MulOp::Div if b != 0 => Some(Value::Int(a / b)),
                        MulOp::Mod if b != 0 => Some(Value::Int(a % b)),
                        _ => None,
                    },
                    (Value::Float(a), Value::Float(b)) => match op {
                        MulOp::Mul => Some(Value::Float(a * b)),
                        MulOp::Div => Some(Value::Float(a / b)),
                        _ => None,
                    },
                    _ => None,
                }
            }

            _ => None,
        }
    }
}

pub fn analyse(root: &ParseNode) -> Semanter {
    let mut sem = Semanter::new();
    sem.check(root);
    sem
}
