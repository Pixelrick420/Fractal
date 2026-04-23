use crate::compiler::builtins::{BType, ALL_BUILTINS};
use crate::compiler::parser::{AccessStep, AssignOp, CmpOp, MulOp, ParseNode, UnOp};
use crate::compiler::retcheck::check_function_returns;
use std::collections::HashMap;
use std::fmt;

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

pub fn sem_type_from_btype(bt: &BType) -> SemType {
    match bt {
        BType::Int => SemType::Int,
        BType::Float => SemType::Float,
        BType::Boolean => SemType::Boolean,
        BType::Char => SemType::Char,
        BType::Void => SemType::Void,
        BType::ListOfChar => SemType::List {
            elem: Box::new(SemType::Char),
        },
        BType::Any => SemType::Unknown,
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

fn suggest_similar<'a>(
    query: &str,
    candidates: impl Iterator<Item = &'a String>,
) -> Option<String> {
    if query.len() <= 2 {
        return None;
    }
    let q: Vec<char> = query.chars().collect();
    let mut best: Option<(usize, &'a String)> = None;

    for name in candidates {
        if name.contains("::") && !query.contains("::") {
            continue;
        }

        if name.len() <= 2 {
            continue;
        }
        let n: Vec<char> = name.chars().collect();
        let max_len = q.len().max(n.len());

        let mut dp = vec![vec![0usize; n.len() + 1]; q.len() + 1];
        for i in 0..=q.len() {
            dp[i][0] = i;
        }
        for j in 0..=n.len() {
            dp[0][j] = j;
        }
        for i in 1..=q.len() {
            for j in 1..=n.len() {
                dp[i][j] = if q[i - 1] == n[j - 1] {
                    dp[i - 1][j - 1]
                } else {
                    1 + dp[i - 1][j].min(dp[i][j - 1]).min(dp[i - 1][j - 1])
                };
            }
        }
        let dist = dp[q.len()][n.len()];

        let threshold = if max_len <= 5 { 1 } else { 2 };
        if dist <= threshold {
            if best.map_or(true, |(d, _)| dist < d) {
                best = Some((dist, name));
            }
        }
    }
    best.map(|(_, name)| name.clone())
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

    fn all_names(&self) -> impl Iterator<Item = &String> {
        self.scopes.iter().flat_map(|s| s.keys())
    }

    fn defined_in_current(&self, name: &str) -> bool {
        self.scopes.last().map_or(false, |s| s.contains_key(name))
    }
}

fn format_notes(rest: &str) {
    for raw in rest.lines() {
        let trimmed = raw.trim();

        let text = if let Some(t) = trimmed.strip_prefix("note:") {
            Some(t)
        } else {
            trimmed.strip_prefix("hint:")
        };
        if let Some(t) = text {
            eprintln!(" \x1b[1;34m  =\x1b[0m \x1b[1;32mhint\x1b[0m: {}", t.trim());
        } else if !trimmed.is_empty() {
            eprintln!(" \x1b[1;34m  =\x1b[0m {}", trimmed);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticWarning {
    pub message: String,
    pub line: Option<usize>,
}

impl fmt::Display for SemanticWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lines = self.message.splitn(2, '\n');
        let main = lines.next().unwrap_or(&self.message);
        if let Some(ln) = self.line {
            write!(
                f,
                "\x1b[1;33mWarning\x1b[0m \x1b[1;34m[line {}]\x1b[0m\x1b[1;33m:\x1b[0m {}",
                ln, main
            )
        } else {
            write!(f, "\x1b[1;33mWarning:\x1b[0m {}", main)
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub message: String,
    pub line: Option<usize>,
}

impl SemanticError {
    fn new(msg: impl Into<String>) -> Self {
        SemanticError {
            message: msg.into(),
            line: None,
        }
    }

    fn with_line(msg: impl Into<String>, line: usize) -> Self {
        SemanticError {
            message: msg.into(),
            line: Some(line),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut lines = self.message.splitn(2, '\n');
        let main = lines.next().unwrap_or(&self.message);
        if let Some(ln) = self.line {
            write!(
                f,
                "\x1b[1;31mSemantic Error\x1b[0m \x1b[1;34m[line {}]\x1b[0m\x1b[1;31m:\x1b[0m {}",
                ln, main
            )
        } else {
            write!(f, "\x1b[1;31mSemantic Error:\x1b[0m {}", main)
        }
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
                let rest = w.message.splitn(2, '\n').nth(1).unwrap_or("");
                format_notes(rest);
            }
            println!();
        }
        if !self.errors.is_empty() {
            println!("\x1b[1;31m✗  {} error(s):\x1b[0m", self.errors.len());
            for e in &self.errors {
                eprintln!("   {}", e);
                let rest = e.message.splitn(2, '\n').nth(1).unwrap_or("");
                format_notes(rest);
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

        for b in ALL_BUILTINS {
            scopes.define(Symbol {
                name: b.name.to_string(),
                kind: SymbolKind::Function {
                    params: b.params.iter().map(sem_type_from_btype).collect(),
                },
                sem_type: sem_type_from_btype(&b.ret),
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

    fn error_at(&mut self, line: usize, msg: impl Into<String>) {
        self.errors.push(SemanticError::with_line(msg, line));
    }

    fn warn_at(&mut self, line: usize, msg: impl Into<String>) {
        self.warnings.push(SemanticWarning {
            message: msg.into(),
            line: Some(line),
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
            ParseNode::TypeInt(_) => SemType::Int,
            ParseNode::TypeFloat(_) => SemType::Float,
            ParseNode::TypeChar(_) => SemType::Char,
            ParseNode::TypeBoolean(_) => SemType::Boolean,
            ParseNode::TypeVoid(_) => SemType::Void,
            ParseNode::TypeArray { elem, size, .. } => SemType::Array {
                elem: Box::new(self.resolve_type_node(elem)),
                size: *size,
            },
            ParseNode::TypeList { elem, .. } => SemType::List {
                elem: Box::new(self.resolve_type_node(elem)),
            },
            ParseNode::TypeStruct { name, .. } => SemType::Struct(name.clone()),
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
            ParseNode::IntLit(_, _) => SemType::Int,
            ParseNode::FloatLit(_, _) => SemType::Float,
            ParseNode::CharLit(_, _) => SemType::Char,
            ParseNode::StringLit(s, _) => SemType::Array {
                elem: Box::new(SemType::Char),
                size: s.chars().count() as i64,
            },
            ParseNode::BoolLit(_, _) => SemType::Boolean,
            ParseNode::Null(_) => SemType::Unknown,

            ParseNode::Identifier(name, line) => match self.scopes.lookup(name) {
                Some(sym) => sym.sem_type.clone(),
                None => {
                    let suggestion = suggest_similar(name, self.scopes.all_names());
                    let msg = match suggestion {
                        Some(ref s) => format!(
                            "undefined variable `{}`\nhint: a variable named `{}` is in scope - did you mean `{}`?",
                            name, s, s
                        ),
                        None => format!(
                            "undefined variable `{}`\nnote: the variable must be declared before use with e.g. `:int {} = ...;`",
                            name, name
                        ),
                    };
                    self.error_at(*line, msg);
                    SemType::Unknown
                }
            },

            ParseNode::AccessChain { base, steps, line } => {
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
                        let (t, _base) = match self.scopes.lookup(base) {
                            Some(sym) => (sym.sem_type.clone(), false),
                            None => {
                                let is_bare_call =
                                    steps.len() == 1 && matches!(steps[0], AccessStep::Call(_));
                                if !is_bare_call {
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
                                    let suggestion =
                                        suggest_similar(&qualified, self.scopes.all_names());
                                    let msg = match suggestion {
                                        Some(ref s) => format!(
                                            "undefined identifier `{}`\nhint: did you mean `{}`?",
                                            qualified, s
                                        ),
                                        None => format!("undefined identifier `{}`", qualified),
                                    };
                                    self.error_at(*line, msg);
                                }

                                (SemType::Unknown, true)
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
                                                self.error_at(
                                                    *line,
                                                    format!(
                                                        "struct `{}` has no field `{}`",
                                                        struct_name, field
                                                    ),
                                                );
                                                SemType::Unknown
                                            }
                                        }
                                    }
                                    _ => {
                                        self.error_at(
                                            *line,
                                            format!("undefined struct type `{}`", struct_name),
                                        );
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
                                self.error_at(*line, msg);
                                SemType::Unknown
                            }
                        },
                        AccessStep::Index(idx_expr) => {
                            let idx_ty = self.infer_expr(idx_expr);
                            if !matches!(idx_ty, SemType::Int | SemType::Unknown) {
                                self.error_at(
                                    *line,
                                    format!(
                                        "array/list index must be `:int`, got `{}`",
                                        idx_ty.display()
                                    ),
                                );
                            }

                            let literal_idx: Option<i64> = match idx_expr.as_ref() {
                                ParseNode::IntLit(n, _) => Some(*n),
                                ParseNode::Unary {
                                    op: UnOp::Neg,
                                    operand,
                                    ..
                                } => {
                                    if let ParseNode::IntLit(n, _) = operand.as_ref() {
                                        Some(-n)
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };
                            if let (SemType::Array { size, .. }, Some(idx)) = (&ty, literal_idx) {
                                if idx < 0 || idx >= *size {
                                    self.error_at(
                                        *line,
                                        format!(
                                            "index {} is out of bounds for array of size {} \
                                         (valid indices: 0..{})",
                                            idx,
                                            size,
                                            size - 1
                                        ),
                                    );
                                }
                            }
                            match &ty {
                                SemType::Array { elem, .. } => *elem.clone(),
                                SemType::List { elem } => *elem.clone(),
                                SemType::Unknown => SemType::Unknown,
                                other => {
                                    self.error_at(
                                        *line,
                                        format!("type `{}` is not indexable", other.display()),
                                    );
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
                                    let expected_sig = param_types
                                        .iter()
                                        .map(|t| t.display())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                                    self.error_at(
                                        *line,
                                        format!(
                                            "function `{}` expects {} argument(s) ({}), got {}",
                                            func_name,
                                            param_types.len(),
                                            expected_sig,
                                            arg_types.len()
                                        ),
                                    );
                                }

                                if !variadic {
                                    for (i, (pt, at)) in
                                        param_types.iter().zip(arg_types.iter()).enumerate()
                                    {
                                        if matches!(pt, SemType::Unknown) {
                                            continue;
                                        }
                                        if !Self::types_compatible(pt, at) {
                                            self.error_at(
                                                *line,
                                                format!(
                                                "argument {} of `{}` expects type `{}`, got `{}`",
                                                i + 1,
                                                func_name,
                                                pt.display(),
                                                at.display()
                                            ),
                                            );
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
                                            self.error_at(
                                                *line,
                                                format!(
                                                "`{}` requires a `:list<T>` as its first argument, \
                                                 got `{}`; arrays are fixed-size and cannot be \
                                                 modified with list functions",
                                                func_name,
                                                at.display()
                                            ),
                                            );
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
                                        let value_ty = arg_types.last().unwrap();

                                        let is_insert = func_name == "insert"
                                            || func_name.ends_with("::insert");
                                        if is_insert && arg_types.len() == 3 {
                                            if !matches!(
                                                &arg_types[1],
                                                SemType::Int | SemType::Unknown
                                            ) {
                                                self.error_at(
                                                    *line,
                                                    format!(
                                                    "`insert` index (argument 2) must be `:int`, \
                                                     got `{}`",
                                                    arg_types[1].display()
                                                ),
                                                );
                                            }
                                        }
                                        if !matches!(value_ty, SemType::Unknown)
                                            && !matches!(list_elem.as_ref(), SemType::Unknown)
                                            && !Self::types_compatible(list_elem, value_ty)
                                        {
                                            self.error_at(
                                                *line,
                                                format!(
                                                    "`{}` expects an element of type `{}` \
                                                 (matching the list), but got `{}`",
                                                    func_name,
                                                    list_elem.display(),
                                                    value_ty.display()
                                                ),
                                            );
                                        }
                                    }
                                }

                                let is_find = func_name == "find" || func_name.ends_with("::find");
                                if is_find && arg_types.len() >= 2 {
                                    let search_ty = &arg_types[1];
                                    let elem_ty = match arg_types.first() {
                                        Some(SemType::List { elem }) => Some(elem.as_ref()),
                                        Some(SemType::Array { elem, .. }) => Some(elem.as_ref()),
                                        _ => None,
                                    };
                                    if let Some(elem) = elem_ty {
                                        if !matches!(search_ty, SemType::Unknown)
                                            && !matches!(elem, SemType::Unknown)
                                            && !Self::types_compatible(elem, search_ty)
                                        {
                                            self.error_at(*line, format!(
                                                "`find` searches for a value of type `{}` \
                                                 (matching the collection element type), but got `{}`",
                                                elem.display(),
                                                search_ty.display()
                                            ));
                                        }
                                    }
                                }

                                let is_print = func_name == "print";
                                if is_print {
                                    if let Some(first_arg) = args.first() {
                                        if !matches!(first_arg, ParseNode::StringLit(_, _)) {
                                            self.error_at(*line,
                                                "first argument to `print` must be a string literal; \
                                                 e.g. `print(\"{}\", value)` - a variable of type \
                                                 `:array<:char>` is not accepted as a format string"
                                                    .to_string(),
                                            );
                                        }
                                    }
                                }
                                if is_print && arg_types.len() >= 2 {
                                    for (i, at) in arg_types[1..].iter().enumerate() {
                                        let printable = matches!(
                                            at,
                                            SemType::Int
                                                | SemType::Float
                                                | SemType::Char
                                                | SemType::Boolean
                                                | SemType::Unknown
                                        );
                                        if !printable {
                                            let type_explanation = match at {
                                                SemType::Void => {

                                                    let call_hint = if let Some(arg_expr) = args.get(i + 1) {
                                                        match arg_expr {
                                                            ParseNode::AccessChain { base, steps, .. }
                                                                if steps.last().map_or(false, |s| matches!(s, AccessStep::Call(_))) =>
                                                            {
                                                                format!(
                                                                    "\nnote: `{}(...)` returns `:void` - it produces no value and cannot be printed\n\
                                                                     hint: call `{}(...)` on its own line as a statement, not as an argument to `print`",
                                                                    base, base
                                                                )
                                                            }
                                                            _ => "\nnote: `:void` means the expression produces no value - only functions that return a value can be printed".to_string(),
                                                        }
                                                    } else {
                                                        "\nnote: `:void` means the expression produces no value".to_string()
                                                    };
                                                    format!(
                                                        "`print` argument {} has type `:void`, which cannot be printed\n\
                                                         note: `print` can only format values of type `:int`, `:float`, `:char`, or `:boolean`{}",
                                                        i + 1, call_hint
                                                    )
                                                }
                                                SemType::Struct(sname) => format!(
                                                    "`print` argument {} has type `:struct<{}>`, which cannot be printed directly\n\
                                                     note: structs are composite types - `print` only accepts `:int`, `:float`, `:char`, and `:boolean` values\n\
                                                     hint: access a printable field instead, e.g. `print(\"{{}}\", {}::field_name)`\n\
                                                     hint: to print a numeric field from `{}`, use `{}::field_name` as the argument",
                                                    i + 1, sname,

                                                    if let Some(arg_expr) = args.get(i + 1) {
                                                        match arg_expr {
                                                            ParseNode::AccessChain { base, .. } => base.as_str(),
                                                            _ => "your_var",
                                                        }
                                                    } else { "your_var" },
                                                    sname,
                                                    if let Some(arg_expr) = args.get(i + 1) {
                                                        match arg_expr {
                                                            ParseNode::AccessChain { base, .. } => base.as_str(),
                                                            _ => "your_var",
                                                        }
                                                    } else { "your_var" }
                                                ),
                                                SemType::List { elem: _ } => format!(
                                                    "`print` argument {} has type `{}`, which cannot be printed directly\n\
                                                     note: lists are not scalar values - `print` only accepts `:int`, `:float`, `:char`, and `:boolean`\n\
                                                     hint: to print a specific element, index into it: e.g. `print(\"{{}}\", my_list[0])`\n\
                                                     hint: to print all elements, use a `!for` loop over the list",
                                                    i + 1, at.display()
                                                ),
                                                SemType::Array { elem: _, size } => format!(
                                                    "`print` argument {} has type `{}`, which cannot be printed directly\n\
                                                     note: arrays are not scalar values - `print` only accepts `:int`, `:float`, `:char`, and `:boolean`\n\
                                                     hint: to print a specific element, index into it: e.g. `print(\"{{}}\", my_array[0])`\n\
                                                     hint: to print all {} elements, use a `!for` loop",
                                                    i + 1, at.display(), size
                                                ),
                                                _ => format!(
                                                    "`print` argument {} has type `{}`, which cannot be printed; \
                                                     only `:int`, `:float`, `:char`, and `:boolean` values are printable",
                                                    i + 1,
                                                    at.display()
                                                ),
                                            };
                                            self.error_at(*line, type_explanation);
                                        }
                                    }
                                }

                                let is_pop = func_name == "pop" || func_name.ends_with("::pop");
                                if is_pop {
                                    if let Some(SemType::List { elem }) = arg_types.first() {
                                        return *elem.clone();
                                    }
                                }

                                let is_to_str = func_name == "to_str";
                                if is_to_str {
                                    if let Some(at) = arg_types.first() {
                                        if !matches!(
                                            at,
                                            SemType::Int
                                                | SemType::Float
                                                | SemType::Char
                                                | SemType::Boolean
                                                | SemType::Unknown
                                        ) {
                                            self.error_at(
                                                *line,
                                                format!(
                                                    "`to_str` only works on primitive types \
                                                 (`:int`, `:float`, `:char`, `:boolean`), \
                                                 got `{}`",
                                                    at.display()
                                                ),
                                            );
                                        }
                                    }
                                }

                                let is_abs = func_name == "abs" || func_name.ends_with("::abs");
                                if is_abs {
                                    if let Some(at) = arg_types.first() {
                                        if !matches!(
                                            at,
                                            SemType::Int | SemType::Float | SemType::Unknown
                                        ) {
                                            self.error_at(*line, format!(
                                                "`abs` requires a numeric argument (`:int` or `:float`), \
                                                 got `{}`",
                                                at.display()
                                            ));
                                        }
                                        if matches!(at, SemType::Int | SemType::Float) {
                                            return at.clone();
                                        }
                                    }
                                }

                                let is_minmax = func_name == "min"
                                    || func_name == "max"
                                    || func_name.ends_with("::min")
                                    || func_name.ends_with("::max");
                                if is_minmax && arg_types.len() >= 2 {
                                    let at0 = &arg_types[0];
                                    let at1 = &arg_types[1];
                                    if !matches!(
                                        at0,
                                        SemType::Int | SemType::Float | SemType::Unknown
                                    ) {
                                        self.error_at(
                                            *line,
                                            format!(
                                            "`{}` requires numeric arguments (`:int` or `:float`), \
                                             but argument 1 is `{}`",
                                            func_name, at0.display()
                                        ),
                                        );
                                    } else if !matches!(
                                        at1,
                                        SemType::Int | SemType::Float | SemType::Unknown
                                    ) {
                                        self.error_at(
                                            *line,
                                            format!(
                                            "`{}` requires numeric arguments (`:int` or `:float`), \
                                             but argument 2 is `{}`",
                                            func_name, at1.display()
                                        ),
                                        );
                                    } else if at0.is_numeric() && at1.is_numeric() && at0 != at1 {
                                        self.error_at(
                                            *line,
                                            format!(
                                            "`{}` requires both arguments to be the same type, \
                                             got `{}` and `{}`; use an explicit cast",
                                            func_name,
                                            at0.display(),
                                            at1.display()
                                        ),
                                        );
                                    }
                                    if matches!(at0, SemType::Int | SemType::Float) {
                                        return at0.clone();
                                    }
                                    if matches!(at1, SemType::Int | SemType::Float) {
                                        return at1.clone();
                                    }
                                }

                                let is_len = func_name == "len" || func_name.ends_with("::len");
                                if is_len {
                                    if let Some(at) = arg_types.first() {
                                        if !matches!(
                                            at,
                                            SemType::Array { .. }
                                                | SemType::List { .. }
                                                | SemType::Unknown
                                        ) {
                                            self.error_at(
                                                *line,
                                                format!(
                                                "`len` requires an `:array` or `:list` argument, \
                                                 got `{}`",
                                                at.display()
                                            ),
                                            );
                                        }
                                    }
                                }

                                ret.clone()
                            } else {
                                if self.scopes.lookup(&func_name).is_some() {
                                    self.error_at(*line, format!(
                                        "`{}` is not a function and cannot be called\nnote: `{}` is declared as a variable - did you mean to read its value instead of calling it?",
                                        func_name, func_name
                                    ));
                                } else {
                                    let suggestion =
                                        suggest_similar(&func_name, self.scopes.all_names());
                                    let msg = match suggestion {
                                        Some(ref s) => format!(
                                            "undefined function `{}`\nhint: a name `{}` is in scope - did you mean to call `{}`?",
                                            func_name, s, s
                                        ),
                                        None => format!(
                                            "undefined function `{}`\nnote: make sure the function is defined with `!func {}(...)` before it is called",
                                            func_name, func_name
                                        ),
                                    };
                                    self.error_at(*line, msg);
                                }
                                SemType::Unknown
                            }
                        }
                    };
                }
                ty
            }

            ParseNode::Cast {
                target_type,
                expr,
                line,
            } => {
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
                    self.error_at(
                        *line,
                        format!(
                            "illegal cast from `{}` to `{}`; only these casts are allowed: \
                         `:int`↔`:float`, `:int`↔`:char`, `:int`↔`:boolean`, `:float`↔`:boolean`",
                            src.display(),
                            tgt.display()
                        ),
                    );
                }
                tgt
            }

            ParseNode::ArrayLit(elems, line) => {
                if elems.is_empty() {
                    return SemType::Unknown;
                }
                let elem_types: Vec<SemType> = elems.iter().map(|e| self.infer_expr(e)).collect();
                let elem_ty = elem_types.first().cloned().unwrap_or(SemType::Unknown);
                for (i, t) in elem_types.iter().enumerate() {
                    if !Self::types_compatible(&elem_ty, t) {
                        self.error_at(
                            *line,
                            format!(
                                "array/list literal element {} has type `{}`, expected `{}`",
                                i,
                                t.display(),
                                elem_ty.display()
                            ),
                        );
                    }
                }

                SemType::Array {
                    elem: Box::new(elem_ty),
                    size: elem_types.len() as i64,
                }
            }

            ParseNode::StructLit(fields, _line) => {
                for (_, val) in fields {
                    self.infer_expr(val);
                }
                SemType::Unknown
            }

            ParseNode::LogOr { left, right, line } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Boolean | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("logical operand must be `:boolean`, got `{}`", lt.display()),
                    );
                }
                if !matches!(rt, SemType::Boolean | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("logical operand must be `:boolean`, got `{}`", rt.display()),
                    );
                }
                SemType::Boolean
            }

            ParseNode::LogAnd { left, right, line } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Boolean | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("logical operand must be `:boolean`, got `{}`", lt.display()),
                    );
                }
                if !matches!(rt, SemType::Boolean | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("logical operand must be `:boolean`, got `{}`", rt.display()),
                    );
                }
                SemType::Boolean
            }

            ParseNode::LogNot { operand, line } => {
                let t = self.infer_expr(operand);
                if !matches!(t, SemType::Boolean | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("`!not` operand must be `:boolean`, got `{}`", t.display()),
                    );
                }
                SemType::Boolean
            }

            ParseNode::Cmp {
                left,
                right,
                op,
                line,
            } => {
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
                        self.error_at(
                            *line,
                            format!(
                                "`{}` is not valid for type `{}`; \
                             only `:int`, `:float`, `:char`, and `:boolean` can be compared",
                                op_str,
                                lt.display()
                            ),
                        );
                    } else if !is_simple(&rt) {
                        let op_str = match op {
                            CmpOp::EqEq => "==",
                            CmpOp::Ne => "~=",
                            CmpOp::Gt => ">",
                            CmpOp::Lt => "<",
                            CmpOp::Ge => ">=",
                            CmpOp::Le => "<=",
                        };
                        self.error_at(
                            *line,
                            format!(
                                "`{}` is not valid for type `{}`; \
                             only `:int`, `:float`, `:char`, and `:boolean` can be compared",
                                op_str,
                                rt.display()
                            ),
                        );
                    } else if !Self::types_compatible(&lt, &rt) {
                        let op_str = match op {
                            CmpOp::EqEq => "==",
                            CmpOp::Ne => "~=",
                            CmpOp::Gt => ">",
                            CmpOp::Lt => "<",
                            CmpOp::Ge => ">=",
                            CmpOp::Le => "<=",
                        };
                        self.error_at(
                            *line,
                            format!(
                                "cannot compare `{}` with `{}` using `{}`; \
                             both operands must be the same type",
                                lt.display(),
                                rt.display(),
                                op_str
                            ),
                        );
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
                            self.error_at(
                                *line,
                                format!(
                                    "`{}` is not valid for type `{}`; \
                                 ordering comparisons require `:int`, `:float`, or `:char`",
                                    op_str,
                                    lt.display()
                                ),
                            );
                        }
                    }
                }
                SemType::Boolean
            }

            ParseNode::BitOr { left, right, line } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("bitwise operand must be `:int`, got `{}`", lt.display()),
                    );
                }
                if !matches!(rt, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("bitwise operand must be `:int`, got `{}`", rt.display()),
                    );
                }
                SemType::Int
            }

            ParseNode::BitXor { left, right, line } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("bitwise operand must be `:int`, got `{}`", lt.display()),
                    );
                }
                if !matches!(rt, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("bitwise operand must be `:int`, got `{}`", rt.display()),
                    );
                }
                SemType::Int
            }

            ParseNode::BitAnd { left, right, line } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("bitwise operand must be `:int`, got `{}`", lt.display()),
                    );
                }
                if !matches!(rt, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!("bitwise operand must be `:int`, got `{}`", rt.display()),
                    );
                }
                SemType::Int
            }

            ParseNode::BitShift {
                left, right, line, ..
            } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !matches!(lt, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!(
                            "`<<`/`>>` left operand must be `:int`, got `{}`",
                            lt.display()
                        ),
                    );
                }
                if !matches!(rt, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!(
                            "`<<`/`>>` shift amount must be `:int`, got `{}`",
                            rt.display()
                        ),
                    );
                }
                SemType::Int
            }

            ParseNode::Add {
                left, right, line, ..
            } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if !lt.is_numeric() && !matches!(lt, SemType::Unknown) {
                    self.error_at(*line, format!(
                        "additive operand must be numeric (`:int` or `:float`), got `{}`\nnote: only `:int` and `:float` values support `+` and `-`",
                        lt.display()
                    ));
                }
                if !rt.is_numeric() && !matches!(rt, SemType::Unknown) {
                    self.error_at(*line, format!(
                        "additive operand must be numeric (`:int` or `:float`), got `{}`\nnote: only `:int` and `:float` values support `+` and `-`",
                        rt.display()
                    ));
                }
                if lt.is_numeric() && rt.is_numeric() && lt != rt {
                    self.error_at(*line, format!(
                        "type mismatch in arithmetic: `{}` + `{}` - operands must be the same type\nhint: use an explicit cast: `:{}(expr)` to convert before the operation",
                        lt.display(),
                        rt.display(),
                        if matches!(lt, SemType::Float) { "int" } else { "float" }
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

            ParseNode::Mul {
                left,
                right,
                op,
                line,
            } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                if matches!(op, MulOp::Mod) {
                    if !matches!(lt, SemType::Int | SemType::Unknown) {
                        self.error_at(
                            *line,
                            format!("`%` left operand must be `:int`, got `{}`", lt.display()),
                        );
                    }
                    if !matches!(rt, SemType::Int | SemType::Unknown) {
                        self.error_at(
                            *line,
                            format!("`%` right operand must be `:int`, got `{}`", rt.display()),
                        );
                    }
                    if matches!(rt, SemType::Int) {
                        if let ParseNode::IntLit(val, _) = right.as_ref() {
                            if *val == 0 {
                                self.error_at(*line, "division by zero is not allowed");
                            }
                        }
                    }
                    return if matches!(lt, SemType::Unknown) && matches!(rt, SemType::Unknown) {
                        SemType::Unknown
                    } else {
                        SemType::Int
                    };
                }

                // Check for division by zero (compile-time constant)
                if matches!(op, MulOp::Div) && matches!(rt, SemType::Int) {
                    if let ParseNode::IntLit(val, _) = right.as_ref() {
                        if *val == 0 {
                            self.error_at(*line, "division by zero is not allowed");
                        }
                    }
                }
                if !lt.is_numeric() && !matches!(lt, SemType::Unknown) {
                    self.error_at(*line, format!(
                        "multiplicative operand must be numeric (`:int` or `:float`), got `{}`\nnote: only `:int` and `:float` values support `*`, `/`",
                        lt.display()
                    ));
                }
                if !rt.is_numeric() && !matches!(rt, SemType::Unknown) {
                    self.error_at(*line, format!(
                        "multiplicative operand must be numeric (`:int` or `:float`), got `{}`\nnote: only `:int` and `:float` values support `*`, `/`",
                        rt.display()
                    ));
                }
                if lt.is_numeric() && rt.is_numeric() && lt != rt {
                    self.error_at(*line, format!(
                        "type mismatch in arithmetic: `{}` * `{}` - operands must be the same type\nhint: use an explicit cast: `:{}(expr)` to convert before the operation",
                        lt.display(),
                        rt.display(),
                        if matches!(lt, SemType::Float) { "int" } else { "float" }
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

            ParseNode::Unary { op, operand, line } => {
                let t = self.infer_expr(operand);
                match op {
                    UnOp::BitNot => {
                        if !matches!(t, SemType::Int | SemType::Unknown) {
                            self.error_at(
                                *line,
                                format!("`~` operand must be `:int`, got `{}`", t.display()),
                            );
                        }
                        SemType::Int
                    }
                    UnOp::Neg => {
                        if !t.is_numeric() && !matches!(t, SemType::Unknown) {
                            self.error_at(
                                *line,
                                format!("unary `-` operand must be numeric, got `{}`", t.display()),
                            );
                        }
                        t
                    }
                }
            }

            _ => SemType::Unknown,
        }
    }

    fn validate_struct_lit(&mut self, struct_name: &str, struct_lit: &ParseNode) {
        let (fields, line) = match struct_lit {
            ParseNode::StructLit(fields, line) => (fields, *line),
            _ => {
                self.error("internal error: validate_struct_lit called with non-struct-literal");
                return;
            }
        };
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

                let mut seen_fields: Vec<&str> = Vec::new();
                for (fname, _) in fields {
                    if seen_fields.contains(&fname.as_str()) {
                        self.error_at(
                            line,
                            format!(
                                "struct `{}` initializer has duplicate field `{}`",
                                struct_name, fname
                            ),
                        );
                    } else {
                        seen_fields.push(fname.as_str());
                    }
                }

                for (fname, fval) in fields {
                    match def_fields.iter().find(|(n, _)| n == fname) {
                        None => {
                            self.error_at(
                                line,
                                format!("struct `{}` has no field `{}`", struct_name, fname),
                            );
                        }
                        Some((_, expected_ty)) => {
                            if let (
                                SemType::Struct(ref sub_name),
                                ParseNode::StructLit(sub_fields, _),
                            ) = (expected_ty, fval)
                            {
                                let sub_name = sub_name.clone();
                                let sub_fields = sub_fields.clone();
                                self.validate_struct_lit(
                                    &sub_name,
                                    &ParseNode::StructLit(sub_fields, line),
                                );
                            } else {
                                let actual_ty = self.infer_expr(fval);
                                if !matches!(actual_ty, SemType::Unknown)
                                    && !Self::types_compatible(expected_ty, &actual_ty)
                                {
                                    self.error_at(
                                        line,
                                        format!(
                                            "field `{}` of struct `{}` expects type `{}`, got `{}`",
                                            fname,
                                            struct_name,
                                            expected_ty.display(),
                                            actual_ty.display()
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }

                for (def_name, _) in &def_fields {
                    if !fields.iter().any(|(n, _)| n == def_name) {
                        self.error_at(
                            line,
                            format!(
                                "struct `{}` initializer is missing field `{}`",
                                struct_name, def_name
                            ),
                        );
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

                    if let Some(existing) = self.scopes.lookup(name) {
                        if existing.origin.starts_with("module:") {
                            let module_name =
                                existing.origin.strip_prefix("module:").unwrap_or(name);
                            self.error(format!(
                                "struct name `{}` conflicts with imported module `{}`\n\
                                 hint: a file `{}.fr` was imported and its module is already named `{}`\n\
                                 hint: rename your struct to avoid ambiguity, e.g. `:struct<My{}>  {{ ... }}`",
                                name, module_name, module_name, module_name, name
                            ));
                            continue;
                        }
                    }
                    let mut resolved_fields: Vec<(String, SemType)> = Vec::new();
                    let mut had_field_error = false;
                    for f in fields {
                        if let ParseNode::Field {
                            data_type,
                            name: fname,
                        } = f
                        {
                            let fty = self.resolve_type_node(data_type);

                            if matches!(fty, SemType::Void) {
                                self.error(format!(
                                    "field `{}` of struct `{}` cannot have type `:void`",
                                    fname, name
                                ));
                                had_field_error = true;
                                continue;
                            }

                            if resolved_fields.iter().any(|(n, _)| n == fname) {
                                self.error(format!(
                                    "struct `{}` has duplicate field `{}`",
                                    name, fname
                                ));
                                had_field_error = true;
                                continue;
                            }
                            resolved_fields.push((fname.clone(), fty));
                        }
                    }
                    if !had_field_error || !resolved_fields.is_empty() {
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
                }
                ParseNode::FuncDef {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    if self.scopes.defined_in_current(name) {
                        let is_builtin = self
                            .scopes
                            .lookup(name)
                            .map_or(false, |s| s.origin == "builtin");
                        if is_builtin {
                            self.error(format!(
                                "function `{}` is already defined in this scope\n\
                                 hint: a built-in function named `{}` exists - \
                                 consider renaming your function to avoid the conflict",
                                name, name
                            ));
                        } else {
                            self.error(format!(
                                "function `{}` is already defined in this scope",
                                name
                            ));
                        }
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

                    if !self.scopes.defined_in_current(name) {
                        self.scopes.define(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Variable,
                            sem_type: SemType::Unknown,
                            scope_depth: self.scope_depth(),
                            origin: format!("module:{}", name),
                        });
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

            ParseNode::StructDef { name, .. } => {
                if self.current_return_type.is_some() {
                    self.error(format!(
                        "struct `{}` cannot be defined inside a function; \
                         move it to the top level (outside all `!func` bodies)",
                        name
                    ));
                }
            }

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

                if matches!(ret, SemType::Void) {}

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

                let mut return_errors: Vec<String> = Vec::new();
                check_function_returns(name, return_type, body, &mut return_errors);
                for e in return_errors {
                    self.error(e);
                }
                self.scopes.pop();
                self.current_origin = saved_origin;
                self.current_return_type = prev_ret;
            }

            ParseNode::Decl {
                data_type,
                name,
                init,
                line,
            } => {
                let decl_ty = self.resolve_type_node(data_type);
                if matches!(decl_ty, SemType::Void) {
                    self.error_at(
                        *line,
                        format!(
                            "cannot declare variable `{}` with type `:void`; \
                         `:void` is only valid as a function return type",
                            name
                        ),
                    );
                    return;
                }
                if let SemType::Array { elem, .. } = &decl_ty {
                    if matches!(elem.as_ref(), SemType::Void) {
                        self.error_at(
                            *line,
                            format!("cannot declare array `{}` with element type `:void`", name),
                        );
                        return;
                    }
                }
                if let SemType::List { elem } = &decl_ty {
                    if matches!(elem.as_ref(), SemType::Void) {
                        self.error_at(
                            *line,
                            format!("cannot declare list `{}` with element type `:void`", name),
                        );
                        return;
                    }
                }
                if self.scopes.defined_in_current(name) {
                    self.error_at(*line, format!(
                        "variable `{}` is already declared in this scope\nnote: each variable name must be unique within a block - choose a different name, or remove the duplicate declaration",
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

                if init.is_none() {
                    match &decl_ty {
                        SemType::Array { .. } => self.warn_at(
                            *line,
                            format!(
                                "array `{}` declared without an initialiser; \
                             consider using `= [...]` to give it an explicit value",
                                name
                            ),
                        ),
                        SemType::List { .. } => self.warn_at(
                            *line,
                            format!(
                                "list `{}` declared without an initialiser; \
                             consider using `= [...]` to give it an explicit value",
                                name
                            ),
                        ),
                        SemType::Struct(_) => self.warn_at(
                            *line,
                            format!(
                                "struct variable `{}` declared without an initialiser; \
                             consider using `= {{ ... }}` or `= !null`",
                                name
                            ),
                        ),
                        _ => {}
                    }
                }
                if let Some(init_expr) = init {
                    let is_empty_literal =
                        matches!(init_expr.as_ref(), ParseNode::ArrayLit(e, _) if e.is_empty());
                    if is_empty_literal {
                        if !matches!(
                            decl_ty,
                            SemType::Array { .. } | SemType::List { .. } | SemType::Unknown
                        ) {
                            self.error_at(
                                *line,
                                format!(
                                    "cannot initialise `{}` (type `{}`) with `[]`; \
                                 `[]` is only valid for `:array` and `:list` types",
                                    name,
                                    decl_ty.display()
                                ),
                            );
                        }
                    } else {
                        if matches!(init_expr.as_ref(), ParseNode::Null(_))
                            && !matches!(decl_ty, SemType::Struct(_) | SemType::Unknown)
                        {
                            self.error_at(
                                *line,
                                format!(
                                    "cannot initialise `{}` with `!null`; \
                                 `!null` can only be assigned to struct-type variables",
                                    name
                                ),
                            );
                            return;
                        }
                        let init_ty = self.infer_expr(init_expr);

                        if matches!(init_ty, SemType::Void) {
                            self.error_at(
                                *line,
                                format!(
                                    "cannot initialise `{}` with a `:void` value; \
                                 `:void` functions return no value",
                                    name
                                ),
                            );
                            return;
                        }

                        if matches!(decl_ty, SemType::List { .. })
                            && matches!(init_ty, SemType::Array { .. })
                            && !matches!(init_expr.as_ref(), ParseNode::ArrayLit(_, _))
                        {
                            self.error_at(
                                *line,
                                format!(
                                    "cannot initialise `{}` (type `{}`) with value of type `{}`; \
                                 arrays and lists are distinct types",
                                    name,
                                    decl_ty.display(),
                                    init_ty.display()
                                ),
                            );
                        } else if !Self::types_compatible(&decl_ty, &init_ty) {
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
                            self.error_at(*line, msg);
                        }
                    }
                }
            }

            ParseNode::StructDecl {
                struct_name,
                var_name,
                init,
                line,
            } => {
                let sem_ty = SemType::Struct(struct_name.clone());

                if let Some(existing) = self.scopes.lookup(var_name) {
                    if existing.origin.starts_with("module:") {
                        let module_name =
                            existing.origin.strip_prefix("module:").unwrap_or(var_name);
                        self.error_at(*line, format!(
                            "variable name `{}` conflicts with imported module `{}`\n\
                             hint: the name `{}` was already brought into scope by `!import \"./{}.fr\";`\n\
                             hint: rename your variable to something else, e.g. `my_{}`",
                            var_name, module_name, var_name, module_name, var_name
                        ));
                    }
                }

                if let Some(existing) = self.scopes.lookup(struct_name) {
                    if existing.origin.starts_with("module:") {
                        let module_name = existing
                            .origin
                            .strip_prefix("module:")
                            .unwrap_or(struct_name);
                        self.error_at(*line, format!(
                            "struct type name `{}` conflicts with imported module `{}`\n\
                             hint: the module `{}` was imported and its symbols are accessed as `{}::name`\n\
                             hint: rename your struct to avoid the conflict, e.g. `:struct<My{}>  {{ ... }}`",
                            struct_name, module_name, module_name, module_name, struct_name
                        ));
                    }
                }

                if self.scopes.lookup(struct_name).is_none() {
                    let suggestion = suggest_similar(struct_name, self.scopes.all_names());
                    let msg = match suggestion {
                        Some(ref s) => format!(
                            "undefined struct type `{}`\nhint: a type named `{}` is in scope - did you mean `:struct<{}>`?",
                            struct_name, s, s
                        ),
                        None => format!(
                            "undefined struct type `{}`\nnote: make sure the struct is defined with `:struct<{}> {{ ... }};` before it is used",
                            struct_name, struct_name
                        ),
                    };
                    self.error_at(*line, msg);
                }
                if self.scopes.defined_in_current(var_name) {
                    self.error_at(
                        *line,
                        format!("variable `{}` is already declared in this scope", var_name),
                    );
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
                    if let ParseNode::StructLit(_, _) = init_expr.as_ref() {
                        self.validate_struct_lit(struct_name, init_expr.as_ref());
                    } else if matches!(init_expr.as_ref(), ParseNode::Null(_)) {
                    } else {
                        self.infer_expr(init_expr);
                    }
                }
            }

            ParseNode::Assign {
                lvalue,
                op,
                expr,
                line,
                ..
            } => {
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
                    self.error_at(
                        *line,
                        format!(
                            "{} requires an `:int` target, got `{}`",
                            op_str,
                            lv_ty.display()
                        ),
                    );
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
                    self.error_at(
                        *line,
                        format!(
                            "{} requires a numeric (`:int` or `:float`) target, got `{}`",
                            op_str,
                            lv_ty.display()
                        ),
                    );
                }

                let is_empty_literal =
                    matches!(expr.as_ref(), ParseNode::ArrayLit(e, _) if e.is_empty());
                if is_empty_literal {
                    if !matches!(
                        lv_ty,
                        SemType::Array { .. } | SemType::List { .. } | SemType::Unknown
                    ) {
                        self.error_at(
                            *line,
                            format!(
                                "cannot assign `[]` to `{}`; \
                             `[]` is only valid for `:array` and `:list` types",
                                lv_ty.display()
                            ),
                        );
                    }
                } else {
                    if matches!(expr.as_ref(), ParseNode::Null(_))
                        && !matches!(lv_ty, SemType::Unknown | SemType::Struct(_))
                    {
                        self.error_at(
                            *line,
                            format!(
                                "cannot assign `!null` to `{}`; \
                             `!null` can only be assigned to struct-type variables",
                                lv_ty.display()
                            ),
                        );
                        return;
                    }

                    if matches!(op, AssignOp::Eq) {
                        if let (SemType::Struct(ref sname), ParseNode::StructLit(_, _)) =
                            (&lv_ty, expr.as_ref())
                        {
                            self.validate_struct_lit(sname, expr.as_ref());
                            return;
                        }
                    }
                    let rv_ty = self.infer_expr(expr);

                    if matches!(rv_ty, SemType::Void) {
                        self.error_at(
                            *line,
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
                        self.error_at(
                            *line,
                            format!(
                                "type mismatch in {}: left is `{}`, right is `{}` \
                             - operands must be the same type; use an explicit cast",
                                op_str,
                                lv_ty.display(),
                                rv_ty.display()
                            ),
                        );
                    } else if !is_compound_op && !Self::types_compatible(&lv_ty, &rv_ty) {
                        let cast_hint = if lv_ty.is_numeric() && rv_ty.is_numeric() {
                            format!(
                                "\nhint: use an explicit cast: `:{}(expr)` to convert the value",
                                match &lv_ty {
                                    SemType::Int => "int",
                                    SemType::Float => "float",
                                    _ => "...",
                                }
                            )
                        } else {
                            String::new()
                        };
                        self.error_at(
                            *line,
                            format!(
                                "cannot assign value of type `{}` to target of type `{}`{}",
                                rv_ty.display(),
                                lv_ty.display(),
                                cast_hint
                            ),
                        );
                    } else if !is_compound_op
                        && matches!(lv_ty, SemType::List { .. })
                        && matches!(rv_ty, SemType::Array { .. })
                        && !matches!(expr.as_ref(), ParseNode::ArrayLit(_, _))
                    {
                        self.error_at(
                            *line,
                            format!(
                                "cannot assign value of type `{}` to target of type `{}`; \
                             arrays and lists are distinct types",
                                rv_ty.display(),
                                lv_ty.display()
                            ),
                        );
                    }
                }
            }

            ParseNode::If {
                condition,
                then_block,
                else_block,
                line,
                ..
            } => {
                let ct = self.infer_expr(condition);
                if !matches!(ct, SemType::Boolean | SemType::Unknown) {
                    self.error_at(*line, format!(
                        "`!if` condition must be `:boolean`, got `{}`\nhint: use a comparison operator (`==`, `~=`, `>`, `<`, `>=`, `<=`) to produce a boolean, or wrap it in a truthiness check",
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
                line,
                ..
            } => {
                let vt = self.resolve_type_node(var_type);
                if !vt.is_integer() && !matches!(vt, SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!(
                            "`!for` loop variable must be `:int`, got `{}`",
                            vt.display()
                        ),
                    );
                }
                let start_ty = self.infer_expr(start);
                if !matches!(start_ty, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!(
                            "`!for` start expression must be `:int`, got `{}`",
                            start_ty.display()
                        ),
                    );
                }
                let stop_ty = self.infer_expr(stop);
                if !matches!(stop_ty, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!(
                            "`!for` stop expression must be `:int`, got `{}`",
                            stop_ty.display()
                        ),
                    );
                }
                let step_ty = self.infer_expr(step);
                if !matches!(step_ty, SemType::Int | SemType::Unknown) {
                    self.error_at(
                        *line,
                        format!(
                            "`!for` step expression must be `:int`, got `{}`",
                            step_ty.display()
                        ),
                    );
                }

                if let ParseNode::IntLit(s, _) = step.as_ref() {
                    if *s == 0 {
                        self.error_at(
                            *line,
                            "`!for` step must be non-zero; a step of 0 produces an infinite loop",
                        );
                    }
                }
                self.scopes.push();

                if self.scopes.lookup(var_name).is_some() {
                    self.error_at(
                        *line,
                        format!(
                            "variable `{}` is already declared in an outer scope - \
                            `!for` loop variable cannot shadow an existing variable",
                            var_name
                        ),
                    );
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

            ParseNode::While {
                condition,
                body,
                line,
                ..
            } => {
                let ct = self.infer_expr(condition);
                if !matches!(ct, SemType::Boolean | SemType::Unknown) {
                    self.error_at(*line, format!(
                        "`!while` condition must be `:boolean`, got `{}`\nhint: use a comparison operator (`==`, `~=`, `>`, `<`, `>=`, `<=`) to produce a boolean",
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

            ParseNode::Return { expr, line } => {
                let is_null = matches!(expr.as_ref(), ParseNode::Null(_));
                let ret_ty = self.infer_expr(expr);
                if let Some(expected) = self.current_return_type.clone() {
                    if matches!(expected, SemType::Void) {
                        if !is_null && !matches!(ret_ty, SemType::Unknown) {
                            self.error_at(*line, format!(
                                "function returns `:void` but `!return` has an expression of type `{}`; \
                                 use bare `!return !null;` for void functions",
                                ret_ty.display()
                            ));
                        }
                    } else if is_null {
                        if !matches!(expected, SemType::Struct(_)) {
                            self.error_at(*line, format!(
                                "cannot return `!null` from a function that returns `{}`; \
                                 `!null` is only valid as a return value for struct-returning functions",
                                expected.display()
                            ));
                        }
                    } else if !Self::types_compatible(&expected, &ret_ty) {
                        self.error_at(
                            *line,
                            format!(
                                "`!return` expression has type `{}`, but function returns `{}`",
                                ret_ty.display(),
                                expected.display()
                            ),
                        );
                    }
                } else {
                    self.error_at(*line, "`!return` used outside of a function\nnote: `!return` can only appear inside a `!func` body - did you accidentally place it at the top level?");
                }
            }

            ParseNode::Exit { expr, line: _ } => {
                self.infer_expr(expr);
            }

            ParseNode::Break { line } => {
                if self.loop_depth == 0 {
                    self.error_at(*line, "`!break` used outside of a loop\nnote: `!break` can only appear inside a `!for` or `!while` body");
                }
            }

            ParseNode::Continue { line } => {
                if self.loop_depth == 0 {
                    self.error_at(*line, "`!continue` used outside of a loop\nnote: `!continue` can only appear inside a `!for` or `!while` body");
                }
            }

            ParseNode::ExprStmt(expr, line) => {
                let ty = self.infer_expr(expr);

                let is_call = matches!(expr.as_ref(), ParseNode::AccessChain { steps, .. }
                    if steps.last().map_or(false, |s| matches!(s, AccessStep::Call(_))));
                if !is_call && !matches!(ty, SemType::Void | SemType::Unknown) {
                    self.warn_at(*line, format!(
                        "expression result of type `{}` is unused\nhint: assign it to a variable with `:{}  name = ...;`, or remove the expression if it has no side effects",
                        ty.display(),
                        match &ty {
                            SemType::Int => "int",
                            SemType::Float => "float",
                            SemType::Char => "char",
                            SemType::Boolean => "boolean",
                            _ => "...",
                        }
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
