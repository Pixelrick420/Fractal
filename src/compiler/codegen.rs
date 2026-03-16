use crate::compiler::parser::{
    AccessStep, AddOp, AssignOp, CmpOp, MulOp, ParseNode, ShiftOp, UnOp,
};
use crate::compiler::semanter::{SemType, SemanticResult, SymbolKind};
use std::collections::HashMap;

pub fn generate(root: &ParseNode, sem: &SemanticResult) -> String {
    let mut cg = CodeGen::new(sem);
    cg.gen_root(root);
    cg.buf
}

struct CodeGen {
    buf: String,
    indent: usize,

    var_types: HashMap<String, SemType>,

    struct_fields: HashMap<String, Vec<(String, SemType)>>,

    module_names: std::collections::HashSet<String>,

    current_return_struct: Option<String>,

    current_return_void: bool,

    struct_params: std::collections::HashSet<String>,
}

impl CodeGen {
    fn new(sem: &SemanticResult) -> Self {
        let mut var_types = HashMap::new();
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
            current_return_void: false,
            struct_params: std::collections::HashSet::new(),
        }
    }

    fn line(&mut self, s: &str) {
        let pad = "    ".repeat(self.indent);
        self.buf.push_str(&pad);
        self.buf.push_str(s);
        self.buf.push('\n');
    }

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

    fn type_str(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeInt => "i64".into(),
            ParseNode::TypeFloat => "f64".into(),
            ParseNode::TypeChar => "char".into(),
            ParseNode::TypeBoolean => "bool".into(),
            ParseNode::TypeVoid => "()".into(),
            ParseNode::TypeArray { elem, size } => format!("[{}; {}]", self.type_str(elem), size),
            ParseNode::TypeList { elem } => match elem.as_ref() {
                ParseNode::TypeStruct { name } => format!("Vec<Option<Box<{}>>>", name),
                _ => format!("Vec<{}>", self.type_str(elem)),
            },
            ParseNode::TypeStruct { name } => name.clone(),
            _ => "/* ? */".into(),
        }
    }

    fn field_type_str(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeStruct { name } => format!("Option<Box<{}>>", name),
            _ => format!("Option<{}>", self.type_str(node)),
        }
    }

    fn ret_type_str(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeVoid => String::new(),
            ParseNode::TypeStruct { name } => format!(" -> Option<Box<{}>>", name),
            _ => format!(" -> {}", self.type_str(node)),
        }
    }

    fn zero_val(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeInt => "0_i64".into(),
            ParseNode::TypeFloat => "0.0_f64".into(),
            ParseNode::TypeChar => "'\\0'".into(),
            ParseNode::TypeBoolean => "false".into(),
            ParseNode::TypeArray { elem, size } => format!("[{}; {}]", self.zero_val(elem), size),
            ParseNode::TypeList { .. } => "Vec::new()".into(),
            ParseNode::TypeStruct { name } => format!("Some(Box::new({}::default()))", name),
            _ => "Default::default()".into(),
        }
    }

    fn gen_root(&mut self, root: &ParseNode) {
        let items = match root {
            ParseNode::Program(v) => v,
            _ => return,
        };

        for item in items {
            if let ParseNode::Module { name, .. } = item {
                self.module_names.insert(name.clone());
            }
        }

        self.line("#![allow(unused_variables, unused_mut, dead_code, non_snake_case, unused_imports, unreachable_patterns)]");
        self.line("use std::io::{self, BufRead, Write};");
        self.blank();

        let (defs, stmts): (Vec<_>, Vec<_>) = items.iter().partition(|n| {
            matches!(
                n,
                ParseNode::FuncDef { .. } | ParseNode::StructDef { .. } | ParseNode::Module { .. }
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
        } else {
            self.line("fn main() {}");
        }
    }

    fn gen_item(&mut self, node: &ParseNode) {
        match node {
            ParseNode::FuncDef {
                name,
                params,
                return_type,
                body,
            } => self.gen_funcdef(name, params, return_type, body),
            ParseNode::StructDef { name, fields } => self.gen_structdef(name, fields),
            ParseNode::Module { name, items } => self.gen_module(name, items),
            _ => self.gen_stmt(node),
        }
    }

    fn gen_structdef(&mut self, name: &str, fields: &[ParseNode]) {
        self.line("#[derive(Debug, Clone, Default)]");
        self.line(&format!("pub struct {} {{", name));
        self.indent();
        for f in fields {
            if let ParseNode::Field {
                data_type,
                name: fname,
            } = f
            {
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

    fn gen_funcdef(
        &mut self,
        name: &str,
        params: &[ParseNode],
        return_type: &ParseNode,
        body: &[ParseNode],
    ) {
        let prev_ret = self.current_return_struct.take();
        let prev_void = self.current_return_void;
        if let ParseNode::TypeStruct { name: sname } = return_type {
            self.current_return_struct = Some(sname.clone());
        }
        self.current_return_void = matches!(return_type, ParseNode::TypeVoid);

        let prev_params = std::mem::take(&mut self.struct_params);
        for p in params {
            if let ParseNode::Param {
                data_type,
                name: pname,
            } = p
            {
                if matches!(data_type.as_ref(), ParseNode::TypeStruct { .. }) {
                    self.struct_params.insert(escape_ident(pname));
                }
            }
        }

        let params_str: Vec<String> = params
            .iter()
            .filter_map(|p| match p {
                ParseNode::Param {
                    data_type,
                    name: pname,
                } => {
                    let ty = match data_type.as_ref() {
                        ParseNode::TypeStruct { name: sname } => {
                            format!("&mut Option<Box<{}>>", sname)
                        }
                        _ => self.type_str(data_type),
                    };
                    Some(format!("mut {}: {}", escape_ident(pname), ty))
                }
                _ => None,
            })
            .collect();

        let ret = self.ret_type_str(return_type);
        let visibility = if name == "main" { "" } else { "pub " };
        self.line(&format!(
            "{}fn {}({}){} {{",
            visibility,
            escape_ident(name),
            params_str.join(", "),
            ret
        ));
        self.indent();
        for s in body {
            self.gen_stmt(s);
        }
        self.dedent();
        self.line("}");

        self.current_return_struct = prev_ret;
        self.current_return_void = prev_void;
        self.struct_params = prev_params;
    }

    fn gen_module(&mut self, name: &str, items: &[ParseNode]) {
        self.module_names.insert(name.to_string());

        self.line(&format!("pub mod {} {{", name));
        self.indent();
        self.line("use super::*;");
        self.blank();

        let (defs, stmts): (Vec<_>, Vec<_>) = items.iter().partition(|n| {
            matches!(
                n,
                ParseNode::FuncDef { .. } | ParseNode::StructDef { .. } | ParseNode::Module { .. }
            )
        });

        for d in &defs {
            self.gen_item(d);
            self.blank();
        }

        let (static_decls, runtime_stmts): (Vec<_>, Vec<_>) = stmts.into_iter().partition(|n| {
            matches!(
                n,
                ParseNode::Decl {
                    data_type,
                    ..
                } if matches!(
                    data_type.as_ref(),
                    ParseNode::TypeInt
                        | ParseNode::TypeFloat
                        | ParseNode::TypeChar
                        | ParseNode::TypeBoolean
                )
            )
        });

        for s in &static_decls {
            if let ParseNode::Decl {
                data_type,
                name: var_name,
                init,
            } = s
            {
                let ty = self.type_str(data_type);
                let rust_ty = match data_type.as_ref() {
                    ParseNode::TypeInt => "i64",
                    ParseNode::TypeFloat => "f64",
                    ParseNode::TypeChar => "char",
                    ParseNode::TypeBoolean => "bool",
                    _ => &ty,
                };
                let rhs = match init {
                    Some(e) => self.gen_expr(e),
                    None => self.zero_val(data_type),
                };
                self.line(&format!(
                    "pub static {}: {} = {};",
                    escape_ident(var_name),
                    rust_ty,
                    rhs
                ));
            }
        }

        if !static_decls.is_empty() && !runtime_stmts.is_empty() {
            self.blank();
        }

        if !runtime_stmts.is_empty() {
            self.line("pub fn init() {");
            self.indent();
            for s in &runtime_stmts {
                self.gen_stmt(s);
            }
            self.dedent();
            self.line("}");
        }

        self.dedent();
        self.line("}");
    }

    fn gen_stmt(&mut self, node: &ParseNode) {
        match node {
            ParseNode::Decl {
                data_type,
                name,
                init,
            } => self.gen_decl(data_type, name, init.as_deref()),

            ParseNode::StructDecl {
                struct_name,
                var_name,
                init,
            } => self.gen_struct_decl(struct_name, var_name, init.as_deref()),

            ParseNode::Assign { lvalue, op, expr } => self.gen_assign(lvalue, op, expr),

            ParseNode::If {
                condition,
                then_block,
                else_block,
            } => self.gen_if(condition, then_block, else_block.as_deref()),

            ParseNode::For {
                var_type,
                var_name,
                start,
                stop,
                step,
                body,
            } => self.gen_for(var_type, var_name, start, stop, step, body),

            ParseNode::While { condition, body } => {
                let c = self.gen_expr(condition);
                self.line(&format!("while {} {{", c));
                self.indent();
                for s in body {
                    self.gen_stmt(s);
                }
                self.dedent();
                self.line("}");
            }

            ParseNode::Return(expr) => match expr.as_ref() {
                ParseNode::Null => {
                    if self.current_return_void {
                        self.line("return;");
                    } else {
                        self.line("return None;");
                    }
                }
                ParseNode::StructLit(fields) => {
                    if let Some(sname) = self.current_return_struct.clone() {
                        let body = self.emit_struct_lit_body(&sname.clone(), fields);
                        self.line(&format!("return Some(Box::new({} {{ {} }}));", sname, body));
                    } else {
                        let parts: Vec<_> = fields
                            .iter()
                            .map(|(f, e)| {
                                let v = self.gen_expr(e);
                                format!("{}: Some({})", f, v)
                            })
                            .collect();
                        self.line(&format!(
                            "return Some(Box::new(/* StructName */ {{ {} }}));",
                            parts.join(", ")
                        ));
                    }
                }

                ParseNode::Identifier(n)
                    if matches!(self.var_types.get(n.as_str()), Some(SemType::Struct(_))) =>
                {
                    self.line(&format!("return {}.clone();", n));
                }
                _ => {
                    let e = self.gen_expr(expr);
                    self.line(&format!("return {};", e));
                }
            },

            ParseNode::Exit(expr) => {
                let e = self.gen_expr(expr);
                self.line(&format!("std::process::exit({} as i32);", e));
            }

            ParseNode::Break => self.line("break;"),
            ParseNode::Continue => self.line("continue;"),

            ParseNode::ExprStmt(e) => {
                self.gen_call_stmt(e);
            }

            ParseNode::FuncDef {
                name,
                params,
                return_type,
                body,
            } => self.gen_funcdef(name, params, return_type, body),
            ParseNode::StructDef { name, fields } => self.gen_structdef(name, fields),

            _ => {}
        }
    }

    fn gen_decl(&mut self, data_type: &ParseNode, name: &str, init: Option<&ParseNode>) {
        if let ParseNode::TypeStruct { name: sname } = data_type {
            let sname = sname.clone();
            self.gen_struct_decl(&sname, name, init);
            return;
        }

        let ty = self.type_str(data_type);
        let rhs = match (data_type, init) {
            (ParseNode::TypeArray { elem, .. }, Some(ParseNode::StringLit(s)))
                if matches!(elem.as_ref(), ParseNode::TypeChar) =>
            {
                let chars: Vec<String> =
                    s.chars().map(|c| format!("'{}'", escape_char(c))).collect();
                format!("[{}]", chars.join(", "))
            }

            (ParseNode::TypeList { .. }, Some(ParseNode::ArrayLit(elems))) => {
                if elems.is_empty() {
                    "Vec::new()".to_string()
                } else {
                    let parts: Vec<_> = elems.iter().map(|e| self.gen_expr(e)).collect();
                    format!("vec![{}]", parts.join(", "))
                }
            }
            (_, Some(e)) => self.gen_expr(e),
            (_, None) => self.zero_val(data_type),
        };
        self.line(&format!(
            "let mut {}: {} = {};",
            escape_ident(name),
            ty,
            rhs
        ));
    }

    fn gen_struct_decl(&mut self, struct_name: &str, var_name: &str, init: Option<&ParseNode>) {
        let rhs = match init {
            None => format!("Some(Box::new({}::default()))", struct_name),
            Some(ParseNode::Null) => "None".to_string(),
            Some(ParseNode::StructLit(fields)) => {
                let sn = struct_name.to_string();
                let body = self.emit_struct_lit_body(&sn, fields);
                format!("Some(Box::new({} {{ {} }}))", struct_name, body)
            }
            Some(other) => self.gen_expr(other),
        };
        self.line(&format!(
            "let mut {}: Option<Box<{}>> = {};",
            escape_ident(var_name),
            struct_name,
            rhs
        ));
    }

    fn emit_struct_lit_body(
        &mut self,
        struct_name: &str,
        fields: &[(String, ParseNode)],
    ) -> String {
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
                let wrapped = match ftypes.get(fname) {
                    Some(SemType::Struct(sname)) => {
                        let sname = sname.clone();
                        if let ParseNode::StructLit(nested_fields) = fexpr {
                            let body = self.emit_struct_lit_body(&sname, nested_fields);
                            format!("{}: Some(Box::new({} {{ {} }}))", fname, sname, body)
                        } else {
                            let val = self.gen_expr(fexpr);
                            format!("{}: Some(Box::new({}))", fname, val)
                        }
                    }
                    _ => {
                        let val = self.gen_expr(fexpr);
                        format!("{}: Some({})", fname, val)
                    }
                };
                wrapped
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn gen_assign(&mut self, lvalue: &ParseNode, op: &AssignOp, expr: &ParseNode) {
        if let ParseNode::AccessChain { base, steps } = lvalue {
            if let Some(AccessStep::Field(fname)) = steps.last() {
                let fname = fname.clone();
                let is_null = matches!(expr, ParseNode::Null);
                let prefix = self.emit_struct_field_prefix(base, steps);

                if matches!(op, AssignOp::Eq) {
                    let rhs = if is_null {
                        "None".to_string()
                    } else if let ParseNode::StructLit(fields) = expr {
                        if let Some(SemType::Struct(sname)) =
                            self.resolve_field_type(base, steps, &fname)
                        {
                            let sname = sname.clone();
                            let body = self.emit_struct_lit_body(&sname, fields);
                            format!("Some(Box::new({} {{ {} }}))", sname, body)
                        } else {
                            let parts: Vec<_> = fields
                                .iter()
                                .map(|(f, e)| {
                                    let v = self.gen_expr(e);
                                    format!("{}: Some({})", f, v)
                                })
                                .collect();
                            format!("Some(/* StructName */ {{ {} }})", parts.join(", "))
                        }
                    } else {
                        let val = self.gen_expr(expr);
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
                    let op_str = Self::assign_op_str(op);
                    let rhs = self.gen_expr(expr);
                    self.line(&format!(
                        "*{}.{}.as_mut().unwrap() {} {};",
                        prefix, fname, op_str, rhs
                    ));
                }
                return;
            }
        }

        let lv = self.gen_lvalue(lvalue);
        let op_str = Self::assign_op_str(op);
        let rv = self.gen_expr(expr);
        self.line(&format!("{} {} {};", lv, op_str, rv));
    }

    fn emit_struct_field_prefix(&mut self, base: &str, steps: &[AccessStep]) -> String {
        let prefix_steps = &steps[..steps.len().saturating_sub(1)];
        let base_is_struct = matches!(self.var_types.get(base), Some(SemType::Struct(_)));

        let mut out = escape_ident(base);
        if base_is_struct {
            out = format!("{}.as_mut().unwrap()", out);
        }

        for (i, step) in prefix_steps.iter().enumerate() {
            match step {
                AccessStep::Field(f) => {
                    out = format!("{}.{}.as_mut().unwrap()", out, f);
                }
                AccessStep::Index(e) => {
                    let idx = self.gen_expr(e);
                    out = format!("{}[{} as usize]", out, idx);

                    let next_is_field = prefix_steps
                        .get(i + 1)
                        .map_or(true, |s| matches!(s, AccessStep::Field(_)));
                    if next_is_field {
                        out = format!("{}.as_mut().unwrap()", out);
                    }
                }
                AccessStep::Call(args) => {
                    let av: Vec<_> = args.iter().map(|a| self.gen_expr(a)).collect();
                    out = format!("{}({})", out, av.join(", "));
                }
            }
        }
        out
    }

    fn resolve_field_type(&self, base: &str, steps: &[AccessStep], fname: &str) -> Option<SemType> {
        let mut cur = self.var_types.get(base).cloned();
        for step in &steps[..steps.len().saturating_sub(1)] {
            cur = match (cur, step) {
                (Some(SemType::Struct(sname)), AccessStep::Field(f)) => self
                    .struct_fields
                    .get(&sname)
                    .and_then(|fs| fs.iter().find(|(n, _)| n == f))
                    .map(|(_, t)| t.clone()),
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
            ParseNode::AccessChain { base, steps } => self.emit_access_chain_mut(base, steps),
            _ => self.gen_expr(node),
        }
    }

    fn gen_list_container(&mut self, node: &ParseNode) -> String {
        match node {
            ParseNode::AccessChain { base, steps } => {
                let lv = self.emit_access_chain_mut(base, steps);

                let is_field_end = steps
                    .last()
                    .map_or(false, |s| matches!(s, AccessStep::Field(_)));
                if is_field_end {
                    format!("{}.as_mut().unwrap()", lv)
                } else {
                    lv
                }
            }
            _ => self.gen_expr(node),
        }
    }

    fn emit_access_chain_mut(&mut self, base: &str, steps: &[AccessStep]) -> String {
        let base_sem = self.var_types.get(base).cloned();
        let base_is_struct = matches!(&base_sem, Some(SemType::Struct(_)));
        let mut cur_type = base_sem;
        let mut out = escape_ident(base);

        if base_is_struct && !steps.is_empty() {
            out = format!("{}.as_mut().unwrap()", out);
        }

        for (i, step) in steps.iter().enumerate() {
            let is_last = i == steps.len() - 1;
            match step {
                AccessStep::Field(fname) => {
                    let field_t = if let Some(SemType::Struct(sname)) = &cur_type {
                        self.struct_fields
                            .get(sname)
                            .and_then(|fs| fs.iter().find(|(n, _)| n == fname))
                            .map(|(_, t)| t.clone())
                    } else {
                        None
                    };

                    out = if is_last {
                        format!("{}.{}", out, fname)
                    } else {
                        match &field_t {
                            Some(SemType::Struct(_)) => {
                                format!("{}.{}.as_mut().unwrap()", out, fname)
                            }
                            _ => {
                                format!("{}.{}.as_mut().unwrap()", out, fname)
                            }
                        }
                    };
                    cur_type = field_t;
                }

                AccessStep::Index(idx_expr) => {
                    let idx = self.gen_expr(idx_expr);
                    out = format!("{}[{} as usize]", out, idx);
                    cur_type = match cur_type {
                        Some(SemType::Array { elem, .. }) => Some(*elem),
                        Some(SemType::List { elem }) => Some(*elem),
                        _ => None,
                    };
                    if matches!(&cur_type, Some(SemType::Struct(_))) && !is_last {
                        out = format!("{}.as_mut().unwrap()", out);
                    }
                }

                AccessStep::Call(args) => {
                    let av: Vec<_> = args.iter().map(|a| self.gen_call_arg(a)).collect();
                    out = format!("{}({})", out, av.join(", "));
                    cur_type = None;
                }
            }
        }
        out
    }

    fn assign_op_str(op: &AssignOp) -> &'static str {
        match op {
            AssignOp::Eq => "=",
            AssignOp::PlusEq => "+=",
            AssignOp::MinusEq => "-=",
            AssignOp::StarEq => "*=",
            AssignOp::SlashEq => "/=",
            AssignOp::PercentEq => "%=",
            AssignOp::AmpEq => "&=",
            AssignOp::PipeEq => "|=",
            AssignOp::CaretEq => "^=",
        }
    }

    fn gen_if(&mut self, cond: &ParseNode, then_blk: &[ParseNode], else_blk: Option<&[ParseNode]>) {
        let c = self.gen_expr(cond);
        self.line(&format!("if {} {{", c));
        self.indent();
        for s in then_blk {
            self.gen_stmt(s);
        }
        self.dedent();
        self.close_if_chain(else_blk);
    }

    fn close_if_chain(&mut self, else_blk: Option<&[ParseNode]>) {
        match else_blk {
            None => self.line("}"),
            Some(eb) => {
                if let [ParseNode::If {
                    condition: ec,
                    then_block: et,
                    else_block: ee,
                }] = eb
                {
                    let ec_s = self.gen_expr(ec);
                    let pad = "    ".repeat(self.indent);
                    self.raw(&format!("{}}} else if {} {{\n", pad, ec_s));
                    self.indent();
                    for s in et {
                        self.gen_stmt(s);
                    }
                    self.dedent();
                    self.close_if_chain(ee.as_deref());
                } else {
                    let pad = "    ".repeat(self.indent);
                    self.raw(&format!("{}}} else {{\n", pad));
                    self.indent();
                    for s in eb {
                        self.gen_stmt(s);
                    }
                    self.dedent();
                    self.line("}");
                }
            }
        }
    }

    fn gen_for(
        &mut self,
        var_type: &ParseNode,
        var_name: &str,
        start: &ParseNode,
        stop: &ParseNode,
        step: &ParseNode,
        body: &[ParseNode],
    ) {
        let ty = self.type_str(var_type);
        let s_s = self.gen_expr(start);
        let st_s = self.gen_expr(stop);
        let sp_s = self.gen_expr(step);

        let vn = escape_ident(var_name);
        self.line("{");
        self.indent();
        self.line(&format!("let mut {}: {} = {};", vn, ty, s_s));
        self.line(&format!("while {} < {} {{", vn, st_s));
        self.indent();
        for stmt in body {
            self.gen_stmt(stmt);
        }
        self.line(&format!("{} += {};", vn, sp_s));
        self.dedent();
        self.line("}");
        self.dedent();
        self.line("}");
    }

    fn gen_call_stmt(&mut self, node: &ParseNode) {
        let (func_base, call_args) = match node {
            ParseNode::AccessChain { base, steps } => {
                if steps.len() == 1 {
                    if let AccessStep::Call(args) = &steps[0] {
                        (base.as_str(), args.as_slice())
                    } else {
                        let s = self.gen_expr(node);
                        self.line(&format!("{};", s));
                        return;
                    }
                } else {
                    let s = self.gen_expr(node);
                    self.line(&format!("{};", s));
                    return;
                }
            }
            _ => {
                let s = self.gen_expr(node);
                self.line(&format!("{};", s));
                return;
            }
        };

        if self.try_builtin(func_base, call_args).is_some() {
            let s = self.gen_expr(node);
            self.line(&format!("{};", s));
            return;
        }

        let mut mut_bases: std::collections::HashSet<String> = std::collections::HashSet::new();
        for arg in call_args {
            match arg {
                ParseNode::Identifier(n)
                    if matches!(self.var_types.get(n.as_str()), Some(SemType::Struct(_)))
                        && !self.struct_params.contains(n.as_str()) =>
                {
                    mut_bases.insert(n.clone());
                }
                ParseNode::AccessChain { base, steps }
                    if self.access_chain_is_struct(base, steps) =>
                {
                    mut_bases.insert(base.clone());
                }
                _ => {}
            }
        }

        let mut hoisted: Vec<(usize, String)> = Vec::new();
        for (i, arg) in call_args.iter().enumerate() {
            if let ParseNode::AccessChain { base, steps } = arg {
                if mut_bases.contains(base.as_str()) && !self.access_chain_is_struct(base, steps) {
                    let tmp = format!("__arg_{}_{}", func_base, i);
                    let val = self.emit_access_chain(base, steps);
                    self.line(&format!("let {} = {};", tmp, val));
                    hoisted.push((i, tmp));
                }
            }
        }

        let hoisted_map: HashMap<usize, String> = hoisted.into_iter().collect();
        let args_str: Vec<String> = call_args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                if let Some(tmp) = hoisted_map.get(&i) {
                    tmp.clone()
                } else {
                    self.gen_call_arg(arg)
                }
            })
            .collect();

        self.line(&format!(
            "{}({});",
            escape_ident(func_base),
            args_str.join(", ")
        ));
    }

    fn gen_call_arg(&mut self, node: &ParseNode) -> String {
        match node {
            ParseNode::Identifier(n) => {
                if matches!(self.var_types.get(n.as_str()), Some(SemType::Struct(_))) {
                    if self.struct_params.contains(n.as_str()) {
                        return n.clone();
                    }
                    return format!("&mut {}", n);
                }
                n.clone()
            }

            ParseNode::AccessChain { base, steps } => {
                let is_struct_result = self.access_chain_is_struct(base, steps);
                let inner = self.emit_access_chain(base, steps);
                if is_struct_result {
                    if let Some(stripped) = inner.strip_suffix(".clone()") {
                        return format!("&mut {}", stripped);
                    }

                    if self.struct_params.contains(base.as_str()) && steps.is_empty() {
                        return inner;
                    }
                    format!("&mut {}", inner)
                } else {
                    inner
                }
            }
            _ => self.gen_expr(node),
        }
    }

    fn access_chain_is_struct(&self, base: &str, steps: &[AccessStep]) -> bool {
        let mut cur = self.var_types.get(base).cloned();

        if steps.is_empty() {
            return matches!(cur, Some(SemType::Struct(_)));
        }
        for step in steps {
            cur = match (cur, step) {
                (Some(SemType::Struct(sname)), AccessStep::Field(f)) => self
                    .struct_fields
                    .get(&sname)
                    .and_then(|fs| fs.iter().find(|(n, _)| n == f))
                    .map(|(_, t)| t.clone()),
                (Some(SemType::Array { elem, .. }), AccessStep::Index(_)) => Some(*elem),
                (Some(SemType::List { elem }), AccessStep::Index(_)) => Some(*elem),
                _ => None,
            };
        }
        matches!(cur, Some(SemType::Struct(_)))
    }

    fn gen_expr(&mut self, node: &ParseNode) -> String {
        match node {
            ParseNode::IntLit(v) => format!("{}_i64", v),
            ParseNode::FloatLit(v) => {
                format!("{:?}_f64", v)
            }
            ParseNode::CharLit(c) => format!("'{}'", escape_char(*c)),
            ParseNode::StringLit(s) => {
                let escaped: String = s.chars().map(|c| escape_char(c)).collect();
                format!("\"{}\".to_string()", escaped)
            }
            ParseNode::BoolLit(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            ParseNode::Null => "None".into(),

            ParseNode::Identifier(n) => {
                let escaped = escape_ident(n);
                if matches!(self.var_types.get(n.as_str()), Some(SemType::Struct(_))) {
                    format!("{}.clone()", escaped)
                } else {
                    escaped
                }
            }

            ParseNode::AccessChain { base, steps } => self.emit_access_chain(base, steps),

            ParseNode::ArrayLit(elems) => {
                if elems.is_empty() {
                    "Vec::new()".into()
                } else {
                    let parts: Vec<_> = elems.iter().map(|e| self.gen_expr(e)).collect();
                    format!("[{}]", parts.join(", "))
                }
            }

            ParseNode::StructLit(fields) => {
                let parts: Vec<_> = fields
                    .iter()
                    .map(|(fname, fexpr)| {
                        let val = if matches!(fexpr, ParseNode::Null) {
                            "None".to_string()
                        } else {
                            format!("Some({})", self.gen_expr(fexpr))
                        };
                        format!("{}: {}", fname, val)
                    })
                    .collect();
                format!("/* StructName */ {{ {} }}", parts.join(", "))
            }

            ParseNode::LogOr { left, right } => {
                format!("({} || {})", self.gen_expr(left), self.gen_expr(right))
            }
            ParseNode::LogAnd { left, right } => {
                format!("({} && {})", self.gen_expr(left), self.gen_expr(right))
            }
            ParseNode::LogNot { operand } => format!("(!{})", self.gen_expr(operand)),

            ParseNode::Cmp { left, op, right } => {
                let op_s = match op {
                    CmpOp::Gt => ">",
                    CmpOp::Lt => "<",
                    CmpOp::Ge => ">=",
                    CmpOp::Le => "<=",
                    CmpOp::EqEq => "==",
                    CmpOp::Ne => "!=",
                };
                format!(
                    "({} {} {})",
                    self.gen_expr(left),
                    op_s,
                    self.gen_expr(right)
                )
            }

            ParseNode::BitOr { left, right } => {
                format!("({} | {})", self.gen_expr(left), self.gen_expr(right))
            }
            ParseNode::BitXor { left, right } => {
                format!("({} ^ {})", self.gen_expr(left), self.gen_expr(right))
            }
            ParseNode::BitAnd { left, right } => {
                format!("({} & {})", self.gen_expr(left), self.gen_expr(right))
            }
            ParseNode::BitShift { left, op, right } => {
                let op_s = match op {
                    ShiftOp::Left => "<<",
                    ShiftOp::Right => ">>",
                };
                format!(
                    "({} {} {})",
                    self.gen_expr(left),
                    op_s,
                    self.gen_expr(right)
                )
            }

            ParseNode::Add { left, op, right } => {
                let op_s = match op {
                    AddOp::Add => "+",
                    AddOp::Sub => "-",
                };
                format!(
                    "({} {} {})",
                    self.gen_expr(left),
                    op_s,
                    self.gen_expr(right)
                )
            }
            ParseNode::Mul { left, op, right } => {
                let op_s = match op {
                    MulOp::Mul => "*",
                    MulOp::Div => "/",
                    MulOp::Mod => "%",
                };
                format!(
                    "({} {} {})",
                    self.gen_expr(left),
                    op_s,
                    self.gen_expr(right)
                )
            }

            ParseNode::Unary { op, operand } => {
                let op_s = match op {
                    UnOp::Neg => "-",
                    UnOp::BitNot => "!",
                };
                format!("({}{})", op_s, self.gen_expr(operand))
            }

            ParseNode::Cast { target_type, expr } => {
                let e = self.gen_expr(expr);
                match target_type.as_ref() {
                    ParseNode::TypeChar => {
                        format!("(char::from_u32({} as u32).unwrap())", e)
                    }

                    ParseNode::TypeBoolean => match expr.as_ref() {
                        ParseNode::FloatLit(_) => format!("({} != 0.0_f64)", e),
                        ParseNode::Cast {
                            target_type: inner_ty,
                            ..
                        } if matches!(inner_ty.as_ref(), ParseNode::TypeFloat) => {
                            format!("({} != 0.0_f64)", e)
                        }
                        _ => format!("({} != 0_i64)", e),
                    },

                    ParseNode::TypeFloat => match expr.as_ref() {
                        ParseNode::BoolLit(_) => format!("(({} as i64) as f64)", e),
                        ParseNode::Identifier(n)
                            if matches!(self.var_types.get(n.as_str()), Some(SemType::Boolean)) =>
                        {
                            format!("(({} as i64) as f64)", e)
                        }
                        ParseNode::Cast {
                            target_type: inner_ty,
                            ..
                        } if matches!(inner_ty.as_ref(), ParseNode::TypeBoolean) => {
                            format!("(({} as i64) as f64)", e)
                        }
                        _ => format!("({} as f64)", e),
                    },

                    ParseNode::TypeInt => format!("({} as i64)", e),
                    _ => {
                        let ty = self.type_str(target_type);
                        format!("({} as {})", e, ty)
                    }
                }
            }

            _ => "/* unsupported expr */".into(),
        }
    }

    fn emit_access_chain(&mut self, base: &str, steps: &[AccessStep]) -> String {
        if steps.len() == 1 {
            if let AccessStep::Call(args) = &steps[0] {
                if let Some(s) = self.try_builtin(base, args) {
                    return s;
                }
            }
        }

        let base_escaped = escape_ident(base);
        let base_is_param_struct = self.struct_params.contains(base);

        if steps.len() == 1 {
            if let AccessStep::Field(fname) = &steps[0] {
                if !self.var_types.contains_key(base) {
                    if base_is_param_struct {
                        return format!("{}.as_ref().unwrap().{}.unwrap()", base_escaped, fname);
                    }
                    return format!("{}::{}", base_escaped, fname);
                }
            }
        }

        if steps.len() >= 2 {
            if let AccessStep::Field(qualified_name) = &steps[0] {
                let full = format!("{}::{}", base_escaped, qualified_name);
                if !self.var_types.contains_key(base) {
                    if base_is_param_struct {
                        let mut out = format!("{}.as_ref().unwrap()", base_escaped);
                        for (i, step) in steps.iter().enumerate() {
                            let is_last = i == steps.len() - 1;
                            match step {
                                AccessStep::Field(f) => {
                                    out = if is_last {
                                        format!("{}.{}.unwrap()", out, f)
                                    } else {
                                        format!("{}.{}.as_ref().unwrap()", out, f)
                                    };
                                }
                                AccessStep::Index(e) => {
                                    let idx = self.gen_expr(e);
                                    out = format!("{}[{} as usize]", out, idx);
                                }
                                AccessStep::Call(args) => {
                                    let av: Vec<_> =
                                        args.iter().map(|a| self.gen_call_arg(a)).collect();
                                    out = format!("{}({})", out, av.join(", "));
                                }
                            }
                        }
                        return out;
                    }
                    let mut out = full;
                    for step in &steps[1..] {
                        match step {
                            AccessStep::Call(args) => {
                                let av: Vec<_> =
                                    args.iter().map(|a| self.gen_call_arg(a)).collect();

                                if let Some(s) = self.try_builtin(qualified_name, args) {
                                    return format!("{}::{}", base, s);
                                }
                                out = format!("{}({})", out, av.join(", "));
                            }
                            AccessStep::Field(f) => out = format!("{}::{}", out, f),
                            AccessStep::Index(e) => {
                                let idx = self.gen_expr(e);
                                out = format!("{}[{} as usize]", out, idx);
                            }
                        }
                    }
                    return out;
                }
            }
        }

        let base_sem = self.var_types.get(base).cloned();

        let base_is_struct = base_is_param_struct || matches!(&base_sem, Some(SemType::Struct(_)));
        let mut cur_type = if base_is_param_struct { None } else { base_sem };
        let mut out = base_escaped;

        if base_is_struct && !steps.is_empty() {
            out = format!("{}.as_ref().unwrap()", out);
        }

        for (i, step) in steps.iter().enumerate() {
            let is_last = i == steps.len() - 1;
            match step {
                AccessStep::Field(fname) => {
                    let field_t = if let Some(SemType::Struct(sname)) = &cur_type {
                        self.struct_fields
                            .get(sname)
                            .and_then(|fs| fs.iter().find(|(n, _)| n == fname))
                            .map(|(_, t)| t.clone())
                    } else {
                        None
                    };

                    out = match (&field_t, is_last) {
                        (Some(SemType::Struct(_)), true) => format!("{}.{}.clone()", out, fname),

                        (Some(SemType::Struct(_)), false) => {
                            format!("{}.{}.as_ref().unwrap()", out, fname)
                        }

                        (Some(SemType::List { .. }) | Some(SemType::Array { .. }), true) => {
                            format!("{}.{}.as_ref().unwrap()", out, fname)
                        }

                        (_, true) => format!("{}.{}.unwrap()", out, fname),

                        (_, false) => format!("{}.{}.as_ref().unwrap()", out, fname),
                    };
                    cur_type = field_t;
                }

                AccessStep::Index(idx_expr) => {
                    let idx = self.gen_expr(idx_expr);
                    out = format!("{}[{} as usize]", out, idx);
                    cur_type = match cur_type {
                        Some(SemType::Array { elem, .. }) => Some(*elem),
                        Some(SemType::List { elem }) => Some(*elem),
                        _ => None,
                    };
                    if matches!(&cur_type, Some(SemType::Struct(_))) {
                        if is_last {
                            out = format!("{}.clone()", out);
                        } else {
                            out = format!("{}.as_ref().unwrap()", out);
                        }
                    }
                }

                AccessStep::Call(args) => {
                    let av: Vec<_> = args.iter().map(|a| self.gen_call_arg(a)).collect();
                    out = format!("{}({})", out, av.join(", "));
                    cur_type = None;
                }
            }
        }

        out
    }

    fn try_builtin(&mut self, name: &str, args: &[ParseNode]) -> Option<String> {
        let n = args.len();

        let a: Vec<String> = args.iter().map(|x| self.gen_expr(x)).collect();

        match (name, n) {
            ("print", 0) => Some("println!()".into()),

            ("print", 1) => {
                if let ParseNode::StringLit(s) = &args[0] {
                    let escaped: String = s.chars().map(|c| escape_char(c)).collect();
                    Some(format!("println!(\"{}\")", escaped))
                } else {
                    Some(format!("println!(\"{{}}\", {})", a[0]))
                }
            }

            ("print", _) => {
                let fmt_lit = if let ParseNode::StringLit(s) = &args[0] {
                    let escaped: String = s.chars().map(|c| escape_char(c)).collect();
                    format!("\"{}\"", escaped)
                } else {
                    a[0].clone()
                };
                let rest = a[1..].join(", ");
                Some(format!("println!({}, {})", fmt_lit, rest))
            }

            ("input", _) => Some(
                "{ let mut __ln = String::new(); \
                 io::stdin().lock().read_line(&mut __ln).unwrap(); \
                 __ln.trim().to_string() }"
                    .into(),
            ),

            ("append", 2) => {
                let container = self.gen_list_container(&args[0]);
                let val = self.gen_expr(&args[1]);
                Some(format!("{}.push({})", container, val))
            }
            ("pop", 1) => {
                let container = self.gen_list_container(&args[0]);
                Some(format!("{}.pop().unwrap()", container))
            }
            ("insert", 2) => {
                let container = self.gen_list_container(&args[0]);
                let val = self.gen_expr(&args[1]);
                Some(format!("{}.insert(0, {})", container, val))
            }
            ("insert", 3) => {
                let container = self.gen_list_container(&args[0]);
                let idx = self.gen_expr(&args[1]);
                let val = self.gen_expr(&args[2]);
                Some(format!("{}.insert({} as usize, {})", container, idx, val))
            }
            ("delete", 2) => {
                let container = self.gen_list_container(&args[0]);
                let idx = self.gen_expr(&args[1]);
                Some(format!("{}.remove({} as usize)", container, idx))
            }
            ("find", 2) => Some(format!(
                "{}.iter().position(|__x| *__x == {}).map(|i| i as i64).unwrap_or(-1_i64)",
                a[0], a[1]
            )),
            ("len", 1) => Some(format!("({}.len() as i64)", a[0])),

            ("pow", 2) => Some(format!("({} as f64).powf({} as f64)", a[0], a[1])),
            ("abs", 1) => Some(format!("{}.abs()", a[0])),
            ("sqrt", 1) => Some(format!("({} as f64).sqrt()", a[0])),
            ("floor", 1) => Some(format!("({} as f64).floor() as i64", a[0])),
            ("ceil", 1) => Some(format!("({} as f64).ceil() as i64", a[0])),
            ("min", 2) => Some(format!("{}.min({})", a[0], a[1])),
            ("max", 2) => Some(format!("{}.max({})", a[0], a[1])),

            ("to_int", 1) => Some(format!("({} as i64)", a[0])),
            ("to_float", 1) => Some(format!("({} as f64)", a[0])),

            ("to_str", 1) => Some(format!(
                "{}.to_string().chars().collect::<Vec<char>>()",
                a[0]
            )),

            _ => None,
        }
    }
}

fn escape_ident(name: &str) -> String {
    const RUST_KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "union", "unsafe", "use", "where", "while", "abstract", "become", "box",
        "do", "final", "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
    ];
    if RUST_KEYWORDS.contains(&name) {
        format!("r#{}", name)
    } else {
        name.to_string()
    }
}

fn escape_char(c: char) -> String {
    match c {
        '\n' => "\\n".into(),
        '\t' => "\\t".into(),
        '\r' => "\\r".into(),
        '\\' => "\\\\".into(),
        '\'' => "\\'".into(),
        '\0' => "\\0".into(),
        _ => c.to_string(),
    }
}
