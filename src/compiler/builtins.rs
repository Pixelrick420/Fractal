/// Single source of truth for every built-in function.
///
/// Both the semantic analyser and the code generator derive everything they
/// need from this table:
///
///   * `BuiltinDef::params` / `::ret`  → semanter converts via
///                                        `semanter::sem_type_from_btype`
///                                        and registers these in scope
///   * `BuiltinDef::codegen`           → codegen pattern-matches on this to
///                                        emit the right Rust expression
///
/// To add a new built-in:
///   1. Add one `BuiltinDef` entry to `ALL_BUILTINS`.
///   2. If the expansion is a simple Rust template, use `CodegenRule::Template`.
///      `{0}`, `{1}`, … are replaced with the generated argument strings.
///   3. If it needs special argument handling, add a `CodegenRule` variant and
///      handle it in `codegen::try_builtin`.

// ── Lightweight type descriptor (avoids circular dep with semanter) ──────────

/// Parameter / return type for a built-in function.
/// Converted to `SemType` by `semanter::sem_type_from_btype`.
#[derive(Debug, Clone, PartialEq)]
pub enum BType {
    Int,
    Float,
    Boolean,
    Char,
    Void,
    ListOfChar,
    /// Wildcard — the semanter accepts any type here; also signals variadic arity.
    Any,
}

// ── Code-generation rule ─────────────────────────────────────────────────────

/// Describes how the code generator should expand a call to this built-in.
#[derive(Debug, Clone)]
pub enum CodegenRule {
    /// Rust expression template. `{0}`, `{1}`, … replaced with arg strings.
    Template(&'static str),
    /// `print(fmt, …)` — special handling for format strings + arity.
    Print,
    /// `input(…)` — reads a line from stdin.
    Input,
    /// `append(list, value)` — Vec push; auto-clones struct values.
    Append,
    /// `pop(list)` — Vec pop.
    Pop,
    /// `insert(list [, idx], value)` — Vec insert (2- or 3-arg).
    Insert,
    /// `delete(list, idx)` — Vec remove by index.
    Delete,
}

// ── Builtin descriptor ────────────────────────────────────────────────────────

pub struct BuiltinDef {
    pub name: &'static str,
    /// Parameter types for the semanter. A single `BType::Any` = variadic.
    pub params: &'static [BType],
    pub ret: BType,
    pub codegen: CodegenRule,
}

// ── The table ─────────────────────────────────────────────────────────────────

pub static ALL_BUILTINS: &[BuiltinDef] = &[
    // ── I/O ──────────────────────────────────────────────────────────────────
    BuiltinDef {
        name: "print",
        params: &[BType::Any],
        ret: BType::Void,
        codegen: CodegenRule::Print,
    },
    BuiltinDef {
        name: "input",
        params: &[BType::Any],
        ret: BType::Void,
        codegen: CodegenRule::Input,
    },
    // ── List operations ───────────────────────────────────────────────────────
    BuiltinDef {
        name: "append",
        params: &[BType::Any, BType::Any],
        ret: BType::Void,
        codegen: CodegenRule::Append,
    },
    BuiltinDef {
        name: "pop",
        params: &[BType::Any],
        ret: BType::Any,
        codegen: CodegenRule::Pop,
    },
    BuiltinDef {
        name: "insert",
        params: &[BType::Any, BType::Any],
        ret: BType::Void,
        codegen: CodegenRule::Insert,
    },
    BuiltinDef {
        name: "delete",
        params: &[BType::Any, BType::Int],
        ret: BType::Void,
        codegen: CodegenRule::Delete,
    },
    BuiltinDef {
        name: "find",
        params: &[BType::Any, BType::Any],
        ret: BType::Int,
        codegen: CodegenRule::Template(
            "{0}.iter().position(|__x| *__x == {1}).map(|i| i as i64).unwrap_or(-1_i64)",
        ),
    },
    BuiltinDef {
        name: "len",
        params: &[BType::Any],
        ret: BType::Int,
        codegen: CodegenRule::Template("({0}.len() as i64)"),
    },
    // ── Math ──────────────────────────────────────────────────────────────────
    BuiltinDef {
        name: "abs",
        params: &[BType::Any],
        ret: BType::Any,
        codegen: CodegenRule::Template("{0}.abs()"),
    },
    BuiltinDef {
        name: "sqrt",
        params: &[BType::Float],
        ret: BType::Float,
        codegen: CodegenRule::Template("({0} as f64).sqrt()"),
    },
    BuiltinDef {
        name: "pow",
        params: &[BType::Float, BType::Float],
        ret: BType::Float,
        codegen: CodegenRule::Template("({0} as f64).powf({1} as f64)"),
    },
    BuiltinDef {
        name: "floor",
        params: &[BType::Float],
        ret: BType::Int,
        codegen: CodegenRule::Template("({0} as f64).floor() as i64"),
    },
    BuiltinDef {
        name: "ceil",
        params: &[BType::Float],
        ret: BType::Int,
        codegen: CodegenRule::Template("({0} as f64).ceil() as i64"),
    },
    BuiltinDef {
        name: "min",
        params: &[BType::Any, BType::Any],
        ret: BType::Any,
        codegen: CodegenRule::Template("{0}.min({1})"),
    },
    BuiltinDef {
        name: "max",
        params: &[BType::Any, BType::Any],
        ret: BType::Any,
        codegen: CodegenRule::Template("{0}.max({1})"),
    },
    // ── Conversions ───────────────────────────────────────────────────────────
    BuiltinDef {
        name: "to_int",
        params: &[BType::Any],
        ret: BType::Int,
        codegen: CodegenRule::Template("({0} as i64)"),
    },
    BuiltinDef {
        name: "to_float",
        params: &[BType::Any],
        ret: BType::Float,
        codegen: CodegenRule::Template("({0} as f64)"),
    },
    BuiltinDef {
        name: "to_str",
        params: &[BType::Any],
        ret: BType::ListOfChar,
        codegen: CodegenRule::Template("{0}.to_string().chars().collect::<Vec<char>>()"),
    },
];
