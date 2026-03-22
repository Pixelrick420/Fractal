#[derive(Debug, Clone, PartialEq)]
pub enum BType {
    Int,
    Float,
    Boolean,
    Char,
    Void,
    ListOfChar,

    Any,
}

#[derive(Debug, Clone)]
pub enum CodegenRule {
    Template(&'static str),

    Print,

    Input,

    Append,

    Pop,

    Insert,

    Delete,
}

pub struct BuiltinDef {
    pub name: &'static str,

    pub params: &'static [BType],
    pub ret: BType,
    pub codegen: CodegenRule,
}

pub static ALL_BUILTINS: &[BuiltinDef] = &[
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
        params: &[BType::Any, BType::Any, BType::Int],
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
