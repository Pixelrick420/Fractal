use crate::compiler::parser::{AccessStep, AddOp, AssignOp, CmpOp, MulOp, ParseNode, UnOp};
use crate::compiler::semanter::{SemType, SymbolKind, SemanticResult};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a single flat Rust source file from a fully-analysed Fractal parse
/// tree.
///
/// * `root` – the `ParseNode::Program` returned by the parser
/// * `sem`  – the `SemanticResult` returned by the analyser; its symbol table
///            drives all struct-field `Option`-wrapping decisions
pub fn generate(root: &ParseNode, sem: &SemanticResult) -> String {
    let mut cg = CodeGen::new(sem);
    cg.gen_root(root);
    cg.buf
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal state
// ─────────────────────────────────────────────────────────────────────────────

struct CodeGen {
    buf: String,
    indent: usize,

    /// variable-name → SemType  (flat; last definition wins for shadowed names
    /// — sufficient because the analyser rejects variable shadowing)
    var_types: HashMap<String, SemType>,

    /// struct-name → [(field_name, SemType)]
    struct_fields: HashMap<String, Vec<(String, SemType)>>,

    /// Names of known module blocks — used to choose `::` vs `.` separator.
    module_names: std::collections::HashSet<String>,

    /// Return-type of the function currently being generated (struct name, if
    /// the return type is a struct).  Used to name anonymous struct literals
    /// inside `!return` statements.
    current_return_struct: Option<String>,
}

impl CodeGen {
    fn new(sem: &SemanticResult) -> Self {
        let mut var_types     = HashMap::new();
        let mut struct_fields = HashMap::new();

        for sym in &sem.symbol_table {
            match &sym.kind {
                SymbolKind::Variable => {
                    var_types.insert(sym.name.clone(), sym.sem_type.clone());
                }
                SymbolKind::Struct { fields } => {
                    struct_fields.insert(sym.name.clone(), fields.clone());
                }
                _ => {}
            }
        }

        CodeGen {
            buf: String::new(),
            indent: 0,
            var_types,
            struct_fields,
            module_names: std::collections::HashSet::new(),
            current_return_struct: None,
        }
    }

    // ── low-level output ──────────────────────────────────────────────────────

    /// Emit one indented line (appends `\n`).
    fn line(&mut self, s: &str) {
        let pad = "    ".repeat(self.indent);
        self.buf.push_str(&pad);
        self.buf.push_str(s);
        self.buf.push('\n');
    }

    /// Emit raw text (no indentation, no trailing newline).
    fn raw(&mut self, s: &str) {
        self.buf.push_str(s);
    }

    fn blank(&mut self) {
        self.buf.push('\n');
    }

    fn indent(&mut self) {
        self.indent += 1;
    }
    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    // ── type helpers: ParseNode → Rust type string ────────────────────────────

    /// Plain Rust type — used for variable declarations, params, etc.
    fn type_str(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeInt              => "i64".into(),
            ParseNode::TypeFloat            => "f64".into(),
            ParseNode::TypeChar             => "char".into(),
            ParseNode::TypeBoolean          => "bool".into(),
            ParseNode::TypeVoid             => "()".into(),
            ParseNode::TypeArray { elem, size } =>
                format!("[{}; {}]", self.type_str(elem), size),
            ParseNode::TypeList  { elem }   => format!("Vec<{}>", self.type_str(elem)),
            ParseNode::TypeStruct { name }  => name.clone(),
            _                               => "/* ? */".into(),
        }
    }

    /// Type for a *struct field*: every field is `Option`-wrapped.
    /// Struct-type fields get an extra `Box` to break potential size cycles.
    fn field_type_str(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeStruct { name } => format!("Option<Box<{}>>", name),
            _                              => format!("Option<{}>", self.type_str(node)),
        }
    }

    /// `-> T` suffix for a function signature; empty for `:void`.
    /// Struct return types become `Option<Box<S>>` so `!return !null` compiles.
    fn ret_type_str(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeVoid            => String::new(),
            ParseNode::TypeStruct { name } => format!(" -> Option<Box<{}>>", name),
            _                              => format!(" -> {}", self.type_str(node)),
        }
    }

    /// Zero-initialiser expression for a given type.
    fn zero_val(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeInt              => "0_i64".into(),
            ParseNode::TypeFloat            => "0.0_f64".into(),
            ParseNode::TypeChar             => "'\\0'".into(),
            ParseNode::TypeBoolean          => "false".into(),
            ParseNode::TypeArray { elem, size } =>
                format!("[{}; {}]", self.zero_val(elem), size),
            ParseNode::TypeList  { .. }     => "Vec::new()".into(),
            ParseNode::TypeStruct { .. }    => "None".into(),
            _                               => "Default::default()".into(),
        }
    }

    // ── SemType → Rust type (for access-chain type tracking) ─────────────────

    fn sem_rs(t: &SemType) -> String {
        match t {
            SemType::Int      => "i64".into(),
            SemType::Float    => "f64".into(),
            SemType::Char     => "char".into(),
            SemType::Boolean  => "bool".into(),
            SemType::Void     => "()".into(),
            SemType::Array { elem, size } =>
                format!("[{}; {}]", Self::sem_rs(elem), size),
            SemType::List  { elem }       => format!("Vec<{}>", Self::sem_rs(elem)),
            SemType::Struct(n)            => n.clone(),
            SemType::Unknown              => "_".into(),
        }
    }

    // ── top-level program ─────────────────────────────────────────────────────

    fn gen_root(&mut self, root: &ParseNode) {
        let items = match root {
            ParseNode::Program(v) => v,
            _ => return,
        };

        // Collect module names upfront so :: vs . is resolved correctly.
        for item in items {
            if let ParseNode::Module { name, .. } = item {
                self.module_names.insert(name.clone());
            }
        }

        self.line("#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]");
        self.line("use std::io::{self, BufRead, Write};");
        self.blank();

        // Hoist definitions before fn main().
        let (defs, stmts): (Vec<_>, Vec<_>) = items.iter().partition(|n| {
            matches!(
                n,
                ParseNode::FuncDef  { .. }
                | ParseNode::StructDef { .. }
                | ParseNode::Module    { .. }
            )
        });

        for def in &defs {
            self.gen_item(def);
            self.blank();
        }

        if !stmts.is_empty() {
            self.line("fn main() {");
            self.indent();
            for s in &stmts {
                self.gen_stmt(s);
            }
            self.dedent();
            self.line("}");
        }
    }

    // ── item dispatch ─────────────────────────────────────────────────────────

    fn gen_item(&mut self, node: &ParseNode) {
        match node {
            ParseNode::FuncDef  { name, params, return_type, body } =>
                self.gen_funcdef(name, params, return_type, body),
            ParseNode::StructDef { name, fields } =>
                self.gen_structdef(name, fields),
            ParseNode::Module    { name, items }  =>
                self.gen_module(name, items),
            _ => self.gen_stmt(node),
        }
    }

    // ── struct definition ─────────────────────────────────────────────────────

    fn gen_structdef(&mut self, name: &str, fields: &[ParseNode]) {
        self.line("#[derive(Debug, Clone, Default)]");
        self.line(&format!("pub struct {} {{", name));
        self.indent();
        for f in fields {
            if let ParseNode::Field { data_type, name: fname } = f {
                self.line(&format!(
                    "pub {}: {},",
                    fname,
                    self.field_type_str(data_type)
                ));
            }
        }
        self.dedent();
        self.line("}");
    }

    // ── function definition ───────────────────────────────────────────────────

    fn gen_funcdef(
        &mut self,
        name:        &str,
        params:      &[ParseNode],
        return_type: &ParseNode,
        body:        &[ParseNode],
    ) {
        // Track the struct return type so that StructLit nodes inside `!return`
        // can be emitted with the correct struct name.
        let prev_ret = self.current_return_struct.take();
        if let ParseNode::TypeStruct { name: sname } = return_type {
            self.current_return_struct = Some(sname.clone());
        }

        let params_str: Vec<String> = params
            .iter()
            .filter_map(|p| match p {
                ParseNode::Param { data_type, name: pname } => {
                    // Struct params are passed as Option<Box<S>>.
                    let ty = match data_type.as_ref() {
                        ParseNode::TypeStruct { name: sname } =>
                            format!("Option<Box<{}>>", sname),
                        _ => self.type_str(data_type),
                    };
                    Some(format!("mut {}: {}", pname, ty))
                }
                _ => None,
            })
            .collect();

        let ret = self.ret_type_str(return_type);
        let visibility = if name == "main" { "" } else { "pub " };
        self.line(&format!("{}fn {}({}){} {{", visibility, name, params_str.join(", "), ret));
        self.indent();
        for s in body {
            self.gen_stmt(s);
        }
        self.dedent();
        self.line("}");

        self.current_return_struct = prev_ret;
    }

    // ── module → `pub mod name { use super::*; … }` ───────────────────────────

    fn gen_module(&mut self, name: &str, items: &[ParseNode]) {
        self.module_names.insert(name.to_string());

        self.line(&format!("pub mod {} {{", name));
        self.indent();
        self.line("use super::*;");
        self.blank();

        let (defs, stmts): (Vec<_>, Vec<_>) = items.iter().partition(|n| {
            matches!(
                n,
                ParseNode::FuncDef  { .. }
                | ParseNode::StructDef { .. }
                | ParseNode::Module    { .. }
            )
        });

        for d in &defs {
            self.gen_item(d);
            self.blank();
        }

        // Non-definition items inside a module body are gathered into an
        // `init()` function; the caller is responsible for invoking it.
        if !stmts.is_empty() {
            self.line("pub fn init() {");
            self.indent();
            for s in &stmts {
                self.gen_stmt(s);
            }
            self.dedent();
            self.line("}");
        }

        self.dedent();
        self.line("}");
    }

    // ── statements ────────────────────────────────────────────────────────────

    fn gen_stmt(&mut self, node: &ParseNode) {
        match node {
            ParseNode::Decl { data_type, name, init } =>
                self.gen_decl(data_type, name, init.as_deref()),

            ParseNode::StructDecl { struct_name, var_name, init } =>
                self.gen_struct_decl(struct_name, var_name, init.as_deref()),

            ParseNode::Assign { lvalue, op, expr } =>
                self.gen_assign(lvalue, op, expr),

            ParseNode::If { condition, then_block, else_block } =>
                self.gen_if(condition, then_block, else_block.as_deref()),

            ParseNode::For { var_type, var_name, start, stop, step, body } =>
                self.gen_for(var_type, var_name, start, stop, step, body),

            ParseNode::While { condition, body } => {
                let c = self.gen_expr(condition);
                self.line(&format!("while {} {{", c));
                self.indent();
                for s in body { self.gen_stmt(s); }
                self.dedent();
                self.line("}");
            }

            ParseNode::Return(expr) => {
                match expr.as_ref() {
                    ParseNode::Null => {
                        self.line("return None;");
                    }
                    ParseNode::StructLit(fields) => {
                        if let Some(sname) = self.current_return_struct.clone() {
                            let body = self.emit_struct_lit_body(&sname.clone(), fields);
                            self.line(&format!(
                                "return Some(Box::new({} {{ {} }}));",
                                sname, body
                            ));
                        } else {
                            // Struct name unknown at this point — emit with a placeholder.
                            let parts: Vec<_> = fields.iter().map(|(f, e)| {
                                let v = self.gen_expr(e);
                                format!("{}: Some({})", f, v)
                            }).collect();
                            self.line(&format!(
                                "return Some(Box::new(/* StructName */ {{ {} }}));",
                                parts.join(", ")
                            ));
                        }
                    }
                    _ => {
                        let e = self.gen_expr(expr);
                        self.line(&format!("return {};", e));
                    }
                }
            }

            ParseNode::Exit(expr) => {
                let e = self.gen_expr(expr);
                self.line(&format!("std::process::exit({} as i32);", e));
            }

            ParseNode::Break    => self.line("break;"),
            ParseNode::Continue => self.line("continue;"),

            ParseNode::ExprStmt(e) => {
                let s = self.gen_expr(e);
                self.line(&format!("{};", s));
            }

            // Definitions may appear inside function bodies.
            ParseNode::FuncDef  { name, params, return_type, body } =>
                self.gen_funcdef(name, params, return_type, body),
            ParseNode::StructDef { name, fields } =>
                self.gen_structdef(name, fields),

            _ => {}
        }
    }

    // ── variable declaration ──────────────────────────────────────────────────

    fn gen_decl(&mut self, data_type: &ParseNode, name: &str, init: Option<&ParseNode>) {
        // TypeStruct declarations are handled exactly like explicit StructDecl nodes.
        if let ParseNode::TypeStruct { name: sname } = data_type {
            let sname = sname.clone();
            self.gen_struct_decl(&sname, name, init);
            return;
        }

        let ty  = self.type_str(data_type);
        let rhs = match init {
            Some(e) => self.gen_expr(e),
            None    => self.zero_val(data_type),
        };
        self.line(&format!("let mut {}: {} = {};", name, ty, rhs));
    }

    // ── struct variable declaration  (Option<Box<S>>) ─────────────────────────

    fn gen_struct_decl(
        &mut self,
        struct_name: &str,
        var_name:    &str,
        init:        Option<&ParseNode>,
    ) {
        let rhs = match init {
            None                              => "None".to_string(),
            Some(ParseNode::Null)             => "None".to_string(),
            Some(ParseNode::StructLit(fields)) => {
                let sn   = struct_name.to_string();
                let body = self.emit_struct_lit_body(&sn, fields);
                format!("Some(Box::new({} {{ {} }}))", struct_name, body)
            }
            Some(other) => {
                // Another struct variable, a function call returning Option<Box<S>>, etc.
                // The value is already Option<Box<S>> — use it directly.
                self.gen_expr(other)
            }
        };
        self.line(&format!(
            "let mut {}: Option<Box<{}>> = {};",
            var_name, struct_name, rhs
        ));
    }

    /// Emit the field list of a struct literal as `field: Some(expr), …`.
    /// Struct-typed fields get an extra `Box::new(…)`.
    fn emit_struct_lit_body(
        &mut self,
        struct_name: &str,
        fields:      &[(String, ParseNode)],
    ) -> String {
        // Build a local field-type lookup so we know which fields need Box.
        let ftypes: HashMap<String, SemType> = self
            .struct_fields
            .get(struct_name)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect();

        fields
            .iter()
            .map(|(fname, fexpr)| {
                if matches!(fexpr, ParseNode::Null) {
                    return format!("{}: None", fname);
                }
                let val = self.gen_expr(fexpr);
                let wrapped = match ftypes.get(fname) {
                    Some(SemType::Struct(_)) => format!("Some(Box::new({}))", val),
                    _                        => format!("Some({})", val),
                };
                format!("{}: {}", fname, wrapped)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    // ── assignment ────────────────────────────────────────────────────────────
    //
    // The last step of an lvalue may be a struct field (Option<T> or
    // Option<Box<S>>).  If so we must emit `= Some(rhs)` / `= None` instead of
    // `= rhs`, and compound operators (`+=` etc.) must deref through the Option.

    fn gen_assign(&mut self, lvalue: &ParseNode, op: &AssignOp, expr: &ParseNode) {
        if let ParseNode::AccessChain { base, steps } = lvalue {
            if let Some(AccessStep::Field(fname)) = steps.last() {
                let fname  = fname.clone();
                let is_null = matches!(expr, ParseNode::Null);
                let prefix  = self.emit_struct_field_prefix(base, steps);

                if matches!(op, AssignOp::Eq) {
                    let rhs = if is_null {
                        "None".to_string()
                    } else if let ParseNode::StructLit(fields) = expr {
                        // Inline struct literal: need the target field's struct name.
                        if let Some(SemType::Struct(sname)) =
                            self.resolve_field_type(base, steps, &fname)
                        {
                            let sname = sname.clone();
                            let body  = self.emit_struct_lit_body(&sname, fields);
                            format!("Some(Box::new({} {{ {} }}))", sname, body)
                        } else {
                            let parts: Vec<_> = fields.iter().map(|(f, e)| {
                                let v = self.gen_expr(e);
                                format!("{}: Some({})", f, v)
                            }).collect();
                            format!("Some(/* StructName */ {{ {} }})", parts.join(", "))
                        }
                    } else {
                        let val       = self.gen_expr(expr);
                        let needs_box = matches!(
                            self.resolve_field_type(base, steps, &fname),
                            Some(SemType::Struct(_))
                        );
                        if needs_box {
                            format!("Some(Box::new({}))", val)
                        } else {
                            format!("Some({})", val)
                        }
                    };
                    self.line(&format!("{}.{} = {};", prefix, fname, rhs));
                } else {
                    // Compound operator on an Option<T> field:
                    //   *prefix.field.as_mut().unwrap() += rhs
                    let op_str = Self::assign_op_str(op);
                    let rhs    = self.gen_expr(expr);
                    self.line(&format!(
                        "*{}.{}.as_mut().unwrap() {} {};",
                        prefix, fname, op_str, rhs
                    ));
                }
                return;
            }
        }

        // Plain variable or index lvalue.
        let lv     = self.gen_lvalue(lvalue);
        let op_str = Self::assign_op_str(op);
        let rv     = self.gen_expr(expr);
        self.line(&format!("{} {} {};", lv, op_str, rv));
    }

    /// Return the mutable access path to the struct that *owns* the final field.
    ///
    /// Example  (steps = [Field("val")]):
    ///   n is Option<Box<Node>>  →  "n.as_mut().unwrap()"
    ///
    /// Example  (steps = [Field("next"), Field("val")]):
    ///   →  "n.as_mut().unwrap().next.as_mut().unwrap()"
    fn emit_struct_field_prefix(&mut self, base: &str, steps: &[AccessStep]) -> String {
        let prefix_steps = &steps[..steps.len().saturating_sub(1)];
        let base_is_struct = matches!(self.var_types.get(base), Some(SemType::Struct(_)));

        let mut out = base.to_string();
        if base_is_struct {
            out = format!("{}.as_mut().unwrap()", out);
        }

        for step in prefix_steps {
            match step {
                AccessStep::Field(f) => {
                    out = format!("{}.{}.as_mut().unwrap()", out, f);
                }
                AccessStep::Index(e) => {
                    let idx = self.gen_expr(e);
                    out = format!("{}[{} as usize]", out, idx);
                }
                AccessStep::Call(args) => {
                    let av: Vec<_> = args.iter().map(|a| self.gen_expr(a)).collect();
                    out = format!("{}({})", out, av.join(", "));
                }
            }
        }
        out
    }

    /// Walk the type chain to return the SemType of the last field.
    fn resolve_field_type(
        &self,
        base:  &str,
        steps: &[AccessStep],
        fname: &str,
    ) -> Option<SemType> {
        let mut cur = self.var_types.get(base).cloned();
        for step in &steps[..steps.len().saturating_sub(1)] {
            cur = match (cur, step) {
                (Some(SemType::Struct(sname)), AccessStep::Field(f)) => {
                    self.struct_fields
                        .get(&sname)
                        .and_then(|fs| fs.iter().find(|(n, _)| n == f))
                        .map(|(_, t)| t.clone())
                }
                _ => None,
            };
        }
        if let Some(SemType::Struct(sname)) = cur {
            self.struct_fields
                .get(&sname)
                .and_then(|fs| fs.iter().find(|(n, _)| n == fname))
                .map(|(_, t)| t.clone())
        } else {
            None
        }
    }

    fn gen_lvalue(&mut self, node: &ParseNode) -> String {
        match node {
            ParseNode::AccessChain { base, steps } =>
                self.emit_access_chain(base, steps),
            _ => self.gen_expr(node),
        }
    }

    fn assign_op_str(op: &AssignOp) -> &'static str {
        match op {
            AssignOp::Eq        => "=",
            AssignOp::PlusEq    => "+=",
            AssignOp::MinusEq   => "-=",
            AssignOp::StarEq    => "*=",
            AssignOp::SlashEq   => "/=",
            AssignOp::PercentEq => "%=",
            AssignOp::AmpEq     => "&=",
            AssignOp::PipeEq    => "|=",
            AssignOp::CaretEq   => "^=",
        }
    }

    // ── if / elif / else ──────────────────────────────────────────────────────

    fn gen_if(
        &mut self,
        cond:      &ParseNode,
        then_blk:  &[ParseNode],
        else_blk:  Option<&[ParseNode]>,
    ) {
        let c = self.gen_expr(cond);
        self.line(&format!("if {} {{", c));
        self.indent();
        for s in then_blk { self.gen_stmt(s); }
        self.dedent();
        self.close_if_chain(else_blk);
    }

    /// Emit the closing brace(s) of an if chain, handling elif desugaring.
    fn close_if_chain(&mut self, else_blk: Option<&[ParseNode]>) {
        match else_blk {
            None => self.line("}"),
            Some(eb) => {
                // `!elif` desugars as a single nested If inside the else block.
                if let [ParseNode::If {
                    condition:  ec,
                    then_block: et,
                    else_block: ee,
                }] = eb
                {
                    let ec_s = self.gen_expr(ec);
                    let pad  = "    ".repeat(self.indent);
                    self.raw(&format!("{}}} else if {} {{\n", pad, ec_s));
                    self.indent();
                    for s in et { self.gen_stmt(s); }
                    self.dedent();
                    self.close_if_chain(ee.as_deref());
                } else {
                    let pad = "    ".repeat(self.indent);
                    self.raw(&format!("{}}} else {{\n", pad));
                    self.indent();
                    for s in eb { self.gen_stmt(s); }
                    self.dedent();
                    self.line("}");
                }
            }
        }
    }

    // ── for loop ──────────────────────────────────────────────────────────────
    //
    // `!for (:int i, start, stop, step) { … }` →
    //   { let mut i: i64 = start; while i < stop { … i += step; } }
    //
    // Wrapped in a block so the loop variable is scoped, matching Fractal semantics.

    fn gen_for(
        &mut self,
        var_type: &ParseNode,
        var_name: &str,
        start:    &ParseNode,
        stop:     &ParseNode,
        step:     &ParseNode,
        body:     &[ParseNode],
    ) {
        let ty   = self.type_str(var_type);
        let s_s  = self.gen_expr(start);
        let st_s = self.gen_expr(stop);
        let sp_s = self.gen_expr(step);

        self.line("{");
        self.indent();
        self.line(&format!("let mut {}: {} = {};", var_name, ty, s_s));
        self.line(&format!("while {} < {} {{", var_name, st_s));
        self.indent();
        for stmt in body { self.gen_stmt(stmt); }
        self.line(&format!("{} += {};", var_name, sp_s));
        self.dedent();
        self.line("}");
        self.dedent();
        self.line("}");
    }

    // ── expressions ───────────────────────────────────────────────────────────

    fn gen_expr(&mut self, node: &ParseNode) -> String {
        match node {
            ParseNode::IntLit(v)    => format!("{}_i64", v),
            ParseNode::FloatLit(v)  => {
                // {:?} always emits a decimal point for f64.
                format!("{:?}_f64", v)
            }
            ParseNode::CharLit(c)   => format!("'{}'", escape_char(*c)),
            ParseNode::StringLit(s) => {
                // Emit as a Rust String literal. When a :list<:char> is needed
                // the semanter will have guided usage, but the raw literal form
                // is a String — compatible with print and all string contexts.
                let escaped: String = s.chars().map(|c| escape_char(c)).collect();
                format!("\"{}\".to_string()", escaped)
            }
            ParseNode::BoolLit(b)   => if *b { "true".into() } else { "false".into() },
            ParseNode::Null         => "None".into(),

            ParseNode::Identifier(n) => n.clone(),

            ParseNode::AccessChain { base, steps } =>
                self.emit_access_chain(base, steps),

            ParseNode::ArrayLit(elems) => {
                let parts: Vec<_> = elems.iter().map(|e| self.gen_expr(e)).collect();
                format!("[{}]", parts.join(", "))
            }

            // Struct literal without a named context (e.g. passed as a function
            // argument).  We emit with a placeholder so Rust gives a clear error
            // instead of a cryptic one — the user replaces /* StructName */.
            ParseNode::StructLit(fields) => {
                let parts: Vec<_> = fields.iter().map(|(fname, fexpr)| {
                    let val = if matches!(fexpr, ParseNode::Null) {
                        "None".to_string()
                    } else {
                        format!("Some({})", self.gen_expr(fexpr))
                    };
                    format!("{}: {}", fname, val)
                }).collect();
                format!("/* StructName */ {{ {} }}", parts.join(", "))
            }

            ParseNode::LogOr  { left, right } =>
                format!("({} || {})", self.gen_expr(left), self.gen_expr(right)),
            ParseNode::LogAnd { left, right } =>
                format!("({} && {})", self.gen_expr(left), self.gen_expr(right)),
            ParseNode::LogNot { operand }      =>
                format!("(!{})", self.gen_expr(operand)),

            ParseNode::Cmp { left, op, right } => {
                let op_s = match op {
                    CmpOp::Gt   => ">",
                    CmpOp::Lt   => "<",
                    CmpOp::Ge   => ">=",
                    CmpOp::Le   => "<=",
                    CmpOp::EqEq => "==",
                    CmpOp::Ne   => "!=",
                };
                format!("({} {} {})", self.gen_expr(left), op_s, self.gen_expr(right))
            }

            ParseNode::BitOr  { left, right } =>
                format!("({} | {})",  self.gen_expr(left), self.gen_expr(right)),
            ParseNode::BitXor { left, right } =>
                format!("({} ^ {})",  self.gen_expr(left), self.gen_expr(right)),
            ParseNode::BitAnd { left, right } =>
                format!("({} & {})",  self.gen_expr(left), self.gen_expr(right)),

            ParseNode::Add { left, op, right } => {
                let op_s = match op { AddOp::Add => "+", AddOp::Sub => "-" };
                format!("({} {} {})", self.gen_expr(left), op_s, self.gen_expr(right))
            }
            ParseNode::Mul { left, op, right } => {
                let op_s = match op {
                    MulOp::Mul => "*",
                    MulOp::Div => "/",
                    MulOp::Mod => "%",
                };
                format!("({} {} {})", self.gen_expr(left), op_s, self.gen_expr(right))
            }

            ParseNode::Unary { op, operand } => {
                let op_s = match op { UnOp::Neg => "-", UnOp::BitNot => "!" };
                format!("({}{})", op_s, self.gen_expr(operand))
            }

            ParseNode::Cast { target_type, expr } => {
                let ty = self.type_str(target_type);
                let e  = self.gen_expr(expr);
                format!("({} as {})", e, ty)
            }

            _ => "/* unsupported expr */".into(),
        }
    }

    // ── access chain ──────────────────────────────────────────────────────────
    //
    // Struct variables in Rust are  Option<Box<S>>.
    // Struct fields    in Rust are  Option<T>  or  Option<Box<S>>.
    //
    // Reading rules applied as we walk the chain:
    //
    //  • base var of struct type               → base.as_ref().unwrap()
    //  • primitive / array / list field (last) → .field.unwrap()
    //  • struct field (last, rvalue)           → .field.clone()   (Option<Box<S>>)
    //  • struct field (not last)               → .field.as_ref().unwrap()
    //
    // Module-qualified access (math::sin, mymod::func):
    //  • base not in var_types  AND first step is Field
    //    → use `::` separator (Rust module path)

    fn emit_access_chain(&mut self, base: &str, steps: &[AccessStep]) -> String {
        // ── builtins (direct call or module-scoped call) ──────────────────────
        if steps.len() == 1 {
            if let AccessStep::Call(args) = &steps[0] {
                if let Some(s) = self.try_builtin(base, args) {
                    return s;
                }
            }
        }
        // e.g.  math::sin(x)  →  AccessChain { base:"math", [Field("sin"), Call([x])] }
        if steps.len() >= 2 {
            if let AccessStep::Field(qualified_name) = &steps[0] {
                // base is a module; qualified function name is base::qualified_name
                let full = format!("{}::{}", base, qualified_name);
                if !self.var_types.contains_key(base) {
                    // Reconstruct the full call using the remaining steps.
                    let mut out = full;
                    for step in &steps[1..] {
                        match step {
                            AccessStep::Call(args) => {
                                let av: Vec<_> = args.iter().map(|a| self.gen_expr(a)).collect();
                                // Check if this is a known builtin under the module.
                                if let Some(s) = self.try_builtin(qualified_name, args) {
                                    return format!("{}::{}", base, s);
                                }
                                out = format!("{}({})", out, av.join(", "));
                            }
                            AccessStep::Field(f)   => out = format!("{}::{}", out, f),
                            AccessStep::Index(e)   => {
                                let idx = self.gen_expr(e);
                                out = format!("{}[{} as usize]", out, idx);
                            }
                        }
                    }
                    return out;
                }
            }
        }

        // ── type-tracking walk ────────────────────────────────────────────────
        let base_sem        = self.var_types.get(base).cloned();
        let base_is_struct  = matches!(&base_sem, Some(SemType::Struct(_)));
        let mut cur_type    = base_sem;
        let mut out         = base.to_string();

        // Unwrap the base struct variable before accessing its members.
        if base_is_struct && !steps.is_empty() {
            out = format!("{}.as_ref().unwrap()", out);
        }

        for (i, step) in steps.iter().enumerate() {
            let is_last = i == steps.len() - 1;
            match step {
                AccessStep::Field(fname) => {
                    // Look up the field's type from the current struct.
                    let field_t = if let Some(SemType::Struct(sname)) = &cur_type {
                        self.struct_fields
                            .get(sname)
                            .and_then(|fs| fs.iter().find(|(n, _)| n == fname))
                            .map(|(_, t)| t.clone())
                    } else {
                        None
                    };

                    out = match (&field_t, is_last) {
                        // Struct field, last step in rvalue context:
                        // return the Option<Box<S>> as-is so it can be used as a
                        // struct variable, compared to None, passed to a function, etc.
                        (Some(SemType::Struct(_)), true) =>
                            format!("{}.{}.clone()", out, fname),
                        // Struct field, intermediate step: unwrap to get &S.
                        (Some(SemType::Struct(_)), false) =>
                            format!("{}.{}.as_ref().unwrap()", out, fname),
                        // Primitive / collection field, last step: unwrap the Option.
                        (_, true) =>
                            format!("{}.{}.unwrap()", out, fname),
                        // Primitive / collection field, intermediate: just access.
                        (_, false) =>
                            format!("{}.{}", out, fname),
                    };
                    cur_type = field_t;
                }

                AccessStep::Index(idx_expr) => {
                    let idx = self.gen_expr(idx_expr);
                    out      = format!("{}[{} as usize]", out, idx);
                    cur_type = match cur_type {
                        Some(SemType::Array { elem, .. }) => Some(*elem),
                        Some(SemType::List  { elem })     => Some(*elem),
                        _                                  => None,
                    };
                }

                AccessStep::Call(args) => {
                    let av: Vec<_> = args.iter().map(|a| self.gen_expr(a)).collect();
                    out      = format!("{}({})", out, av.join(", "));
                    cur_type = None; // return type not tracked past a call
                }
            }
        }

        out
    }

    // ── built-in function rewrites ────────────────────────────────────────────

    fn try_builtin(&mut self, name: &str, args: &[ParseNode]) -> Option<String> {
        let a: Vec<String> = args.iter().map(|x| self.gen_expr(x)).collect();
        let n = a.len();

        match (name, n) {
            // ── I/O ──────────────────────────────────────────────────────────
            ("print", 0) => Some("println!()".into()),
            ("print", 1) => Some(format!("println!(\"{{}}\", {})", a[0])),
            ("print", _) => {
                let fmt = (0..n).map(|_| "{}").collect::<Vec<_>>().join(" ");
                Some(format!("println!(\"{}\", {})", fmt, a.join(", ")))
            }
            ("input", 0) => Some(
                "{ let mut __ln = String::new(); \
                 io::stdin().lock().read_line(&mut __ln).unwrap(); \
                 __ln.trim().to_string() }"
                    .into(),
            ),

            // ── list operations ───────────────────────────────────────────────
            ("append", 2) => Some(format!("{}.push({})", a[0], a[1])),
            ("pop",    1) => Some(format!("{}.pop().unwrap()", a[0])),
            ("insert", 3) => Some(format!("{}.insert({} as usize, {})", a[0], a[1], a[2])),
            ("delete", 2) => Some(format!("{}.remove({} as usize)", a[0], a[1])),
            ("find",   2) => Some(format!(
                "{}.iter().position(|__x| *__x == {}).map(|i| i as i64).unwrap_or(-1_i64)",
                a[0], a[1]
            )),
            ("len", 1) => Some(format!("({}.len() as i64)", a[0])),

            // ── math ──────────────────────────────────────────────────────────
            ("pow",   2) => Some(format!("({} as f64).powf({} as f64)", a[0], a[1])),
            ("abs",   1) => Some(format!("{}.abs()", a[0])),
            ("sqrt",  1) => Some(format!("({} as f64).sqrt()", a[0])),
            ("floor", 1) => Some(format!("({} as f64).floor() as i64", a[0])),
            ("ceil",  1) => Some(format!("({} as f64).ceil() as i64", a[0])),
            ("min",   2) => Some(format!("{}.min({})", a[0], a[1])),
            ("max",   2) => Some(format!("{}.max({})", a[0], a[1])),

            // ── type conversions ──────────────────────────────────────────────
            ("to_int",   1) => Some(format!("({} as i64)", a[0])),
            ("to_float", 1) => Some(format!("({} as f64)", a[0])),
            // to_str: returns a Vec<char> (matches :list<:char>)
            ("to_str",   1) => Some(format!(
                "{}.to_string().chars().collect::<Vec<char>>()",
                a[0]
            )),

            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Character / string escaping
// ─────────────────────────────────────────────────────────────────────────────

fn escape_char(c: char) -> String {
    match c {
        '\n' => "\\n".into(),
        '\t' => "\\t".into(),
        '\r' => "\\r".into(),
        '\\' => "\\\\".into(),
        '\'' => "\\'".into(),
        '\0' => "\\0".into(),
        _    => c.to_string(),
    }
}