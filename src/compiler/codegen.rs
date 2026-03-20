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

pub fn generate_debug(root: &ParseNode, sem: &SemanticResult, debug_out_path: &str) -> String {
    let mut cg = CodeGen::new(sem);
    cg.debug_mode = true;
    cg.debug_path = debug_out_path.replace('\\', "\\\\").replace('"', "\\\"");
    cg.debug_current_func = "main".into();
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
    struct_param_types: HashMap<String, String>,
    bool_params: std::collections::HashSet<String>,
    array_params: std::collections::HashSet<String>,
    array_param_elem_types: HashMap<String, SemType>,
    list_params: std::collections::HashSet<String>,
    list_param_elem_types: HashMap<String, SemType>,
    func_return_types: HashMap<String, SemType>,
    hoist_buf: Vec<String>,
    hoist_counter: usize,
    local_var_types: HashMap<String, SemType>,

    debug_mode: bool,
    debug_path: String,
    debug_step: usize,

    debug_visible_vars: Vec<(String, String)>,
    debug_current_func: String,
}

impl CodeGen {
    fn new(sem: &SemanticResult) -> Self {
        let mut var_types = HashMap::new();
        let mut struct_fields = HashMap::new();
        let mut func_return_types = HashMap::new();

        for sym in &sem.symbol_table {
            match &sym.kind {
                SymbolKind::Variable => {
                    if sym.origin.starts_with("param:") {
                        continue;
                    }
                    if sym.scope_depth == 0 {
                        var_types.insert(sym.name.clone(), sym.sem_type.clone());
                    }
                }
                SymbolKind::Struct { fields } => {
                    struct_fields.insert(sym.name.clone(), fields.clone());
                }
                SymbolKind::Function { .. } => {
                    func_return_types.insert(sym.name.clone(), sym.sem_type.clone());
                }
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
            struct_param_types: HashMap::new(),
            bool_params: std::collections::HashSet::new(),
            array_params: std::collections::HashSet::new(),
            list_params: std::collections::HashSet::new(),
            array_param_elem_types: HashMap::new(),
            list_param_elem_types: HashMap::new(),
            func_return_types,
            hoist_buf: Vec::new(),
            hoist_counter: 0,
            local_var_types: HashMap::new(),
            debug_mode: false,
            debug_path: String::new(),
            debug_step: 0,
            debug_visible_vars: Vec::new(),
            debug_current_func: String::new(),
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
            ParseNode::TypeArray { elem, size } => match elem.as_ref() {
                ParseNode::TypeStruct { name } => {
                    format!("[Option<Box<{}>>; {}]", escape_struct_name(name), size)
                }
                _ => format!("[{}; {}]", self.type_str(elem), size),
            },
            ParseNode::TypeList { elem } => match elem.as_ref() {
                ParseNode::TypeStruct { name } => {
                    format!("Vec<Option<Box<{}>>>", escape_struct_name(name))
                }
                _ => format!("Vec<{}>", self.type_str(elem)),
            },
            ParseNode::TypeStruct { name } => escape_struct_name(name),
            _ => "/* ? */".into(),
        }
    }

    fn field_type_str(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeStruct { name } => format!("Option<Box<{}>>", escape_struct_name(name)),
            _ => format!("Option<{}>", self.type_str(node)),
        }
    }

    fn ret_type_str(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeVoid => String::new(),
            ParseNode::TypeStruct { name } => {
                format!(" -> Option<Box<{}>>", escape_struct_name(name))
            }
            _ => format!(" -> {}", self.type_str(node)),
        }
    }

    fn zero_val(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeInt => "0_i64".into(),
            ParseNode::TypeFloat => "0.0_f64".into(),
            ParseNode::TypeChar => "'\\0'".into(),
            ParseNode::TypeBoolean => "false".into(),
            ParseNode::TypeVoid => "()".into(),

            ParseNode::TypeArray { elem, size } => {
                let elem_zero = self.zero_val_array_elem(elem);
                let elems: Vec<String> = (0..*size).map(|_| elem_zero.clone()).collect();
                format!("[{}]", elems.join(", "))
            }

            ParseNode::TypeList { .. } => "Vec::new()".into(),

            ParseNode::TypeStruct { name } => self.zero_val_struct(name),

            _ => "0_i64".into(),
        }
    }

    fn zero_val_array_elem(&self, node: &ParseNode) -> String {
        match node {
            ParseNode::TypeStruct { name } => self.zero_val_struct(name),
            _ => self.zero_val(node),
        }
    }

    fn zero_val_struct(&self, name: &str) -> String {
        let sname = escape_struct_name(name);
        match self.struct_fields.get(name) {
            None => format!("Some(Box::new({}::default()))", sname),
            Some(fields) => {
                let field_inits: Vec<String> = fields
                    .iter()
                    .map(|(fname, ftype)| {
                        let val = self.zero_val_for_sem(ftype);
                        format!("{}: {}", fname, val)
                    })
                    .collect();
                format!("Some(Box::new({} {{ {} }}))", sname, field_inits.join(", "))
            }
        }
    }

    fn zero_val_for_sem(&self, sem: &SemType) -> String {
        match sem {
            SemType::Int => "Some(0_i64)".into(),
            SemType::Float => "Some(0.0_f64)".into(),
            SemType::Char => "Some('\\0')".into(),
            SemType::Boolean => "Some(false)".into(),

            SemType::Array { elem, size } => {
                let elem_zero = self.zero_val_for_sem_inner(elem);
                let elems: Vec<String> = (0..*size).map(|_| elem_zero.clone()).collect();
                format!("Some([{}])", elems.join(", "))
            }

            SemType::List { .. } => "Some(Vec::new())".into(),

            SemType::Struct(sname) => self.zero_val_struct(sname),

            _ => "Some(0_i64)".into(),
        }
    }

    fn zero_val_for_sem_inner(&self, sem: &SemType) -> String {
        match sem {
            SemType::Int => "0_i64".into(),
            SemType::Float => "0.0_f64".into(),
            SemType::Char => "'\\0'".into(),
            SemType::Boolean => "false".into(),
            SemType::Struct(sname) => self.zero_val_struct(sname),
            SemType::Array { elem, size } => {
                let inner = self.zero_val_for_sem_inner(elem);
                let elems: Vec<String> = (0..*size).map(|_| inner.clone()).collect();
                format!("[{}]", elems.join(", "))
            }
            _ => "0_i64".into(),
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
        if self.debug_mode {
            self.emit_debug_runtime();
            self.blank();
        }

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
            if self.debug_mode {
                self.line("__fractal_debug_init();");
            }
            for s in &stmts {
                self.gen_stmt(s);
                if self.debug_mode {
                    self.emit_snapshot(s);
                }
            }
            if self.debug_mode {
                let step = self.debug_step;
                self.debug_step += 1;
                let func = self.debug_current_func.clone();
                let vars_code = self.build_vars_json_code();
                let finished_line = format!(
                    "__fractal_debug_snapshot!({s}, \"Program finished\", \"{f}\", 0, [{v}], true, None::<&str>);",
                    s = step,
                    f = func,
                    v = vars_code,
                );
                self.line(&finished_line);
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
        self.line(&format!("pub struct {} {{", escape_struct_name(name)));
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
        let prev_param_types = std::mem::take(&mut self.struct_param_types);
        let prev_bool_params = std::mem::take(&mut self.bool_params);
        let prev_array_params = std::mem::take(&mut self.array_params);
        let prev_list_params = std::mem::take(&mut self.list_params);
        let prev_array_param_elem_types = std::mem::take(&mut self.array_param_elem_types);
        let prev_list_param_elem_types = std::mem::take(&mut self.list_param_elem_types);
        let prev_local_vars = std::mem::take(&mut self.local_var_types);

        for p in params {
            if let ParseNode::Param {
                data_type,
                name: pname,
            } = p
            {
                if let ParseNode::TypeStruct { name: sname } = data_type.as_ref() {
                    self.struct_params.insert(pname.clone());
                    self.struct_param_types.insert(pname.clone(), sname.clone());
                }
                if matches!(data_type.as_ref(), ParseNode::TypeBoolean) {
                    self.bool_params.insert(pname.clone());
                }
                if matches!(data_type.as_ref(), ParseNode::TypeArray { .. }) {
                    self.array_params.insert(pname.clone());
                    if let ParseNode::TypeArray { elem, .. } = data_type.as_ref() {
                        let elem_sem = self.parse_node_to_sem_type(elem);
                        self.array_param_elem_types.insert(pname.clone(), elem_sem);
                    }
                }
                if matches!(data_type.as_ref(), ParseNode::TypeList { .. }) {
                    self.list_params.insert(pname.clone());
                    if let ParseNode::TypeList { elem } = data_type.as_ref() {
                        let elem_sem = self.parse_node_to_sem_type(elem);
                        self.list_param_elem_types.insert(pname.clone(), elem_sem);
                    }
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
                            format!("&mut Option<Box<{}>>", escape_struct_name(sname))
                        }
                        ParseNode::TypeArray { elem, size } => match elem.as_ref() {
                            ParseNode::TypeStruct { name: sname } => {
                                format!(
                                    "&mut [Option<Box<{}>>; {}]",
                                    escape_struct_name(sname),
                                    size
                                )
                            }
                            _ => format!("&mut [{}; {}]", self.type_str(elem), size),
                        },
                        ParseNode::TypeList { elem } => match elem.as_ref() {
                            ParseNode::TypeStruct { name: sname } => {
                                format!("&mut Vec<Option<Box<{}>>>", escape_struct_name(sname))
                            }
                            _ => format!("&mut Vec<{}>", self.type_str(elem)),
                        },
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

        let prev_dbg_func = self.debug_current_func.clone();
        let prev_dbg_vars = self.debug_visible_vars.clone();
        self.debug_current_func = name.to_string();
        self.debug_visible_vars.clear();

        if self.debug_mode {
            self.line("__fractal_debug_init();");
        }
        for s in body {
            self.gen_stmt(s);
            if self.debug_mode {
                self.emit_snapshot(s);
            }
        }

        self.debug_current_func = prev_dbg_func;
        self.debug_visible_vars = prev_dbg_vars;
        self.dedent();
        self.line("}");

        self.current_return_struct = prev_ret;
        self.current_return_void = prev_void;
        self.struct_params = prev_params;
        self.struct_param_types = prev_param_types;
        self.bool_params = prev_bool_params;
        self.array_params = prev_array_params;
        self.list_params = prev_list_params;
        self.array_param_elem_types = prev_array_param_elem_types;
        self.list_param_elem_types = prev_list_param_elem_types;
        self.local_var_types = prev_local_vars;
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
                ParseNode::Decl { data_type, .. }
                if matches!(
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

    fn flush_hoists(&mut self) {
        let lines = std::mem::take(&mut self.hoist_buf);
        for l in lines {
            self.line(&l);
        }
    }

    fn gen_stmt(&mut self, node: &ParseNode) {
        self.flush_hoists();
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

                if matches!(condition.as_ref(), ParseNode::BoolLit(true))
                    && !block_contains_break(body)
                {
                    self.line("unreachable!();");
                }
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
                        self.flush_hoists();
                        self.line(&format!(
                            "return Some(Box::new({} {{ {} }}));",
                            escape_struct_name(&sname),
                            body
                        ));
                    } else {
                        let parts: Vec<_> = fields
                            .iter()
                            .map(|(f, e)| {
                                let v = self.gen_expr(e);
                                format!("{}: Some({})", f, v)
                            })
                            .collect();
                        self.flush_hoists();
                        self.line(&format!(
                            "return Some(Box::new(/* StructName */ {{ {} }}));",
                            parts.join(", ")
                        ));
                    }
                }

                ParseNode::AccessChain { base: n, steps }
                    if steps.is_empty()
                        && (matches!(self.var_types.get(n.as_str()), Some(SemType::Struct(_)))
                            || matches!(
                                self.local_var_types.get(n.as_str()),
                                Some(SemType::Struct(_))
                            )
                            || self.struct_params.contains(n.as_str())) =>
                {
                    self.line(&format!("return {}.clone();", escape_ident(n)));
                }

                ParseNode::AccessChain { base: n, steps }
                    if steps.is_empty() && self.array_params.contains(n.as_str()) =>
                {
                    self.line(&format!("return {}.clone();", escape_ident(n)));
                }

                ParseNode::AccessChain { base: n, steps }
                    if steps.is_empty() && self.list_params.contains(n.as_str()) =>
                {
                    self.line(&format!("return {}.clone();", escape_ident(n)));
                }
                _ => {
                    let e = self.gen_expr(expr);
                    self.flush_hoists();
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

        if matches!(data_type, ParseNode::TypeBoolean) {
            self.local_var_types
                .insert(name.to_string(), SemType::Boolean);
        }
        if let ParseNode::TypeArray { elem, size } = data_type {
            self.local_var_types.insert(
                name.to_string(),
                SemType::Array {
                    elem: Box::new(self.parse_node_to_sem_type(elem)),
                    size: *size,
                },
            );
        }
        if let ParseNode::TypeList { elem } = data_type {
            self.local_var_types.insert(
                name.to_string(),
                SemType::List {
                    elem: Box::new(self.parse_node_to_sem_type(elem)),
                },
            );
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
        self.flush_hoists();
        self.line(&format!(
            "let mut {}: {} = {};",
            escape_ident(name),
            ty,
            rhs
        ));
        if self.debug_mode {
            let tl = parse_node_type_label(data_type);
            self.debug_visible_vars.push((escape_ident(name), tl));
        }
    }

    fn gen_struct_decl(&mut self, struct_name: &str, var_name: &str, init: Option<&ParseNode>) {
        self.local_var_types.insert(
            var_name.to_string(),
            SemType::Struct(struct_name.to_string()),
        );

        let rhs = match init {
            None => self.zero_val_struct(struct_name),
            Some(ParseNode::Null) => "None".to_string(),
            Some(ParseNode::StructLit(fields)) => {
                let sn = struct_name.to_string();
                let body = self.emit_struct_lit_body(&sn, fields);
                format!(
                    "Some(Box::new({} {{ {} }}))",
                    escape_struct_name(struct_name),
                    body
                )
            }
            Some(other) => self.gen_expr(other),
        };
        self.flush_hoists();
        self.line(&format!(
            "let mut {}: Option<Box<{}>> = {};",
            escape_ident(var_name),
            escape_struct_name(struct_name),
            rhs
        ));
        if self.debug_mode {
            let sn = struct_name.to_string();
            self.debug_visible_vars
                .push((escape_ident(var_name), format!(":struct<{}>", sn)));
        }
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
                            format!(
                                "{}: Some(Box::new({} {{ {} }}))",
                                fname,
                                escape_struct_name(&sname),
                                body
                            )
                        } else {
                            let val = self.gen_expr(fexpr);
                            let rhs_is_call = matches!(fexpr,
                                ParseNode::AccessChain { steps, .. }
                                    if matches!(steps.last(), Some(AccessStep::Call(_)))
                            );
                            if rhs_is_call {
                                format!("{}: {}", fname, val)
                            } else {
                                format!("{}: Some(Box::new({}))", fname, val)
                            }
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
                            format!(
                                "Some(Box::new({} {{ {} }}))",
                                escape_struct_name(&sname),
                                body
                            )
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
                        let field_is_struct = matches!(
                            self.resolve_field_type(base, steps, &fname),
                            Some(SemType::Struct(_))
                        );
                        let rhs_is_already_option_box = field_is_struct && {
                            if let ParseNode::AccessChain {
                                base: _rhs_base,
                                steps: rhs_steps,
                            } = expr
                            {
                                matches!(rhs_steps.last(), Some(AccessStep::Call(_))) || {
                                    rhs_steps.len() == 1
                                        && matches!(&rhs_steps[0], AccessStep::Call(_))
                                }
                            } else {
                                false
                            }
                        };
                        if rhs_is_already_option_box {
                            val
                        } else if field_is_struct {
                            format!("Some(Box::new({}))", val)
                        } else {
                            format!("Some({})", val)
                        }
                    };
                    self.flush_hoists();
                    self.line(&format!("{}.{} = {};", prefix, fname, rhs));
                } else {
                    let op_str = Self::assign_op_str(op);
                    let rhs = self.gen_expr(expr);
                    self.flush_hoists();
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
        self.flush_hoists();
        self.line(&format!("{} {} {};", lv, op_str, rv));
    }

    fn emit_struct_field_prefix(&mut self, base: &str, steps: &[AccessStep]) -> String {
        let prefix_steps = &steps[..steps.len().saturating_sub(1)];
        let base_is_struct = matches!(
            self.var_types
                .get(base)
                .or_else(|| self.local_var_types.get(base)),
            Some(SemType::Struct(_))
        ) || self.struct_params.contains(base);

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
        let mut cur = self
            .var_types
            .get(base)
            .cloned()
            .or_else(|| self.local_var_types.get(base).cloned())
            .or_else(|| {
                self.struct_param_types
                    .get(base)
                    .map(|sname| SemType::Struct(sname.clone()))
            });
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
        let base_sem = self
            .var_types
            .get(base)
            .cloned()
            .or_else(|| self.local_var_types.get(base).cloned())
            .or_else(|| {
                self.struct_param_types
                    .get(base)
                    .map(|sname| SemType::Struct(sname.clone()))
            })
            .or_else(|| {
                self.array_param_elem_types
                    .get(base)
                    .map(|elem| SemType::Array {
                        elem: Box::new(elem.clone()),
                        size: 0,
                    })
            })
            .or_else(|| {
                self.list_param_elem_types
                    .get(base)
                    .map(|elem| SemType::List {
                        elem: Box::new(elem.clone()),
                    })
            });
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
                    let tmp = format!("__idx_{}", self.hoist_counter);
                    self.hoist_counter += 1;
                    self.hoist_buf
                        .push(format!("let {} = {} as usize;", tmp, idx));
                    out = format!("{}[{}]", out, tmp);
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
            self.gen_stmt_for_body(stmt, &vn, &sp_s);
        }
        self.line(&format!("{} += {};", vn, sp_s));
        self.dedent();
        self.line("}");
        self.dedent();
        self.line("}");
    }

    fn gen_stmt_for_body(&mut self, node: &ParseNode, var_name: &str, step_expr: &str) {
        match node {
            ParseNode::Continue => {
                self.line(&format!("{} += {};", var_name, step_expr));
                self.line("continue;");
            }
            ParseNode::Break => {
                self.line("break;");
            }
            ParseNode::If {
                condition,
                then_block,
                else_block,
            } => {
                self.gen_if_for_body(
                    condition,
                    then_block,
                    else_block.as_deref(),
                    var_name,
                    step_expr,
                );
            }
            ParseNode::For {
                var_type,
                var_name: inner_vn,
                start,
                stop,
                step,
                body,
            } => {
                self.gen_for(var_type, inner_vn, start, stop, step, body);
            }
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
            other => {
                self.gen_stmt(other);
            }
        }
    }

    fn gen_if_for_body(
        &mut self,
        cond: &ParseNode,
        then_blk: &[ParseNode],
        else_blk: Option<&[ParseNode]>,
        var_name: &str,
        step_expr: &str,
    ) {
        let c = self.gen_expr(cond);
        self.line(&format!("if {} {{", c));
        self.indent();
        for s in then_blk {
            self.gen_stmt_for_body(s, var_name, step_expr);
        }
        self.dedent();
        self.close_if_chain_for_body(else_blk, var_name, step_expr);
    }

    fn close_if_chain_for_body(
        &mut self,
        else_blk: Option<&[ParseNode]>,
        var_name: &str,
        step_expr: &str,
    ) {
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
                        self.gen_stmt_for_body(s, var_name, step_expr);
                    }
                    self.dedent();
                    self.close_if_chain_for_body(ee.as_deref(), var_name, step_expr);
                } else {
                    let pad = "    ".repeat(self.indent);
                    self.raw(&format!("{}}} else {{\n", pad));
                    self.indent();
                    for s in eb {
                        self.gen_stmt_for_body(s, var_name, step_expr);
                    }
                    self.dedent();
                    self.line("}");
                }
            }
        }
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
                ParseNode::AccessChain { base, steps }
                    if self.access_chain_is_struct(base, steps) =>
                {
                    mut_bases.insert(base.clone());
                }
                _ => {}
            }
        }

        let mut array_struct_args: HashMap<String, Vec<(usize, String)>> = HashMap::new();
        for (i, arg) in call_args.iter().enumerate() {
            if let ParseNode::AccessChain { base, steps } = arg {
                if steps.len() == 1 {
                    if let AccessStep::Index(idx_expr) = &steps[0] {
                        if self.access_chain_is_struct(base, steps) {
                            let idx = self.gen_expr(idx_expr);
                            array_struct_args
                                .entry(base.clone())
                                .or_default()
                                .push((i, idx));
                        }
                    }
                }
            }
        }

        let mut split_hoisted: HashMap<usize, String> = HashMap::new();
        for (arr_base, entries) in &array_struct_args {
            if entries.len() >= 2 {
                let mut sorted = entries.clone();
                sorted.sort_by_key(|(i, _)| *i);
                let arr_escaped = escape_ident(arr_base);

                if sorted.len() == 2 {
                    let (i0, idx0) = &sorted[0];
                    let (i1, idx1) = &sorted[1];
                    let tmp0 = format!("__spl_{}_{}", arr_base, i0);
                    let tmp1 = format!("__spl_{}_{}", arr_base, i1);
                    let lo_tmp = format!("__split_lo_{}", arr_base);
                    let hi_tmp = format!("__split_hi_{}", arr_base);
                    let mid_tmp = format!("__split_mid_{}", arr_base);

                    self.line(&format!(
                        "let {mid} = if ({a} as usize) < ({b} as usize) {{ {a} as usize + 1 }} else {{ {b} as usize + 1 }};",
                        mid = mid_tmp, a = idx0, b = idx1,
                    ));
                    self.line(&format!(
                        "let ({lo}, {hi}) = {arr}.split_at_mut({mid});",
                        lo = lo_tmp,
                        hi = hi_tmp,
                        arr = arr_escaped,
                        mid = mid_tmp,
                    ));
                    self.line(&format!(
                        "let {t0} = if ({a} as usize) < ({b} as usize) {{ &mut {lo}[{a} as usize] }} else {{ &mut {hi}[{a} as usize - {mid}] }};",
                        t0 = tmp0, a = idx0, b = idx1, lo = lo_tmp, hi = hi_tmp, mid = mid_tmp,
                    ));
                    self.line(&format!(
                        "let {t1} = if ({b} as usize) < ({a} as usize) {{ &mut {lo}[{b} as usize] }} else {{ &mut {hi}[{b} as usize - {mid}] }};",
                        t1 = tmp1, a = idx0, b = idx1, lo = lo_tmp, hi = hi_tmp, mid = mid_tmp,
                    ));
                    split_hoisted.insert(*i0, tmp0);
                    split_hoisted.insert(*i1, tmp1);
                } else {
                    for (i, idx) in &sorted {
                        let tmp = format!("__spl_{}_{}", arr_base, i);
                        self.line(&format!(
                            "let {tmp} = unsafe {{ &mut *({arr}.as_mut_ptr().add({idx} as usize)) }};",
                            tmp = tmp, arr = arr_escaped, idx = idx,
                        ));
                        split_hoisted.insert(*i, tmp);
                    }
                }
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
                if let Some(tmp) = split_hoisted.get(&i) {
                    tmp.clone()
                } else if let Some(tmp) = hoisted_map.get(&i) {
                    tmp.clone()
                } else {
                    self.gen_call_arg(arg)
                }
            })
            .collect();

        self.flush_hoists();
        self.line(&format!(
            "{}({});",
            escape_ident(func_base),
            args_str.join(", ")
        ));
    }

    fn gen_call_arg(&mut self, node: &ParseNode) -> String {
        match node {
            ParseNode::AccessChain { base, steps } => {
                let is_struct_result = self.access_chain_is_struct(base, steps);
                let call_returns_struct =
                    if !is_struct_result && matches!(steps.last(), Some(AccessStep::Call(_))) {
                        let func_name = steps.iter().rev().skip(1).find_map(|s| {
                            if let AccessStep::Field(f) = s {
                                Some(f.as_str())
                            } else {
                                None
                            }
                        });
                        func_name.map_or(false, |fname| {
                            let qualified = format!("{}::{}", base, fname);
                            matches!(
                                self.func_return_types
                                    .get(fname)
                                    .or_else(|| self.func_return_types.get(&qualified)),
                                Some(SemType::Struct(_))
                            )
                        })
                    } else {
                        false
                    };

                if steps.is_empty() {
                    let base_type = self
                        .var_types
                        .get(base.as_str())
                        .cloned()
                        .or_else(|| self.local_var_types.get(base.as_str()).cloned());
                    let is_array_or_list = matches!(
                        &base_type,
                        Some(SemType::Array { .. }) | Some(SemType::List { .. })
                    );
                    let is_array_param = self.array_params.contains(base.as_str());
                    let is_list_param = self.list_params.contains(base.as_str());
                    if is_array_or_list || is_array_param || is_list_param {
                        let esc = escape_ident(base);
                        if is_array_param || is_list_param {
                            return esc;
                        } else {
                            return format!("unsafe {{ &mut *(&mut {} as *mut _) }}", esc);
                        }
                    }
                }

                if is_struct_result || call_returns_struct {
                    if matches!(steps.last(), Some(AccessStep::Call(_))) {
                        let val = self.emit_access_chain(base, steps);
                        let tmp = format!("__hoist_{}", self.hoist_counter);
                        self.hoist_counter += 1;
                        self.hoist_buf.push(format!("let mut {} = {};", tmp, val));
                        return format!("&mut {}", tmp);
                    }

                    let base_escaped = escape_ident(base);
                    let base_is_param = self.struct_params.contains(base.as_str());
                    let base_is_local_struct = matches!(
                        self.var_types
                            .get(base.as_str())
                            .or_else(|| self.local_var_types.get(base.as_str())),
                        Some(SemType::Struct(_))
                    );

                    if steps.is_empty() {
                        if base_is_param {
                            return base_escaped;
                        }
                        return format!("&mut {}", base_escaped);
                    }

                    let mut out = base_escaped.clone();
                    if base_is_param || base_is_local_struct {
                        out = format!("{}.as_mut().unwrap()", out);
                    }
                    for (i, step) in steps.iter().enumerate() {
                        let is_last = i == steps.len() - 1;
                        match step {
                            AccessStep::Field(f) => {
                                if is_last {
                                    return format!("&mut {}.{}", out, f);
                                } else {
                                    out = format!("{}.{}.as_mut().unwrap()", out, f);
                                }
                            }
                            AccessStep::Index(e) => {
                                let idx = self.gen_expr(e);
                                if is_last {
                                    return format!(
                                        "unsafe {{ &mut *{}.as_mut_ptr().add({} as usize) }}",
                                        out, idx
                                    );
                                }
                                out = format!("{}[{} as usize]", out, idx);
                                out = format!("{}.as_mut().unwrap()", out);
                            }
                            AccessStep::Call(args) => {
                                let av: Vec<_> =
                                    args.iter().map(|a| self.gen_call_arg(a)).collect();
                                out = format!("{}({})", out, av.join(", "));
                            }
                        }
                    }
                    format!("&mut {}", out)
                } else {
                    self.emit_access_chain(base, steps)
                }
            }
            _ => self.gen_expr(node),
        }
    }

    fn parse_node_to_sem_type(&self, node: &ParseNode) -> SemType {
        match node {
            ParseNode::TypeInt => SemType::Int,
            ParseNode::TypeFloat => SemType::Float,
            ParseNode::TypeChar => SemType::Char,
            ParseNode::TypeBoolean => SemType::Boolean,
            ParseNode::TypeVoid => SemType::Void,
            ParseNode::TypeArray { elem, size } => SemType::Array {
                elem: Box::new(self.parse_node_to_sem_type(elem)),
                size: *size,
            },
            ParseNode::TypeList { elem } => SemType::List {
                elem: Box::new(self.parse_node_to_sem_type(elem)),
            },
            ParseNode::TypeStruct { name } => SemType::Struct(name.clone()),
            _ => SemType::Unknown,
        }
    }

    fn access_chain_is_struct(&self, base: &str, steps: &[AccessStep]) -> bool {
        let mut cur = if let Some(sname) = self.struct_param_types.get(base) {
            Some(SemType::Struct(sname.clone()))
        } else if let Some(elem) = self.array_param_elem_types.get(base) {
            Some(SemType::Array {
                elem: Box::new(elem.clone()),
                size: 0,
            })
        } else if let Some(elem) = self.list_param_elem_types.get(base) {
            Some(SemType::List {
                elem: Box::new(elem.clone()),
            })
        } else {
            self.var_types
                .get(base)
                .cloned()
                .or_else(|| self.local_var_types.get(base).cloned())
        };

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
            ParseNode::FloatLit(v) => format!("{:?}_f64", v),
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
                        ParseNode::AccessChain { base: n, steps }
                            if steps.is_empty()
                                && (matches!(
                                    self.var_types
                                        .get(n.as_str())
                                        .or_else(|| self.local_var_types.get(n.as_str())),
                                    Some(SemType::Boolean)
                                ) || self.bool_params.contains(n.as_str())) =>
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
                if !self.var_types.contains_key(base)
                    && !self.local_var_types.contains_key(base)
                    && !self.struct_params.contains(base)
                {
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
                if !self.var_types.contains_key(base)
                    && !self.local_var_types.contains_key(base)
                    && !self.struct_params.contains(base)
                {
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

        let base_sem = self
            .var_types
            .get(base)
            .cloned()
            .or_else(|| self.local_var_types.get(base).cloned())
            .or_else(|| {
                self.array_param_elem_types
                    .get(base)
                    .map(|elem| SemType::Array {
                        elem: Box::new(elem.clone()),
                        size: 0,
                    })
            })
            .or_else(|| {
                self.list_param_elem_types
                    .get(base)
                    .map(|elem| SemType::List {
                        elem: Box::new(elem.clone()),
                    })
            });

        let base_is_struct = base_is_param_struct || matches!(&base_sem, Some(SemType::Struct(_)));
        let mut cur_type = if let Some(sname) = self.struct_param_types.get(base) {
            Some(SemType::Struct(sname.clone()))
        } else {
            base_sem
        };
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

                    let call_ret: Option<SemType> = {
                        let fname = if i > 0 {
                            if let Some(AccessStep::Field(f)) = steps.get(i - 1) {
                                let q = format!("{}::{}", base, f);
                                self.func_return_types
                                    .get(f.as_str())
                                    .or_else(|| self.func_return_types.get(&q))
                                    .cloned()
                            } else {
                                None
                            }
                        } else {
                            let q = format!("{}::{}", base, base);
                            self.func_return_types
                                .get(base)
                                .or_else(|| self.func_return_types.get(&q))
                                .cloned()
                        };
                        fname
                    };

                    if matches!(&call_ret, Some(SemType::Struct(_))) && !is_last {
                        out = format!("{}.as_ref().unwrap()", out);
                    }
                    cur_type = call_ret;
                }
            }
        }

        out
    }

    fn try_builtin(&mut self, name: &str, args: &[ParseNode]) -> Option<String> {
        let n = args.len();
        let a: Vec<String> = args.iter().map(|x| self.gen_expr(x)).collect();

        match (name, n) {
            ("print", 0) => Some("{ print!(); io::stdout().flush().unwrap(); }".into()),
            ("print", 1) => {
                if let ParseNode::StringLit(s) = &args[0] {
                    let escaped: String = s.chars().map(|c| escape_char(c)).collect();
                    Some(format!(
                        "{{ print!(\"{escaped}\"); io::stdout().flush().unwrap(); }}"
                    ))
                } else {
                    Some(format!(
                        "{{ print!(\"{{}}\", {}); io::stdout().flush().unwrap(); }}",
                        a[0]
                    ))
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
                Some(format!(
                    "{{ print!({}, {}); io::stdout().flush().unwrap(); }}",
                    fmt_lit, rest
                ))
            }
            ("input", _) => {
                let vars = &args[1..];
                if vars.is_empty() {
                    return Some(
                        "{ let mut __ln = String::new(); \
                         io::stdin().lock().read_line(&mut __ln).unwrap(); }"
                            .into(),
                    );
                }
                let mut stmts = String::from(
                    "{ let mut __ln = String::new(); \
                     io::stdin().lock().read_line(&mut __ln).unwrap(); \
                     let mut __toks = __ln.trim().split_whitespace(); ",
                );
                for var_node in vars {
                    let (var_name, var_type) =
                        if let ParseNode::AccessChain { base, steps } = var_node {
                            if steps.is_empty() {
                                let ty = self
                                    .var_types
                                    .get(base.as_str())
                                    .cloned()
                                    .or_else(|| self.local_var_types.get(base.as_str()).cloned())
                                    .unwrap_or(SemType::Unknown);
                                (escape_ident(base), ty)
                            } else {
                                let lv = self.emit_access_chain_mut(base, steps);
                                (lv, SemType::Int)
                            }
                        } else {
                            continue;
                        };

                    let parse_expr = match var_type {
                        SemType::Int => {
                            "__toks.next().unwrap_or(\"\").parse::<i64>().unwrap_or(0_i64)"
                        }
                        SemType::Float => {
                            "__toks.next().unwrap_or(\"\").parse::<f64>().unwrap_or(0.0_f64)"
                        }
                        SemType::Char => {
                            "__toks.next().unwrap_or(\"\").chars().next().unwrap_or('\\0')"
                        }
                        SemType::Boolean => {
                            "matches!(__toks.next().unwrap_or(\"\"), \"true\" | \"1\")"
                        }
                        _ => "__toks.next().unwrap_or(\"\").parse::<i64>().unwrap_or(0_i64)",
                    };
                    stmts.push_str(&format!("{} = {}; ", var_name, parse_expr));
                }
                stmts.push('}');
                Some(stmts)
            }
            ("append", 2) => {
                let container = self.gen_list_container(&args[0]);
                let val = self.gen_expr(&args[1]);
                Some(format!("{}.push({}.clone())", container, val))
            }
            ("pop", 1) => {
                let container = self.gen_list_container(&args[0]);
                Some(format!("{}.pop().unwrap()", container))
            }
            ("insert", 2) => {
                let container = self.gen_list_container(&args[0]);
                let val = self.gen_call_arg(&args[1]);
                Some(format!("{}.insert(0, {})", container, val))
            }
            ("insert", 3) => {
                let container = self.gen_list_container(&args[0]);
                let idx = self.gen_expr(&args[1]);
                let val = self.gen_call_arg(&args[2]);
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

    fn emit_snapshot(&mut self, stmt: &ParseNode) {
        let label = stmt_debug_label(stmt);
        let source_line = stmt_source_line(stmt);
        let step = self.debug_step;
        self.debug_step += 1;
        let func = self.debug_current_func.clone();
        let vars_code = self.build_vars_json_code();
        let line = format!(
            "__fractal_debug_snapshot!({s}, \"{lb}\", \"{fc}\", {ln}, [{vrs}], false, None::<&str>);",
            s = step,
            lb = label.replace('"', "'"),
            fc = func,
            ln = source_line,
            vrs = vars_code,
        );
        self.line(&line);
    }

    fn build_vars_json_code(&self) -> String {
        self.debug_visible_vars
            .iter()
            .map(|(ident, type_label)| {
                format!(
                    "__fractal_debug_var(\"{name}\", \"{tl}\", &{{ let __v = &{ident}; format!(\"{{:?}}\", __v) }})",
                    name = ident,
                    tl = type_label,
                    ident = ident,
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn emit_debug_runtime(&mut self) {
        let path = self.debug_path.clone();

        self.line("use std::sync::{Mutex, Once};");
        self.line("use std::fs::{OpenOptions, File as __DbgFile};");
        self.line("use std::io::{BufWriter as __DbgBufWriter, Write as __DbgWrite};");
        self.blank();

        self.line("static __FRACTAL_DBG_INIT: Once = Once::new();");
        self.line("#[allow(clippy::type_complexity)]");
        self.line("static __FRACTAL_DBG_FILE: Mutex<Option<__DbgBufWriter<__DbgFile>>> = Mutex::new(None);");

        self.line("static __FRACTAL_DBG_PREV: std::sync::OnceLock<Mutex<std::collections::HashMap<String, String>>> = std::sync::OnceLock::new();");
        self.blank();

        self.line("fn __fractal_debug_init() {");
        self.indent();
        self.line("__FRACTAL_DBG_INIT.call_once(|| {");
        self.indent();
        self.line(&format!(
            "let __f = OpenOptions::new().create(true).write(true).truncate(true)\
             .open(\"{path}\").expect(\"cannot open fractal debug file\");",
            path = path
        ));
        self.line("*__FRACTAL_DBG_FILE.lock().unwrap() = Some(__DbgBufWriter::new(__f));");
        self.dedent();
        self.line("});");
        self.dedent();
        self.line("}");
        self.blank();

        self.line("fn __fractal_debug_json_escape(s: &str) -> String {");
        self.indent();
        self.line("let mut o = String::new();");
        self.line("for c in s.chars() {");
        self.indent();
        self.line("match c {");
        self.indent();
        self.line(r#"'"'  => o.push_str("\\\""),"#);
        self.line(r#"'\\' => o.push_str("\\\\"),"#);
        self.line(r#"'\n' => o.push_str("\\n"),"#);
        self.line(r#"'\t' => o.push_str("\\t"),"#);
        self.line(r#"'\r' => o.push_str("\\r"),"#);
        self.line("c    => o.push(c),");
        self.dedent();
        self.line("}");
        self.dedent();
        self.line("}");
        self.line("o");
        self.dedent();
        self.line("}");
        self.blank();

        self.line("fn __fractal_debug_var(name: &str, type_label: &str, value: &str) -> String {");
        self.indent();
        self.line("let changed = {");
        self.indent();

        self.line("let mutex = __FRACTAL_DBG_PREV.get_or_init(|| Mutex::new(std::collections::HashMap::new()));");
        self.line("let mut prev_map = mutex.lock().unwrap();");
        self.line("let prev = prev_map.get(name).cloned().unwrap_or_default();");
        self.line("let did_change = value != prev.as_str();");
        self.line("prev_map.insert(name.to_string(), value.to_string());");
        self.line("did_change");
        self.dedent();
        self.line("};");
        self.line(r#"let mut s = String::from("{");"#);
        self.line(r#"s.push_str("\"name\":\""); s.push_str(&__fractal_debug_json_escape(name)); s.push_str("\",");"#);
        self.line(r#"s.push_str("\"type\":\""); s.push_str(&__fractal_debug_json_escape(type_label)); s.push_str("\",");"#);
        self.line(r#"s.push_str("\"value\":\""); s.push_str(&__fractal_debug_json_escape(value)); s.push_str("\",");"#);
        self.line(r#"s.push_str("\"changed\":"); s.push_str(if changed { "true" } else { "false" }); s.push('}');"#);
        self.line("s");
        self.dedent();
        self.line("}");
        self.blank();

        self.raw("macro_rules! __fractal_debug_snapshot {\n");
        self.raw("    ($step:expr, $label:expr, $func:expr, $line:expr, [$($var_str:expr),* $(,)?], $finished:expr, $error:expr) => {{\n");
        self.raw("        let mut __dbg_g = __FRACTAL_DBG_FILE.lock().unwrap();\n");
        self.raw("        if let Some(ref mut __dbg_w) = *__dbg_g {\n");
        self.raw("            let __dbg_vars: Vec<String> = vec![$($var_str),*];\n");
        self.raw("            let __dbg_scope = {\n");
        self.raw("                let mut __sc = String::from(\"{\\\"label\\\":\\\"\");\n");
        self.raw("                __sc.push_str(&__fractal_debug_json_escape($func));\n");
        self.raw("                __sc.push_str(\"\\\",\\\"vars\\\":[\");\n");
        self.raw("                __sc.push_str(&__dbg_vars.join(\",\"));\n");
        self.raw("                __sc.push_str(\"]}\");\n");
        self.raw("                __sc\n");
        self.raw("            };\n");
        self.raw("            let __dbg_err: String = match ($error as Option<&str>) {\n");
        self.raw("                None      => \"null\".into(),\n");
        self.raw("                Some(__e) => { let mut __es = String::from(\"\\\"\"); __es.push_str(&__fractal_debug_json_escape(__e)); __es.push('\"'); __es },\n");
        self.raw("            };\n");
        self.raw("            let __dbg_line = {\n");
        self.raw("                let mut __ln = String::from(\"{\\\"step\\\":\");\n");
        self.raw("                __ln.push_str(&$step.to_string());\n");
        self.raw("                __ln.push_str(\",\\\"label\\\":\\\"\");\n");
        self.raw("                __ln.push_str(&__fractal_debug_json_escape($label));\n");

        self.raw("                __ln.push_str(\"\\\",\\\"line\\\":\");\n");
        self.raw("                __ln.push_str(&($line as usize).to_string());\n");
        self.raw("                __ln.push_str(\",\\\"stack\\\":[\\\"\");\n");
        self.raw("                __ln.push_str(&__fractal_debug_json_escape($func));\n");
        self.raw("                __ln.push_str(\"\\\"],\\\"scopes\\\":[\");\n");
        self.raw("                __ln.push_str(&__dbg_scope);\n");
        self.raw(
            "                __ln.push_str(\"],\\\"output\\\":\\\"\\\",\\\"finished\\\":\");\n",
        );
        self.raw("                __ln.push_str(if $finished { \"true\" } else { \"false\" });\n");
        self.raw("                __ln.push_str(\",\\\"error\\\":\");\n");
        self.raw("                __ln.push_str(&__dbg_err);\n");
        self.raw("                __ln.push('}');\n");
        self.raw("                __ln\n");
        self.raw("            };\n");
        self.raw("            let _ = writeln!(__dbg_w, \"{}\", __dbg_line);\n");
        self.raw("            let _ = __dbg_w.flush();\n");
        self.raw("        }\n");
        self.raw("    }};\n");
        self.raw("}\n");
    }
}

fn block_contains_break(stmts: &[ParseNode]) -> bool {
    stmts.iter().any(node_contains_break)
}

fn node_contains_break(node: &ParseNode) -> bool {
    match node {
        ParseNode::Break => true,

        ParseNode::If {
            then_block,
            else_block,
            ..
        } => {
            block_contains_break(then_block)
                || else_block.as_deref().map_or(false, block_contains_break)
        }

        ParseNode::For { .. } | ParseNode::While { .. } => false,
        _ => false,
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

fn escape_struct_name(name: &str) -> String {
    const RUST_BUILTIN_TYPES: &[&str] = &[
        "Box",
        "Vec",
        "Option",
        "Result",
        "String",
        "Ok",
        "Err",
        "Some",
        "None",
        "Drop",
        "Copy",
        "Clone",
        "Send",
        "Sync",
        "Sized",
        "Fn",
        "FnMut",
        "FnOnce",
        "Iterator",
        "Default",
        "From",
        "Into",
        "ToString",
        "AsRef",
        "AsMut",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ];
    if RUST_BUILTIN_TYPES.contains(&name) {
        format!("Fr{}", name)
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

fn stmt_debug_label(node: &ParseNode) -> String {
    match node {
        ParseNode::Decl {
            name,
            data_type,
            init,
        } => format!(
            "Decl {} : {}{}",
            name,
            parse_node_type_label(data_type),
            if init.is_some() { " =" } else { "" }
        ),
        ParseNode::StructDecl {
            var_name,
            struct_name,
            ..
        } => {
            format!("StructDecl {} : {}", var_name, struct_name)
        }
        ParseNode::Assign { op, .. } => format!("Assign {:?}", op),
        ParseNode::If { .. } => "If".into(),
        ParseNode::For { var_name, .. } => format!("For {}", var_name),
        ParseNode::While { .. } => "While".into(),
        ParseNode::Return(_) => "Return".into(),
        ParseNode::Exit(_) => "Exit".into(),
        ParseNode::ExprStmt(_) => "ExprStmt".into(),
        _ => "stmt".into(),
    }
}

fn stmt_source_line(node: &ParseNode) -> usize {
    match node {
        _ => 0,
    }
}

fn parse_node_type_label(node: &ParseNode) -> String {
    match node {
        ParseNode::TypeInt => ":int".into(),
        ParseNode::TypeFloat => ":float".into(),
        ParseNode::TypeChar => ":char".into(),
        ParseNode::TypeBoolean => ":bool".into(),
        ParseNode::TypeVoid => ":void".into(),
        ParseNode::TypeArray { .. } => ":array".into(),
        ParseNode::TypeList { .. } => ":list".into(),
        ParseNode::TypeStruct { name } => format!(":struct<{}>", name),
        _ => "?".into(),
    }
}
