use crate::compiler::parser::{AccessStep, AssignOp, CmpOp, MulOp, ParseNode, UnOp};
use std::collections::HashMap;
use std::fmt;

pub const BUILTIN_FUNCTIONS: &[&str] = &[
    "print", "input", "starts", "ends", "append", "pop", "insert", "find", "delete", "len", "pow",
    "abs", "sqrt",
];

#[derive(Debug, Clone, PartialEq)]
pub enum SemType {
    Int,
    Float,
    Char,
    Boolean,
    Void,
    Array { elem: Box<SemType>, size: i64 },
    List { elem: Box<SemType> },
    Struct(String),

    Unknown,
}

impl SemType {
    fn display(&self) -> String {
        match self {
            SemType::Int => ":int".into(),
            SemType::Float => ":float".into(),
            SemType::Char => ":char".into(),
            SemType::Boolean => ":boolean".into(),
            SemType::Void => ":void".into(),
            SemType::Array { elem, size } => format!(":array<{}, {}>", elem.display(), size),
            SemType::List { elem } => format!(":list<{}>", elem.display()),
            SemType::Struct(n) => format!(":struct<{}>", n),
            SemType::Unknown => "<unknown>".into(),
        }
    }

    fn is_numeric(&self) -> bool {
        matches!(self, SemType::Int | SemType::Float)
    }

    fn is_integer(&self) -> bool {
        matches!(self, SemType::Int)
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub sem_type: SemType,
    pub scope_depth: usize,
    pub origin: String,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match &self.kind {
            SymbolKind::Variable => self.sem_type.display(),
            SymbolKind::Function { params } => {
                let ps: Vec<String> = params.iter().map(|p| p.display()).collect();
                format!("fn({}) -> {}", ps.join(", "), self.sem_type.display())
            }
            SymbolKind::Struct { fields } => {
                let fs: Vec<String> = fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, t.display()))
                    .collect();
                format!("struct {{ {} }}", fs.join(", "))
            }
        };
        write!(
            f,
            "{:<30} : {:<40} [scope={}] [{}]",
            self.name, type_str, self.scope_depth, self.origin
        )
    }
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable,
    Function { params: Vec<SemType> },
    Struct { fields: Vec<(String, SemType)> },
}

struct ScopeStack {
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
        self.scopes.pop();
    }

    fn pop_with_frame(&mut self) -> HashMap<String, Symbol> {
        self.scopes.pop().unwrap_or_default()
    }

    fn define(&mut self, sym: Symbol) {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(sym.name.clone(), sym);
        }
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(s) = scope.get(name) {
                return Some(s);
            }
        }
        None
    }

    fn defined_in_current(&self, name: &str) -> bool {
        self.scopes.last().map_or(false, |s| s.contains_key(name))
    }
}

#[derive(Debug, Clone)]
pub struct SemanticWarning {
    pub message: String,
}

impl fmt::Display for SemanticWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1;33mWarning:\x1b[0m {}", self.message)
    }
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
}

impl SemanticError {
    fn new(msg: impl Into<String>) -> Self {
        SemanticError {
            message: msg.into(),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\x1b[1;31mSemantic Error:\x1b[0m {}", self.message)
    }
}

pub struct SemanticResult {
    pub errors: Vec<SemanticError>,
    pub warnings: Vec<SemanticWarning>,
    pub symbol_table: Vec<Symbol>,
}

impl SemanticResult {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn print_errors(&self) {
        if !self.warnings.is_empty() {
            println!("\x1b[1;33m⚠  {} warning(s):\x1b[0m", self.warnings.len());
            for w in &self.warnings {
                eprintln!("   {}", w);
            }
            println!();
        }
        if self.errors.is_empty() {
            println!("\x1b[1;32m No semantic errors.\x1b[0m\n");
        } else {
            println!("\x1b[1;31m✗  {} error(s):\x1b[0m", self.errors.len());
            for e in &self.errors {
                eprintln!("   {}", e);
            }
            println!();
        }
    }

    pub fn print_symbol_table(&self) {
        let line = "═".repeat(95);
        println!("\n\x1b[1;34m╔{}╗\x1b[0m", line);
        println!("\x1b[1;34m║{:^95}║\x1b[0m", "  SYMBOL TABLE  ");
        println!("\x1b[1;34m╚{}╝\x1b[0m", line);
        println!(
            "\x1b[1m{:<30}   {:<40}   SCOPE   ORIGIN\x1b[0m",
            "NAME", "TYPE"
        );
        println!("{}", "─".repeat(95));
        for sym in &self.symbol_table {
            println!("{}", sym);
        }
        println!("{}", "─".repeat(95));
        println!("  {} symbol(s)\n", self.symbol_table.len());
    }
}

struct Analyzer {
    scopes: ScopeStack,
    errors: Vec<SemanticError>,
    warnings: Vec<SemanticWarning>,
    all_symbols: Vec<Symbol>,

    current_return_type: Option<SemType>,

    loop_depth: usize,
    current_origin: String,
}

impl Analyzer {
    fn new() -> Self {
        let mut scopes = ScopeStack::new();

        let builtins: &[(&str, &[SemType], SemType)] = &[
            (
                "append",
                &[SemType::Unknown, SemType::Unknown],
                SemType::Void,
            ),
            (
                "prepend",
                &[SemType::Unknown, SemType::Unknown],
                SemType::Void,
            ),
            ("remove", &[SemType::Unknown, SemType::Int], SemType::Void),
            ("len", &[SemType::Unknown], SemType::Int),
            ("pop", &[SemType::Unknown], SemType::Unknown),
            ("print", &[SemType::Unknown], SemType::Void),
            ("println", &[SemType::Unknown], SemType::Void),
            (
                "input",
                &[],
                SemType::List {
                    elem: Box::new(SemType::Char),
                },
            ),
            ("abs", &[SemType::Unknown], SemType::Unknown),
            ("sqrt", &[SemType::Float], SemType::Float),
            ("pow", &[SemType::Float, SemType::Float], SemType::Float),
            ("floor", &[SemType::Float], SemType::Int),
            ("ceil", &[SemType::Float], SemType::Int),
            (
                "min",
                &[SemType::Unknown, SemType::Unknown],
                SemType::Unknown,
            ),
            (
                "max",
                &[SemType::Unknown, SemType::Unknown],
                SemType::Unknown,
            ),
            (
                "insert",
                &[SemType::Unknown, SemType::Unknown],
                SemType::Void,
            ),
            ("delete", &[SemType::Unknown, SemType::Int], SemType::Void),
            ("find", &[SemType::Unknown, SemType::Unknown], SemType::Int),
            (
                "starts",
                &[SemType::Unknown, SemType::Unknown],
                SemType::Boolean,
            ),
            (
                "ends",
                &[SemType::Unknown, SemType::Unknown],
                SemType::Boolean,
            ),
            ("to_int", &[SemType::Unknown], SemType::Int),
            ("to_float", &[SemType::Unknown], SemType::Float),
            (
                "to_str",
                &[SemType::Unknown],
                SemType::List {
                    elem: Box::new(SemType::Char),
                },
            ),
        ];
        for (name, params, ret) in builtins {
            scopes.define(Symbol {
                name: name.to_string(),
                kind: SymbolKind::Function {
                    params: params.to_vec(),
                },
                sem_type: ret.clone(),
                scope_depth: 0,
                origin: "builtin".to_string(),
            });
        }
        Analyzer {
            scopes,
            errors: Vec::new(),
            warnings: Vec::new(),
            all_symbols: Vec::new(),
            current_return_type: None,
            loop_depth: 0,
            current_origin: "global".to_string(),
        }
    }

    fn error(&mut self, msg: impl Into<String>) {
        self.errors.push(SemanticError::new(msg));
    }

    fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(SemanticWarning {
            message: msg.into(),
        });
    }

    fn scope_depth(&self) -> usize {
        self.scopes.scopes.len().saturating_sub(1)
    }

    fn declare_sym(&mut self, sym: Symbol) {
        if self.scopes.defined_in_current(&sym.name) {
            return;
        }
        self.all_symbols.push(sym.clone());
        self.scopes.define(sym);
    }

    fn resolve_type_node(&self, node: &ParseNode) -> SemType {
        match node {
            ParseNode::TypeInt => SemType::Int,
            ParseNode::TypeFloat => SemType::Float,
            ParseNode::TypeChar => SemType::Char,
            ParseNode::TypeBoolean => SemType::Boolean,
            ParseNode::TypeVoid => SemType::Void,
            ParseNode::TypeArray { elem, size } => SemType::Array {
                elem: Box::new(self.resolve_type_node(elem)),
                size: *size,
            },
            ParseNode::TypeList { elem } => SemType::List {
                elem: Box::new(self.resolve_type_node(elem)),
            },
            ParseNode::TypeStruct { name } => SemType::Struct(name.clone()),
            _ => SemType::Unknown,
        }
    }

    fn qualify_struct_type(ty: &SemType, module: &str) -> SemType {
        match ty {
            SemType::Struct(n) if !n.contains("::") => {
                SemType::Struct(format!("{}::{}", module, n))
            }
            SemType::Array { elem, size } => SemType::Array {
                elem: Box::new(Self::qualify_struct_type(elem, module)),
                size: *size,
            },
            SemType::List { elem } => SemType::List {
                elem: Box::new(Self::qualify_struct_type(elem, module)),
            },
            other => other.clone(),
        }
    }

    fn qualify_symbol_kind(kind: &SymbolKind, module: &str) -> SymbolKind {
        match kind {
            SymbolKind::Struct { fields } => {
                let qfields = fields
                    .iter()
                    .map(|(fname, ftype)| (fname.clone(), Self::qualify_struct_type(ftype, module)))
                    .collect();
                SymbolKind::Struct { fields: qfields }
            }
            SymbolKind::Function { params } => {
                let qparams = params
                    .iter()
                    .map(|p| Self::qualify_struct_type(p, module))
                    .collect();
                SymbolKind::Function { params: qparams }
            }
            SymbolKind::Variable => SymbolKind::Variable,
        }
    }

    fn types_compatible(a: &SemType, b: &SemType) -> bool {
        if matches!(a, SemType::Void) || matches!(b, SemType::Void) {
            return false;
        }

        if a == b {
            return true;
        }

        if matches!(a, SemType::Unknown) || matches!(b, SemType::Unknown) {
            return true;
        }

        if let (SemType::List { elem: ea }, SemType::List { elem: eb }) = (a, b) {
            if matches!(ea.as_ref(), SemType::Unknown) || matches!(eb.as_ref(), SemType::Unknown) {
                return true;
            }
            return Self::types_compatible(ea, eb);
        }

        if let (SemType::Array { elem: ae, size: sa }, SemType::Array { elem: be, size: sb }) =
            (a, b)
        {
            return sa == sb && Self::types_compatible(ae, be);
        }

        if let (SemType::List { elem: le }, SemType::Array { elem: ae, .. }) = (a, b) {
            if matches!(le.as_ref(), SemType::Unknown) || matches!(ae.as_ref(), SemType::Unknown) {
                return true;
            }
            return Self::types_compatible(le, ae);
        }

        false
    }

    fn infer_expr(&mut self, node: &ParseNode) -> SemType {
        match node {
            ParseNode::IntLit(_) => SemType::Int,
            ParseNode::FloatLit(_) => SemType::Float,
            ParseNode::CharLit(_) => SemType::Char,
            ParseNode::StringLit(s) => SemType::Array {
                elem: Box::new(SemType::Char),
                size: s.chars().count() as i64,
            },
            ParseNode::BoolLit(_) => SemType::Boolean,
            ParseNode::Null => SemType::Unknown,

            ParseNode::Identifier(name) => match self.scopes.lookup(name) {
                Some(sym) => sym.sem_type.clone(),
                None => {
                    self.error(format!("undefined variable `{}`", name));
                    SemType::Unknown
                }
            },

            ParseNode::AccessChain { base, steps } => {
                let qualified_key: Option<String> =
                    if let Some(AccessStep::Field(first_field)) = steps.first() {
                        let key = format!("{}::{}", base, first_field);
                        if self.scopes.lookup(&key).is_some() {
                            Some(key)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                let (mut ty, remaining_steps): (SemType, &[AccessStep]) =
                    if let Some(ref qkey) = qualified_key {
                        let t = self.scopes.lookup(qkey).unwrap().sem_type.clone();
                        (t, &steps[1..])
                    } else {
                        let t = match self.scopes.lookup(base) {
                            Some(sym) => sym.sem_type.clone(),
                            None => {
                                let qualified = steps
                                    .iter()
                                    .take_while(|s| matches!(s, AccessStep::Field(_)))
                                    .fold(base.clone(), |acc, s| {
                                        if let AccessStep::Field(f) = s {
                                            format!("{}::{}", acc, f)
                                        } else {
                                            acc
                                        }
                                    });
                                self.error(format!("undefined identifier `{}`", qualified));
                                SemType::Unknown
                            }
                        };
                        (t, steps.as_slice())
                    };

                for step in remaining_steps {
                    ty = match step {
                        AccessStep::Field(field) => match &ty {
                            SemType::Struct(struct_name) => {
                                let struct_name = struct_name.clone();
                                match self.scopes.lookup(&struct_name) {
                                    Some(Symbol {
                                        kind: SymbolKind::Struct { fields },
                                        ..
                                    }) => {
                                        let found = fields
                                            .iter()
                                            .find(|(n, _)| n == field)
                                            .map(|(_, t)| t.clone());
                                        match found {
                                            Some(t) => t,
                                            None => {
                                                self.error(format!(
                                                    "struct `{}` has no field `{}`",
                                                    struct_name, field
                                                ));
                                                SemType::Unknown
                                            }
                                        }
                                    }
                                    _ => {
                                        self.error(format!(
                                            "undefined struct type `{}`",
                                            struct_name
                                        ));
                                        SemType::Unknown
                                    }
                                }
                            }
                            SemType::Unknown => SemType::Unknown,
                            other => {
                                let msg = format!(
                                    "type `{}` has no fields; cannot access `{}`",
                                    other.display(),
                                    field
                                );
                                self.error(msg);
                                SemType::Unknown
                            }
                        },
                        AccessStep::Index(idx_expr) => {
                            let idx_ty = self.infer_expr(idx_expr);
                            if !matches!(idx_ty, SemType::Int | SemType::Unknown) {
                                self.error(format!(
                                    "array/list index must be `:int`, got `{}`",
                                    idx_ty.display()
                                ));
                            }

                            if let (SemType::Array { size, .. }, ParseNode::IntLit(idx)) =
                                (&ty, idx_expr.as_ref())
                            {
                                if *idx < 0 || *idx >= *size {
                                    self.error(format!(
                                        "index {} is out of bounds for array of size {}",
                                        idx, size
                                    ));
                                }
                            }
                            match &ty {
                                SemType::Array { elem, .. } => *elem.clone(),
                                SemType::List { elem } => *elem.clone(),
                                SemType::Unknown => SemType::Unknown,
                                other => {
                                    self.error(format!(
                                        "type `{}` is not indexable",
                                        other.display()
                                    ));
                                    SemType::Unknown
                                }
                            }
                        }
                        AccessStep::Call(args) => {
                            let arg_types: Vec<SemType> =
                                args.iter().map(|a| self.infer_expr(a)).collect();

                            let func_name = if let Some(ref qkey) = qualified_key {
                                qkey.clone()
                            } else {
                                base.clone()
                            };
                            let func_sym = self.scopes.lookup(&func_name).cloned();

                            if let Some(Symbol {
                                kind:
                                    SymbolKind::Function {
                                        params: ref param_types,
                                    },
                                sem_type: ref ret,
                                ..
                            }) = func_sym
                            {
                                let variadic = param_types.len() == 1
                                    && matches!(param_types[0], SemType::Unknown);

                                if !variadic && arg_types.len() != param_types.len() {
                                    self.error(format!(
                                        "function `{}` expects {} argument(s), got {}",
                                        func_name,
                                        param_types.len(),
                                        arg_types.len()
                                    ));
                                }

                                if !variadic {
                                    for (i, (pt, at)) in
                                        param_types.iter().zip(arg_types.iter()).enumerate()
                                    {
                                        if matches!(pt, SemType::Unknown) {
                                            continue;
                                        }
                                        if !Self::types_compatible(pt, at) {
                                            self.error(format!(
                                                "argument {} of `{}` expects type `{}`, got `{}`",
                                                i + 1,
                                                func_name,
                                                pt.display(),
                                                at.display()
                                            ));
                                        }
                                    }
                                }

                                let list_only_func = matches!(
                                    func_name.as_str(),
                                    "append" | "insert" | "pop" | "delete"
                                ) || func_name.ends_with("::append")
                                    || func_name.ends_with("::insert")
                                    || func_name.ends_with("::pop")
                                    || func_name.ends_with("::delete");

                                if list_only_func {
                                    if let Some(at) = arg_types.first() {
                                        if !matches!(at, SemType::List { .. } | SemType::Unknown) {
                                            self.error(format!(
                                                "`{}` requires a `:list<T>` as its first argument, \
                                                 got `{}`; arrays are fixed-size and cannot be \
                                                 modified with list functions",
                                                func_name,
                                                at.display()
                                            ));
                                        }
                                    }
                                }

                                let is_append_or_insert = func_name == "append"
                                    || func_name == "insert"
                                    || func_name.ends_with("::append")
                                    || func_name.ends_with("::insert");
                                if is_append_or_insert && arg_types.len() >= 2 {
                                    if let Some(SemType::List { elem: list_elem }) =
                                        arg_types.first()
                                    {
                                        let value_ty = &arg_types[1];
                                        if !matches!(value_ty, SemType::Unknown)
                                            && !matches!(list_elem.as_ref(), SemType::Unknown)
                                            && !Self::types_compatible(list_elem, value_ty)
                                        {
                                            self.error(format!(
                                                "`{}` expects an element of type `{}` \
                                                 (matching the list), but got `{}`",
                                                func_name,
                                                list_elem.display(),
                                                value_ty.display()
                                            ));
                                        }
                                    }
                                }

                                let is_find = func_name == "find" || func_name.ends_with("::find");
                                if is_find && arg_types.len() >= 2 {
                                    if let Some(SemType::List { elem: list_elem }) =
                                        arg_types.first()
                                    {
                                        let search_ty = &arg_types[1];
                                        if !matches!(search_ty, SemType::Unknown)
                                            && !matches!(list_elem.as_ref(), SemType::Unknown)
                                            && !Self::types_compatible(list_elem, search_ty)
                                        {
                                            self.error(format!(
                                                "`find` searches for a value of type `{}` \
                                                 (matching the list element type), but got `{}`",
                                                list_elem.display(),
                                                search_ty.display()
                                            ));
                                        }
                                    }
                                }

                                let is_pop = func_name == "pop" || func_name.ends_with("::pop");
                                if is_pop {
                                    if let Some(SemType::List { elem }) = arg_types.first() {
                                        return *elem.clone();
                                    }
                                }

                                ret.clone()
                            } else {
                                if self.scopes.lookup(&func_name).is_some() {
                                    self.error(format!(
                                        "`{}` is not a function and cannot be called",
                                        func_name
                                    ));
                                } else {
                                    self.error(format!("undefined function `{}`", func_name));
                                }
                                SemType::Unknown
                            }
                        }
                    };
                }
                ty
            }

            ParseNode::Cast { target_type, expr } => {
                let src = self.infer_expr(expr);
                let tgt = self.resolve_type_node(target_type);

                let legal = matches!(src, SemType::Unknown)
                    || matches!(tgt, SemType::Unknown)
                    || matches!(
                        (&src, &tgt),
                        (SemType::Int, SemType::Int)
                            | (SemType::Float, SemType::Float)
                            | (SemType::Char, SemType::Char)
                            | (SemType::Boolean, SemType::Boolean)
                            | (SemType::Int, SemType::Float)
                            | (SemType::Float, SemType::Int)
                            | (SemType::Int, SemType::Char)
                            | (SemType::Char, SemType::Int)
                            | (SemType::Int, SemType::Boolean)
                            | (SemType::Boolean, SemType::Int)
                            | (SemType::Float, SemType::Boolean)
                            | (SemType::Boolean, SemType::Float)
                    );
                if !legal {
                    self.error(format!(
                        "illegal cast from `{}` to `{}`; only these casts are allowed: \
                         `:int`↔`:float`, `:int`↔`:char`, `:int`↔`:boolean`, `:float`↔`:boolean`",
                        src.display(),
                        tgt.display()
                    ));
                }
                tgt
            }

            ParseNode::ArrayLit(elems) => {
                if elems.is_empty() {
                    return SemType::Unknown;
                }
                let elem_types: Vec<SemType> = elems.iter().map(|e| self.infer_expr(e)).collect();
                let elem_ty = elem_types.first().cloned().unwrap_or(SemType::Unknown);
                for (i, t) in elem_types.iter().enumerate() {
                    if !Self::types_compatible(&elem_ty, t) {
                        self.error(format!(
                            "array/list literal element {} has type `{}`, expected `{}`",
                            i,
                            t.display(),
                            elem_ty.display()
                        ));
                    }
                }

                SemType::Array {
                    elem: Box::new(elem_ty),
                    size: elem_types.len() as i64,
                }
            }

            ParseNode::StructLit(fields) => {
                for (_, val) in fields {
                    self.infer_expr(val);
                }
                SemType::Unknown
            }

            ParseNode::LogOr { left, right } | ParseNode::LogAnd { left, right } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Boolean | SemType::Unknown) {
                    self.error(format!(
                        "logical operand must be `:boolean`, got `{}`",
                        lt.display()
                    ));
                }
                if !matches!(rt, SemType::Boolean | SemType::Unknown) {
                    self.error(format!(
                        "logical operand must be `:boolean`, got `{}`",
                        rt.display()
                    ));
                }
                SemType::Boolean
            }

            ParseNode::LogNot { operand } => {
                let t = self.infer_expr(operand);
                if !matches!(t, SemType::Boolean | SemType::Unknown) {
                    self.error(format!(
                        "`!not` operand must be `:boolean`, got `{}`",
                        t.display()
                    ));
                }
                SemType::Boolean
            }

            ParseNode::Cmp { left, right, op } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);

                if !matches!(lt, SemType::Unknown) && !matches!(rt, SemType::Unknown) {
                    let is_simple = |t: &SemType| {
                        matches!(
                            t,
                            SemType::Int | SemType::Float | SemType::Char | SemType::Boolean
                        )
                    };
                    if !is_simple(&lt) {
                        let op_str = match op {
                            CmpOp::EqEq => "==",
                            CmpOp::Ne => "~=",
                            CmpOp::Gt => ">",
                            CmpOp::Lt => "<",
                            CmpOp::Ge => ">=",
                            CmpOp::Le => "<=",
                        };
                        self.error(format!(
                            "`{}` is not valid for type `{}`; \
                             only `:int`, `:float`, `:char`, and `:boolean` can be compared",
                            op_str,
                            lt.display()
                        ));
                    } else if !is_simple(&rt) {
                        let op_str = match op {
                            CmpOp::EqEq => "==",
                            CmpOp::Ne => "~=",
                            CmpOp::Gt => ">",
                            CmpOp::Lt => "<",
                            CmpOp::Ge => ">=",
                            CmpOp::Le => "<=",
                        };
                        self.error(format!(
                            "`{}` is not valid for type `{}`; \
                             only `:int`, `:float`, `:char`, and `:boolean` can be compared",
                            op_str,
                            rt.display()
                        ));
                    } else if !Self::types_compatible(&lt, &rt) {
                        let op_str = match op {
                            CmpOp::EqEq => "==",
                            CmpOp::Ne => "~=",
                            CmpOp::Gt => ">",
                            CmpOp::Lt => "<",
                            CmpOp::Ge => ">=",
                            CmpOp::Le => "<=",
                        };
                        self.error(format!(
                            "cannot compare `{}` with `{}` using `{}`; \
                             both operands must be the same type",
                            lt.display(),
                            rt.display(),
                            op_str
                        ));
                    } else {
                        let is_ordering_op =
                            matches!(op, CmpOp::Gt | CmpOp::Lt | CmpOp::Ge | CmpOp::Le);
                        if is_ordering_op
                            && !matches!(lt, SemType::Int | SemType::Float | SemType::Char)
                        {
                            let op_str = match op {
                                CmpOp::Gt => ">",
                                CmpOp::Lt => "<",
                                CmpOp::Ge => ">=",
                                CmpOp::Le => "<=",
                                _ => unreachable!(),
                            };
                            self.error(format!(
                                "`{}` is not valid for type `{}`; \
                                 ordering comparisons require `:int`, `:float`, or `:char`",
                                op_str,
                                lt.display()
                            ));
                        }
                    }
                }
                SemType::Boolean
            }

            ParseNode::BitOr { left, right }
            | ParseNode::BitXor { left, right }
            | ParseNode::BitAnd { left, right } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Int | SemType::Unknown) {
                    self.error(format!(
                        "bitwise operand must be `:int`, got `{}`",
                        lt.display()
                    ));
                }
                if !matches!(rt, SemType::Int | SemType::Unknown) {
                    self.error(format!(
                        "bitwise operand must be `:int`, got `{}`",
                        rt.display()
                    ));
                }
                SemType::Int
            }

            ParseNode::BitShift { left, right, .. } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Int | SemType::Unknown) {
                    self.error(format!(
                        "`<<`/`>>` left operand must be `:int`, got `{}`",
                        lt.display()
                    ));
                }
                if !matches!(rt, SemType::Int | SemType::Unknown) {
                    self.error(format!(
                        "`<<`/`>>` shift amount must be `:int`, got `{}`",
                        rt.display()
                    ));
                }
                SemType::Int
            }

            ParseNode::Add { left, right, .. } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !lt.is_numeric() && !matches!(lt, SemType::Unknown) {
                    self.error(format!(
                        "additive operand must be numeric, got `{}`",
                        lt.display()
                    ));
                }
                if !rt.is_numeric() && !matches!(rt, SemType::Unknown) {
                    self.error(format!(
                        "additive operand must be numeric, got `{}`",
                        rt.display()
                    ));
                }
                if lt.is_numeric() && rt.is_numeric() && lt != rt {
                    self.error(format!(
                        "type mismatch in arithmetic: `{}` and `{}` — use an explicit cast",
                        lt.display(),
                        rt.display()
                    ));
                }
                if matches!(lt, SemType::Unknown) && matches!(rt, SemType::Unknown) {
                    SemType::Unknown
                } else if matches!(lt, SemType::Float) || matches!(rt, SemType::Float) {
                    SemType::Float
                } else if matches!(lt, SemType::Unknown) {
                    rt
                } else if matches!(rt, SemType::Unknown) {
                    lt
                } else {
                    SemType::Int
                }
            }

            ParseNode::Mul { left, right, op } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if matches!(op, MulOp::Mod) {
                    if !matches!(lt, SemType::Int | SemType::Unknown) {
                        self.error(format!(
                            "`%` left operand must be `:int`, got `{}`",
                            lt.display()
                        ));
                    }
                    if !matches!(rt, SemType::Int | SemType::Unknown) {
                        self.error(format!(
                            "`%` right operand must be `:int`, got `{}`",
                            rt.display()
                        ));
                    }
                    return if matches!(lt, SemType::Unknown) && matches!(rt, SemType::Unknown) {
                        SemType::Unknown
                    } else {
                        SemType::Int
                    };
                }
                if !lt.is_numeric() && !matches!(lt, SemType::Unknown) {
                    self.error(format!(
                        "multiplicative operand must be numeric, got `{}`",
                        lt.display()
                    ));
                }
                if !rt.is_numeric() && !matches!(rt, SemType::Unknown) {
                    self.error(format!(
                        "multiplicative operand must be numeric, got `{}`",
                        rt.display()
                    ));
                }
                if lt.is_numeric() && rt.is_numeric() && lt != rt {
                    self.error(format!(
                        "type mismatch in arithmetic: `{}` and `{}` — use an explicit cast",
                        lt.display(),
                        rt.display()
                    ));
                }
                if matches!(lt, SemType::Unknown) && matches!(rt, SemType::Unknown) {
                    SemType::Unknown
                } else if matches!(lt, SemType::Float) || matches!(rt, SemType::Float) {
                    SemType::Float
                } else if matches!(lt, SemType::Unknown) {
                    rt
                } else if matches!(rt, SemType::Unknown) {
                    lt
                } else {
                    SemType::Int
                }
            }

            ParseNode::Unary { op, operand } => {
                let t = self.infer_expr(operand);
                match op {
                    UnOp::BitNot => {
                        if !matches!(t, SemType::Int | SemType::Unknown) {
                            self.error(format!(
                                "`~` operand must be `:int`, got `{}`",
                                t.display()
                            ));
                        }
                        SemType::Int
                    }
                    UnOp::Neg => {
                        if !t.is_numeric() && !matches!(t, SemType::Unknown) {
                            self.error(format!(
                                "unary `-` operand must be numeric, got `{}`",
                                t.display()
                            ));
                        }
                        t
                    }
                }
            }

            _ => SemType::Unknown,
        }
    }

    fn validate_struct_lit(&mut self, struct_name: &str, fields: &[(String, ParseNode)]) {
        let sym = self.scopes.lookup(struct_name).cloned();
        match sym {
            Some(Symbol {
                kind:
                    SymbolKind::Struct {
                        fields: ref def_fields,
                    },
                ..
            }) => {
                let def_fields = def_fields.clone();

                for (fname, fval) in fields {
                    match def_fields.iter().find(|(n, _)| n == fname) {
                        None => {
                            self.error(format!(
                                "struct `{}` has no field `{}`",
                                struct_name, fname
                            ));
                        }
                        Some((_, expected_ty)) => {
                            if let (
                                SemType::Struct(ref sub_name),
                                ParseNode::StructLit(ref sub_fields),
                            ) = (expected_ty, fval)
                            {
                                let sub_name = sub_name.clone();
                                let sub_fields = sub_fields.clone();
                                self.validate_struct_lit(&sub_name, &sub_fields);
                            } else {
                                let actual_ty = self.infer_expr(fval);
                                if !matches!(actual_ty, SemType::Unknown)
                                    && !Self::types_compatible(expected_ty, &actual_ty)
                                {
                                    self.error(format!(
                                        "field `{}` of struct `{}` expects type `{}`, got `{}`",
                                        fname,
                                        struct_name,
                                        expected_ty.display(),
                                        actual_ty.display()
                                    ));
                                }
                            }
                        }
                    }
                }

                for (def_name, _) in &def_fields {
                    if !fields.iter().any(|(n, _)| n == def_name) {
                        self.error(format!(
                            "struct `{}` initializer is missing field `{}`",
                            struct_name, def_name
                        ));
                    }
                }
            }
            _ => {
                for (_, fval) in fields {
                    self.infer_expr(fval);
                }
            }
        }
    }

    fn analyze_items(&mut self, items: &[ParseNode]) {
        for item in items {
            match item {
                ParseNode::StructDef { name, fields } => {
                    if self.scopes.defined_in_current(name) {
                        self.error(format!(
                            "struct `{}` is already defined in this scope",
                            name
                        ));
                        continue;
                    }
                    let resolved_fields: Vec<(String, SemType)> = fields
                        .iter()
                        .filter_map(|f| {
                            if let ParseNode::Field {
                                data_type,
                                name: fname,
                            } = f
                            {
                                Some((fname.clone(), self.resolve_type_node(data_type)))
                            } else {
                                None
                            }
                        })
                        .collect();
                    self.declare_sym(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Struct {
                            fields: resolved_fields,
                        },
                        sem_type: SemType::Struct(name.clone()),
                        scope_depth: self.scope_depth(),
                        origin: self.current_origin.clone(),
                    });
                }
                ParseNode::FuncDef {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    if self.scopes.defined_in_current(name) {
                        self.error(format!(
                            "function `{}` is already defined in this scope",
                            name
                        ));
                        continue;
                    }
                    let param_types: Vec<SemType> = params
                        .iter()
                        .filter_map(|p| {
                            if let ParseNode::Param { data_type, .. } = p {
                                Some(self.resolve_type_node(data_type))
                            } else {
                                None
                            }
                        })
                        .collect();
                    let ret = self.resolve_type_node(return_type);
                    self.declare_sym(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Function {
                            params: param_types,
                        },
                        sem_type: ret,
                        scope_depth: self.scope_depth(),
                        origin: format!("func:{}", name),
                    });
                }
                ParseNode::Module {
                    name,
                    items: mod_items,
                } => {
                    let saved_origin = self.current_origin.clone();
                    self.current_origin = format!("module:{}", name);
                    self.scopes.push();
                    self.analyze_items(mod_items);
                    let frame = self.scopes.pop_with_frame();
                    for (sym_name, sym) in &frame {
                        if sym.origin == "builtin" {
                            continue;
                        }
                        let qualified = format!("{}::{}", name, sym_name);
                        if !self.scopes.defined_in_current(&qualified) {
                            let qualified_kind = Self::qualify_symbol_kind(&sym.kind, name);
                            let qualified_sem_type = Self::qualify_struct_type(&sym.sem_type, name);
                            let qualified_sym = Symbol {
                                name: qualified.clone(),
                                kind: qualified_kind,
                                sem_type: qualified_sem_type,
                                scope_depth: self.scope_depth(),
                                origin: format!("module:{}", name),
                            };
                            self.scopes.define(qualified_sym.clone());
                            self.all_symbols.push(qualified_sym);
                        }
                    }
                    self.current_origin = saved_origin;
                }
                _ => {}
            }
        }

        for item in items {
            if !matches!(item, ParseNode::Module { .. }) {
                self.analyze_node(item);
            }
        }
    }

    fn analyze_node(&mut self, node: &ParseNode) {
        match node {
            ParseNode::Program(items) => {
                self.analyze_items(items);
            }

            ParseNode::Module { name, items } => {
                let saved_origin = self.current_origin.clone();
                self.current_origin = format!("module:{}", name);
                self.scopes.push();
                self.analyze_items(items);
                let frame = self.scopes.pop_with_frame();

                for (sym_name, sym) in &frame {
                    if sym.origin == "builtin" {
                        continue;
                    }
                    let qualified = format!("{}::{}", name, sym_name);
                    let qualified_kind = Self::qualify_symbol_kind(&sym.kind, name);
                    let qualified_sem_type = Self::qualify_struct_type(&sym.sem_type, name);
                    let qualified_sym = Symbol {
                        name: qualified.clone(),
                        kind: qualified_kind,
                        sem_type: qualified_sem_type,
                        scope_depth: self.scope_depth(),
                        origin: format!("module:{}", name),
                    };
                    self.scopes.define(qualified_sym.clone());
                    self.all_symbols.push(qualified_sym);
                }
                self.current_origin = saved_origin;
            }

            ParseNode::StructDef { .. } => {}

            ParseNode::FuncDef {
                name,
                params,
                return_type,
                body,
            } => {
                if self.current_return_type.is_some() {
                    self.error(format!(
                        "function `{}` cannot be defined inside another function; \
                         move it to the top level",
                        name
                    ));
                    return;
                }
                let ret = self.resolve_type_node(return_type);
                let prev_ret = self.current_return_type.replace(ret.clone());
                let saved_origin = self.current_origin.clone();
                self.current_origin = format!("fn:{}", name);

                self.scopes.push();
                for param in params {
                    if let ParseNode::Param {
                        data_type,
                        name: pname,
                    } = param
                    {
                        let pt = self.resolve_type_node(data_type);
                        if matches!(pt, SemType::Void) {
                            self.error(format!(
                                "parameter `{}` of function `{}` cannot have type `:void`; \
                                 `:void` is only valid as a function return type",
                                pname, name
                            ));
                            continue;
                        }
                        if self.scopes.defined_in_current(pname) {
                            self.error(format!(
                                "duplicate parameter `{}` in function `{}`",
                                pname, name
                            ));
                        } else {
                            self.declare_sym(Symbol {
                                name: pname.clone(),
                                kind: SymbolKind::Variable,
                                sem_type: pt,
                                scope_depth: self.scope_depth(),
                                origin: format!("param:{}", name),
                            });
                        }
                    }
                }
                for stmt in body {
                    self.analyze_node(stmt);
                }
                self.scopes.pop();
                self.current_origin = saved_origin;
                self.current_return_type = prev_ret;
            }

            ParseNode::Decl {
                data_type,
                name,
                init,
            } => {
                let decl_ty = self.resolve_type_node(data_type);
                if matches!(decl_ty, SemType::Void) {
                    self.error(format!(
                        "cannot declare variable `{}` with type `:void`; \
                         `:void` is only valid as a function return type",
                        name
                    ));
                    return;
                }
                if let SemType::Array { elem, .. } = &decl_ty {
                    if matches!(elem.as_ref(), SemType::Void) {
                        self.error(format!(
                            "cannot declare array `{}` with element type `:void`",
                            name
                        ));
                        return;
                    }
                }
                if let SemType::List { elem } = &decl_ty {
                    if matches!(elem.as_ref(), SemType::Void) {
                        self.error(format!(
                            "cannot declare list `{}` with element type `:void`",
                            name
                        ));
                        return;
                    }
                }
                if self.scopes.defined_in_current(name) {
                    self.error(format!(
                        "variable `{}` is already declared in this scope",
                        name
                    ));
                } else {
                    self.declare_sym(Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Variable,
                        sem_type: decl_ty.clone(),
                        scope_depth: self.scope_depth(),
                        origin: self.current_origin.clone(),
                    });
                }
                if let Some(init_expr) = init {
                    let is_empty_literal =
                        matches!(init_expr.as_ref(), ParseNode::ArrayLit(e) if e.is_empty());
                    if is_empty_literal {
                        if !matches!(
                            decl_ty,
                            SemType::Array { .. } | SemType::List { .. } | SemType::Unknown
                        ) {
                            self.error(format!(
                                "cannot initialise `{}` (type `{}`) with `[]`; \
                                 `[]` is only valid for `:array` and `:list` types",
                                name,
                                decl_ty.display()
                            ));
                        }
                    } else {
                        if matches!(init_expr.as_ref(), ParseNode::Null)
                            && !matches!(decl_ty, SemType::Struct(_) | SemType::Unknown)
                        {
                            self.error(format!(
                                "cannot initialise `{}` with `!null`; \
                                 `!null` can only be assigned to struct-type fields",
                                name
                            ));
                            return;
                        }
                        let init_ty = self.infer_expr(init_expr);

                        if matches!(init_ty, SemType::Void) {
                            self.error(format!(
                                "cannot initialise `{}` with a `:void` value; \
                                 `:void` functions return no value",
                                name
                            ));
                            return;
                        }
                        if !Self::types_compatible(&decl_ty, &init_ty) {
                            let msg = match (&decl_ty, &init_ty) {
                                (
                                    SemType::Array { elem: de, size: ds },
                                    SemType::Array {
                                        elem: ie,
                                        size: is_,
                                    },
                                ) if de == ie => format!(
                                    "array `{}` declared with size {}, \
                                     but initializer has {} element(s)",
                                    name, ds, is_
                                ),
                                _ => format!(
                                    "cannot initialise `{}` (type `{}`) \
                                     with expression of type `{}`",
                                    name,
                                    decl_ty.display(),
                                    init_ty.display()
                                ),
                            };
                            self.error(msg);
                        }
                    }
                }
            }

            ParseNode::StructDecl {
                struct_name,
                var_name,
                init,
            } => {
                let sem_ty = SemType::Struct(struct_name.clone());
                if self.scopes.lookup(struct_name).is_none() {
                    self.error(format!("undefined struct type `{}`", struct_name));
                }
                if self.scopes.defined_in_current(var_name) {
                    self.error(format!(
                        "variable `{}` is already declared in this scope",
                        var_name
                    ));
                } else {
                    self.declare_sym(Symbol {
                        name: var_name.clone(),
                        kind: SymbolKind::Variable,
                        sem_type: sem_ty,
                        scope_depth: self.scope_depth(),
                        origin: self.current_origin.clone(),
                    });
                }
                if let Some(init_expr) = init {
                    if let ParseNode::StructLit(fields) = init_expr.as_ref() {
                        self.validate_struct_lit(struct_name, fields);
                    } else if matches!(init_expr.as_ref(), ParseNode::Null) {
                        self.error(format!(
                            "cannot initialise struct variable `{}` with `!null`; \
                             `!null` is only valid as a `:void` return value",
                            var_name
                        ));
                    } else {
                        self.infer_expr(init_expr);
                    }
                }
            }

            ParseNode::Assign { lvalue, op, expr } => {
                let lv_ty = self.infer_expr(lvalue);

                let is_int_only_op = matches!(
                    op,
                    AssignOp::AmpEq | AssignOp::PipeEq | AssignOp::CaretEq | AssignOp::PercentEq
                );
                if is_int_only_op && !matches!(lv_ty, SemType::Int | SemType::Unknown) {
                    let op_str = match op {
                        AssignOp::AmpEq => "`&=`",
                        AssignOp::PipeEq => "`|=`",
                        AssignOp::CaretEq => "`^=`",
                        AssignOp::PercentEq => "`%=`",
                        _ => unreachable!(),
                    };
                    self.error(format!(
                        "{} requires an `:int` target, got `{}`",
                        op_str,
                        lv_ty.display()
                    ));
                }

                let is_numeric_compound = matches!(
                    op,
                    AssignOp::PlusEq | AssignOp::MinusEq | AssignOp::StarEq | AssignOp::SlashEq
                );
                if is_numeric_compound
                    && !matches!(lv_ty, SemType::Int | SemType::Float | SemType::Unknown)
                {
                    let op_str = match op {
                        AssignOp::PlusEq => "`+=`",
                        AssignOp::MinusEq => "`-=`",
                        AssignOp::StarEq => "`*=`",
                        AssignOp::SlashEq => "`/=`",
                        _ => unreachable!(),
                    };
                    self.error(format!(
                        "{} requires a numeric (`:int` or `:float`) target, got `{}`",
                        op_str,
                        lv_ty.display()
                    ));
                }

                let is_empty_literal =
                    matches!(expr.as_ref(), ParseNode::ArrayLit(e) if e.is_empty());
                if is_empty_literal {
                    if !matches!(
                        lv_ty,
                        SemType::Array { .. } | SemType::List { .. } | SemType::Unknown
                    ) {
                        self.error(format!(
                            "cannot assign `[]` to `{}`; \
                             `[]` is only valid for `:array` and `:list` types",
                            lv_ty.display()
                        ));
                    }
                } else {
                    if matches!(expr.as_ref(), ParseNode::Null)
                        && !matches!(lv_ty, SemType::Unknown | SemType::Struct(_))
                    {
                        self.error(format!(
                            "cannot assign `!null` to `{}`; \
                             `!null` can only be assigned to struct-type fields \
                             (for self-referential struct termination)",
                            lv_ty.display()
                        ));
                        return;
                    }

                    if matches!(op, AssignOp::Eq) {
                        if let (SemType::Struct(ref sname), ParseNode::StructLit(ref fields)) =
                            (&lv_ty, expr.as_ref())
                        {
                            let sname = sname.clone();
                            let fields = fields.clone();
                            self.validate_struct_lit(&sname, &fields);
                            return;
                        }
                    }
                    let rv_ty = self.infer_expr(expr);

                    if matches!(rv_ty, SemType::Void) {
                        self.error(
                            "cannot use a `:void` value in an expression; \
                             `:void` functions return no value"
                                .to_string(),
                        );
                        return;
                    }
                    let is_compound_op = !matches!(op, AssignOp::Eq);
                    if is_compound_op
                        && !matches!(lv_ty, SemType::Unknown)
                        && !matches!(rv_ty, SemType::Unknown)
                        && lv_ty != rv_ty
                    {
                        let op_str = match op {
                            AssignOp::PlusEq => "`+=`",
                            AssignOp::MinusEq => "`-=`",
                            AssignOp::StarEq => "`*=`",
                            AssignOp::SlashEq => "`/=`",
                            AssignOp::PercentEq => "`%=`",
                            AssignOp::AmpEq => "`&=`",
                            AssignOp::PipeEq => "`|=`",
                            AssignOp::CaretEq => "`^=`",
                            AssignOp::Eq => unreachable!(),
                        };
                        self.error(format!(
                            "type mismatch in {}: left is `{}`, right is `{}` \
                             — operands must be the same type; use an explicit cast",
                            op_str,
                            lv_ty.display(),
                            rv_ty.display()
                        ));
                    } else if !is_compound_op && !Self::types_compatible(&lv_ty, &rv_ty) {
                        self.error(format!(
                            "cannot assign value of type `{}` to target of type `{}`",
                            rv_ty.display(),
                            lv_ty.display()
                        ));
                    }
                }
            }

            ParseNode::If {
                condition,
                then_block,
                else_block,
            } => {
                let ct = self.infer_expr(condition);
                if !matches!(ct, SemType::Boolean | SemType::Unknown) {
                    self.error(format!(
                        "`!if` condition must be `:boolean`, got `{}`",
                        ct.display()
                    ));
                }
                self.scopes.push();
                for stmt in then_block {
                    self.analyze_node(stmt);
                }
                self.scopes.pop();
                if let Some(eb) = else_block {
                    self.scopes.push();
                    for stmt in eb {
                        self.analyze_node(stmt);
                    }
                    self.scopes.pop();
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
                let vt = self.resolve_type_node(var_type);
                if !vt.is_integer() && !matches!(vt, SemType::Unknown) {
                    self.error(format!(
                        "`!for` loop variable must be `:int`, got `{}`",
                        vt.display()
                    ));
                }
                let start_ty = self.infer_expr(start);
                if !matches!(start_ty, SemType::Int | SemType::Unknown) {
                    self.error(format!(
                        "`!for` start expression must be `:int`, got `{}`",
                        start_ty.display()
                    ));
                }
                let stop_ty = self.infer_expr(stop);
                if !matches!(stop_ty, SemType::Int | SemType::Unknown) {
                    self.error(format!(
                        "`!for` stop expression must be `:int`, got `{}`",
                        stop_ty.display()
                    ));
                }
                let step_ty = self.infer_expr(step);
                if !matches!(step_ty, SemType::Int | SemType::Unknown) {
                    self.error(format!(
                        "`!for` step expression must be `:int`, got `{}`",
                        step_ty.display()
                    ));
                }
                self.scopes.push();

                if self.scopes.lookup(var_name).is_some() {
                    self.error(format!(
                        "loop variable `{}` shadows an existing variable in the enclosing scope; \
                         rename the loop variable or the outer variable",
                        var_name
                    ));
                }
                self.declare_sym(Symbol {
                    name: var_name.clone(),
                    kind: SymbolKind::Variable,
                    sem_type: vt,
                    scope_depth: self.scope_depth(),
                    origin: "loop".to_string(),
                });
                self.loop_depth += 1;
                for stmt in body {
                    self.analyze_node(stmt);
                }
                self.loop_depth -= 1;
                self.scopes.pop();
            }

            ParseNode::While { condition, body } => {
                let ct = self.infer_expr(condition);
                if !matches!(ct, SemType::Boolean | SemType::Unknown) {
                    self.error(format!(
                        "`!while` condition must be `:boolean`, got `{}`",
                        ct.display()
                    ));
                }
                self.scopes.push();
                self.loop_depth += 1;
                for stmt in body {
                    self.analyze_node(stmt);
                }
                self.loop_depth -= 1;
                self.scopes.pop();
            }

            ParseNode::Return(expr) => {
                let is_null = matches!(expr.as_ref(), ParseNode::Null);
                let ret_ty = self.infer_expr(expr);
                if let Some(expected) = self.current_return_type.clone() {
                    if matches!(expected, SemType::Void) {
                        if !is_null && !matches!(ret_ty, SemType::Unknown) {
                            self.error(format!(
                                "function returns `:void` but `!return` has an expression of type `{}`; \
                                 use bare `!return !null;` for void functions",
                                ret_ty.display()
                            ));
                        }
                    } else if !is_null && !Self::types_compatible(&expected, &ret_ty) {
                        self.error(format!(
                            "`!return` expression has type `{}`, but function returns `{}`",
                            ret_ty.display(),
                            expected.display()
                        ));
                    }
                } else {
                    self.error("`!return` used outside of a function");
                }
            }

            ParseNode::Exit(expr) => {
                self.infer_expr(expr);
            }

            ParseNode::Break => {
                if self.loop_depth == 0 {
                    self.error("`!break` used outside of a loop");
                }
            }

            ParseNode::Continue => {
                if self.loop_depth == 0 {
                    self.error("`!continue` used outside of a loop");
                }
            }

            ParseNode::ExprStmt(expr) => {
                let ty = self.infer_expr(expr);

                let is_call = matches!(expr.as_ref(), ParseNode::AccessChain { steps, .. }
                    if steps.last().map_or(false, |s| matches!(s, AccessStep::Call(_))));
                if !is_call && !matches!(ty, SemType::Void | SemType::Unknown) {
                    self.warn(format!(
                        "expression result of type `{}` is unused; \
                         did you mean to assign this to a variable?",
                        ty.display()
                    ));
                }
            }

            _ => {}
        }
    }
}

pub fn analyze(root: &ParseNode) -> SemanticResult {
    let mut analyzer = Analyzer::new();
    analyzer.analyze_node(root);
    let mut table = analyzer.all_symbols;

    table.retain(|s| s.origin != "builtin");
    table.sort_by(|a, b| a.scope_depth.cmp(&b.scope_depth).then(a.name.cmp(&b.name)));
    SemanticResult {
        errors: analyzer.errors,
        warnings: analyzer.warnings,
        symbol_table: table,
    }
}
