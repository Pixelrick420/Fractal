use crate::compiler::lexer::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum ParseNode {
    Program(Vec<ParseNode>),

    Module {
        name: String,
        items: Vec<ParseNode>,
    },

    FuncDef {
        name: String,
        params: Vec<ParseNode>,
        return_type: Box<ParseNode>,
        body: Vec<ParseNode>,
    },

    Param {
        data_type: Box<ParseNode>,
        name: String,
    },

    StructDef {
        name: String,
        fields: Vec<ParseNode>,
    },

    StructDecl {
        struct_name: String,
        var_name: String,
        init: Option<Box<ParseNode>>,
        line: usize,
    },

    Field {
        data_type: Box<ParseNode>,
        name: String,
    },

    Decl {
        data_type: Box<ParseNode>,
        name: String,
        init: Option<Box<ParseNode>>,
        line: usize,
    },

    Assign {
        lvalue: Box<ParseNode>,
        op: AssignOp,
        expr: Box<ParseNode>,
        line: usize,
    },

    If {
        condition: Box<ParseNode>,
        then_block: Vec<ParseNode>,
        else_block: Option<Vec<ParseNode>>,
        line: usize,
    },

    For {
        var_type: Box<ParseNode>,
        var_name: String,
        start: Box<ParseNode>,
        stop: Box<ParseNode>,
        step: Box<ParseNode>,
        body: Vec<ParseNode>,
        line: usize,
    },

    While {
        condition: Box<ParseNode>,
        body: Vec<ParseNode>,
        line: usize,
    },

    Return {
        expr: Box<ParseNode>,
        line: usize,
    },
    Exit {
        expr: Box<ParseNode>,
        line: usize,
    },
    Break {
        line: usize,
    },
    Continue {
        line: usize,
    },

    ExprStmt(Box<ParseNode>, usize),

    AccessChain {
        base: String,
        steps: Vec<AccessStep>,
        line: usize,
    },

    LogOr {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
        line: usize,
    },
    LogAnd {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
        line: usize,
    },
    LogNot {
        operand: Box<ParseNode>,
        line: usize,
    },

    Cmp {
        left: Box<ParseNode>,
        op: CmpOp,
        right: Box<ParseNode>,
        line: usize,
    },

    BitOr {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
        line: usize,
    },
    BitXor {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
        line: usize,
    },
    BitAnd {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
        line: usize,
    },
    BitShift {
        left: Box<ParseNode>,
        op: ShiftOp,
        right: Box<ParseNode>,
        line: usize,
    },

    Add {
        left: Box<ParseNode>,
        op: AddOp,
        right: Box<ParseNode>,
        line: usize,
    },
    Mul {
        left: Box<ParseNode>,
        op: MulOp,
        right: Box<ParseNode>,
        line: usize,
    },

    Unary {
        op: UnOp,
        operand: Box<ParseNode>,
        line: usize,
    },

    Cast {
        target_type: Box<ParseNode>,
        expr: Box<ParseNode>,
        line: usize,
    },

    ArrayLit(Vec<ParseNode>, usize),

    StructLit(Vec<(String, ParseNode)>, usize),

    Identifier(String, usize),
    IntLit(i64, usize),
    FloatLit(f64, usize),
    CharLit(char, usize),
    StringLit(String, usize),
    BoolLit(bool, usize),
    Null(usize),

    TypeInt(usize),
    TypeFloat(usize),
    TypeChar(usize),
    TypeBoolean(usize),
    TypeVoid(usize),
    TypeArray {
        elem: Box<ParseNode>,
        size: i64,
        line: usize,
    },
    TypeList {
        elem: Box<ParseNode>,
        line: usize,
    },
    TypeStruct {
        name: String,
        line: usize,
    },
}

#[derive(Debug, Clone)]
pub enum AccessStep {
    Field(String),

    Index(Box<ParseNode>),

    Call(Vec<ParseNode>),
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    AmpEq,
    PipeEq,
    CaretEq,
}

#[derive(Debug, Clone)]
pub enum CmpOp {
    Gt,
    Lt,
    Ge,
    Le,
    EqEq,
    Ne,
}

#[derive(Debug, Clone)]
pub enum AddOp {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum MulOp {
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone)]
pub enum ShiftOp {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum UnOp {
    Neg,
    BitNot,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    source_file: String,
    func_depth: usize,
    loop_depth: usize,
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

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub source_file: String,
}

impl ParseError {
    fn new_at(msg: impl Into<String>, line: usize, col: usize, source_file: &str) -> Self {
        ParseError {
            message: msg.into(),
            line,
            col,
            source_file: source_file.to_string(),
        }
    }

    pub fn emit(&self, _preprocessed: &str) {
        let display_file = if self.source_file.is_empty() {
            "<unknown>".to_string()
        } else {
            std::path::Path::new(&self.source_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&self.source_file)
                .to_string()
        };

        let mut lines = self.message.splitn(2, '\n');
        let main_msg = lines.next().unwrap_or(&self.message);
        let rest = lines.next().unwrap_or("");

        eprintln!("\x1b[1;31merror[P000]\x1b[0m\x1b[1m: {}\x1b[0m", main_msg);

        if self.line == 0 {
            eprintln!(" \x1b[1;34m-->\x1b[0m {}", display_file);
            format_notes(rest);
            eprintln!();
            return;
        }

        let original = if !self.source_file.is_empty() {
            std::fs::read_to_string(&self.source_file).unwrap_or_default()
        } else {
            String::new()
        };

        let src_line = original.lines().nth(self.line - 1).unwrap_or("");
        let line_str = self.line.to_string();
        let pad = " ".repeat(line_str.len());
        let caret_pad = " ".repeat(self.col.saturating_sub(1));

        eprintln!(
            " \x1b[1;34m-->\x1b[0m {}:{}:{}",
            display_file, self.line, self.col
        );
        eprintln!(" \x1b[1;34m{pad} |\x1b[0m");
        eprintln!(" \x1b[1;34m{line_str} |\x1b[0m {src_line}");

        let token_len = src_line
            .get(self.col.saturating_sub(1)..)
            .map(|s| {
                s.find(|c: char| c.is_whitespace() || "(){}[];,".contains(c))
                    .filter(|&n| n > 0)
                    .unwrap_or_else(|| s.len().max(1))
            })
            .unwrap_or(1);
        let underline = "^".repeat(token_len);
        eprintln!(" \x1b[1;34m{pad} |\x1b[0m \x1b[1;31m{caret_pad}{underline} unexpected token here\x1b[0m");
        format_notes(rest);
        eprintln!();
    }
}

type PResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(tokens: Vec<Token>, source_file: impl Into<String>) -> Self {
        Parser {
            tokens,
            pos: 0,
            source_file: source_file.into(),
            func_depth: 0,
            loop_depth: 0,
        }
    }

    fn peek(&self) -> Option<&TokenType> {
        self.tokens.get(self.pos).map(|t| &t.token_type)
    }

    fn cur_line(&self) -> usize {
        self.tokens.get(self.pos).map(|t| t.line).unwrap_or(0)
    }

    fn cur_col(&self) -> usize {
        self.tokens.get(self.pos).map(|t| t.col).unwrap_or(0)
    }

    fn advance(&mut self) -> Option<&TokenType> {
        let t = self.tokens.get(self.pos).map(|t| &t.token_type);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn err(&self, msg: impl Into<String>) -> ParseError {
        ParseError::new_at(msg, self.cur_line(), self.cur_col(), &self.source_file)
    }

    fn token_name(tt: &TokenType) -> &'static str {
        match tt {
            TokenType::EndL => "`;`",
            TokenType::LParen => "`(`",
            TokenType::RParen => "`)`",
            TokenType::LBrace => "`{`",
            TokenType::RBrace => "`}`",
            TokenType::LBracket => "`[`",
            TokenType::RBracket => "`]`",
            TokenType::Less => "`<`",
            TokenType::Greater => "`>`",
            TokenType::Comma => "`,`",
            TokenType::Equals => "`=`",
            TokenType::Arrow => "`->`",
            TokenType::ColonColon => "`::`",
            TokenType::Dot => "`.`",
            TokenType::Plus => "`+`",
            TokenType::Minus => "`-`",
            TokenType::Star => "`*`",
            TokenType::Slash => "`/`",
            TokenType::Percent => "`%`",
            TokenType::Ampersand => "`&`",
            TokenType::Pipe => "`|`",
            TokenType::Caret => "`^`",
            TokenType::Tilde => "`~`",
            TokenType::PlusEquals => "`+=`",
            TokenType::MinusEquals => "`-=`",
            TokenType::StarEquals => "`*=`",
            TokenType::SlashEquals => "`/=`",
            TokenType::PercentEquals => "`%=`",
            TokenType::AmpersandEquals => "`&=`",
            TokenType::PipeEquals => "`|=`",
            TokenType::CaretEquals => "`^=`",
            TokenType::EqualsEquals => "`==`",
            TokenType::TildeEquals => "`~=`",
            TokenType::GreaterEquals => "`>=`",
            TokenType::LessEquals => "`<=`",
            TokenType::Start => "`!start`",
            TokenType::End => "`!end`",
            TokenType::Exit => "`!exit`",
            TokenType::If => "`!if`",
            TokenType::Elif => "`!elif`",
            TokenType::Else => "`!else`",
            TokenType::For => "`!for`",
            TokenType::While => "`!while`",
            TokenType::Func => "`!func`",
            TokenType::Return => "`!return`",
            TokenType::Break => "`!break`",
            TokenType::Continue => "`!continue`",
            TokenType::Struct => "`!struct`",
            TokenType::Import => "`!import`",
            TokenType::Module => "`!module`",
            TokenType::And => "`!and`",
            TokenType::Or => "`!or`",
            TokenType::Not => "`!not`",
            TokenType::TypeInt => "`:int`",
            TokenType::TypeFloat => "`:float`",
            TokenType::TypeChar => "`:char`",
            TokenType::TypeBoolean => "`:boolean`",
            TokenType::TypeArray => "`:array`",
            TokenType::TypeList => "`:list`",
            TokenType::TypeStruct => "`:struct`",
            TokenType::TypeVoid => "`:void`",
            TokenType::Identifier(_) => "identifier",
            TokenType::SIntLit(_) => "integer literal",
            TokenType::FloatLit(_) => "float literal",
            TokenType::CharLit(_) => "char literal",
            TokenType::StringLit(_) => "string literal",
            TokenType::BoolLit(_) => "`true` or `false`",
            TokenType::Null => "`!null`",
            TokenType::ModuleStart(_) => "module-start marker",
            TokenType::ModuleEnd(_) => "module-end marker",
            TokenType::FileMap(_, _) => "file-map marker",
            TokenType::NoMatch => "<unrecognised token>",
        }
    }

    fn opt_token_name(tt: Option<&TokenType>) -> String {
        match tt {
            Some(t) => Self::token_name(t).to_string(),
            None => "end of file".to_string(),
        }
    }

    fn expect(&mut self, expected: &TokenType) -> PResult<()> {
        match self.peek() {
            Some(tt) if tt == expected => {
                self.advance();
                Ok(())
            }
            other => {
                let found = Self::opt_token_name(other);
                let want = Self::token_name(expected);
                let msg = match expected {
                    TokenType::EndL =>
                        format!("expected `;` to end the statement, but found {found}\n   \
                                 note: every declaration and simple statement must end with `;`"),
                    TokenType::Arrow =>
                        format!("expected `->` before the return type, but found {found}\n   \
                                 note: function syntax is `!func name(params) -> :type {{ ... }}`"),
                    TokenType::LParen =>
                        format!("expected `(` here, but found {found}"),
                    TokenType::RParen =>
                        format!("expected `)` to close the argument list or condition, but found {found}"),
                    TokenType::LBrace =>
                        format!("expected `{{` to open a block, but found {found}\n   \
                                 note: the opening `{{` must be on the same line as the statement"),
                    TokenType::RBrace =>
                        format!("expected `}}` to close a block, but found {found}\n   \
                                 note: every `{{` must have a matching `}}`"),
                    TokenType::LBracket =>
                        format!("expected `[` here, but found {found}"),
                    TokenType::RBracket =>
                        format!("expected `]` to close the index or array literal, but found {found}"),
                    TokenType::Less =>
                        format!("expected `<` to open a type parameter list, but found {found}\n   \
                                 note: generic types are written as `:array<:int, 5>` or `:list<:float>`"),
                    TokenType::Greater =>
                        format!("expected `>` to close the type parameter list, but found {found}"),
                    TokenType::Comma =>
                        format!("expected `,` to separate items, but found {found}\n   \
                                 note: for-loop syntax is `!for (:type var, start, stop, step) {{ }}`"),
                    TokenType::Equals =>
                        format!("expected `=` for assignment, but found {found}"),
                    TokenType::Start =>
                        format!("expected `!start` at the beginning of the program, but found {found}\n   \
                                 note: every Fractal program must begin with `!start` and end with `!end`"),
                    TokenType::End =>
                        format!("expected `!end` to close the program, but found {found}\n   \
                                 note: every Fractal program must begin with `!start` and end with `!end`"),
                    _ =>
                        format!("expected {want}, but found {found}"),
                };
                Err(self.err(msg))
            }
        }
    }

    fn expect_identifier(&mut self) -> PResult<String> {
        match self.peek().cloned() {
            Some(TokenType::Identifier(s)) => {
                self.advance();
                Ok(s)
            }
            other => {
                let found = Self::opt_token_name(other.as_ref());
                Err(self.err(format!(
                    "expected an identifier (a name) here, but found {found}\n   \
                     note: identifiers must start with a letter or `_` and contain only letters, digits, and `_`"
                )))
            }
        }
    }

    fn expect_int_lit(&mut self) -> PResult<i64> {
        match self.peek().cloned() {
            Some(TokenType::SIntLit(n)) => {
                self.advance();
                if n <= 0 {
                    return Err(self.err(format!(
                        "array size must be a positive integer greater than zero, got {n}\n   \
                         note: arrays need at least one element; use a size like `:array<:int, 5>`"
                    )));
                }
                Ok(n)
            }
            other => {
                let found = Self::opt_token_name(other.as_ref());
                Err(self.err(format!(
                    "expected an integer literal for the array size, but found {found}\n   \
                     note: array size must be a compile-time integer constant, e.g. `:array<:int, 5>`"
                )))
            }
        }
    }

    fn at_endl(&self) -> bool {
        matches!(self.peek(), Some(TokenType::EndL))
    }

    fn parse_struct_type_name(&mut self) -> PResult<String> {
        let first = self.expect_identifier()?;
        if matches!(self.peek(), Some(TokenType::ColonColon)) {
            self.advance();
            let second = self.expect_identifier()?;
            Ok(format!("{}::{}", first, second))
        } else {
            Ok(first)
        }
    }

    pub fn parse_program(&mut self) -> PResult<ParseNode> {
        self.expect(&TokenType::Start)?;
        let items = self.parse_item_list()?;
        self.expect(&TokenType::End)?;
        Ok(ParseNode::Program(items))
    }

    fn parse_item_list(&mut self) -> PResult<Vec<ParseNode>> {
        let mut items = Vec::new();
        loop {
            match self.peek() {
                None | Some(TokenType::End) | Some(TokenType::ModuleEnd(_)) => break,
                _ => items.push(self.parse_item()?),
            }
        }
        Ok(items)
    }

    fn parse_item(&mut self) -> PResult<ParseNode> {
        match self.peek().cloned() {
            Some(TokenType::ModuleStart(name)) => {
                self.advance();
                let items = self.parse_item_list()?;
                match self.peek().cloned() {
                    Some(TokenType::ModuleEnd(end_name)) => {
                        self.advance();
                        if end_name != name {
                            return Err(self.err(format!(
                                "module name mismatch: opened with `$MODULE_START:{name}$` \
                                 but closed with `$MODULE_END:{end_name}$`\n   \
                                 note: the name in the closing marker must exactly match the opening marker"
                            )));
                        }
                        if self.at_endl() {
                            self.advance();
                        }
                        Ok(ParseNode::Module { name, items })
                    }
                    other => Err(self.err(format!(
                        "expected the end of module `{name}`, but found {}\n   \
                         note: every `$MODULE_START:name$` must have a matching `$MODULE_END:name$`",
                        Self::opt_token_name(other.as_ref())
                    ))),
                }
            }

            Some(TokenType::Func) => self.parse_funcdef(),

            Some(TokenType::TypeStruct) => self.parse_struct_item(true),

            Some(t) if Self::is_type_token(&t) => {
                let node = self.parse_decl()?;
                self.expect(&TokenType::EndL)?;
                Ok(node)
            }

            _ => {
                let stmt = self.parse_stmt()?;
                if !matches!(
                    stmt,
                    ParseNode::If { .. }
                        | ParseNode::For { .. }
                        | ParseNode::While { .. }
                        | ParseNode::FuncDef { .. }
                        | ParseNode::StructDef { .. }
                        | ParseNode::StructDecl { .. }
                ) {
                    if self.at_endl() {
                        self.advance();
                    }
                }
                Ok(stmt)
            }
        }
    }

    fn is_type_token(tt: &TokenType) -> bool {
        matches!(
            tt,
            TokenType::TypeInt
                | TokenType::TypeFloat
                | TokenType::TypeChar
                | TokenType::TypeBoolean
                | TokenType::TypeVoid
                | TokenType::TypeArray
                | TokenType::TypeList
                | TokenType::TypeStruct
        )
    }

    fn parse_funcdef(&mut self) -> PResult<ParseNode> {
        if self.func_depth > 0 {
            return Err(self.err(
                "functions cannot be defined inside another function\n   \
                 note: move this `!func` definition to the top level, outside any `!func` body",
            ));
        }
        self.expect(&TokenType::Func)?;
        let name = self.expect_identifier()?;
        self.expect(&TokenType::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenType::RParen)?;
        self.expect(&TokenType::Arrow)?;
        let return_type = self.parse_datatype()?;
        self.func_depth += 1;
        let body = self.parse_block()?;
        self.func_depth -= 1;
        Ok(ParseNode::FuncDef {
            name,
            params,
            return_type: Box::new(return_type),
            body,
        })
    }

    fn parse_params(&mut self) -> PResult<Vec<ParseNode>> {
        let mut params = Vec::new();
        if matches!(self.peek(), Some(TokenType::RParen)) {
            return Ok(params);
        }
        params.push(self.parse_param()?);
        while matches!(self.peek(), Some(TokenType::Comma)) {
            self.advance();
            params.push(self.parse_param()?);
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> PResult<ParseNode> {
        let data_type = self.parse_datatype()?;
        let name = self.expect_identifier()?;
        Ok(ParseNode::Param {
            data_type: Box::new(data_type),
            name,
        })
    }

    fn parse_struct_item(&mut self, consume_endl: bool) -> PResult<ParseNode> {
        let line = self.cur_line();
        self.expect(&TokenType::TypeStruct)?;
        self.expect(&TokenType::Less)?;
        let type_name = self.parse_struct_type_name()?;
        self.expect(&TokenType::Greater)?;

        match self.peek().cloned() {
            Some(TokenType::LBrace) => {
                self.advance();
                let fields = self.parse_fields()?;
                self.expect(&TokenType::RBrace)?;
                self.expect(&TokenType::EndL)?;
                Ok(ParseNode::StructDef {
                    name: type_name,
                    fields,
                })
            }

            Some(TokenType::Identifier(var_name)) => {
                self.advance();
                let init = if matches!(self.peek(), Some(TokenType::Equals)) {
                    self.advance();
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };
                if consume_endl {
                    self.expect(&TokenType::EndL)?;
                }
                Ok(ParseNode::StructDecl {
                    struct_name: type_name,
                    var_name,
                    init,
                    line,
                })
            }

            other => Err(self.err(format!(
                "expected `{{` for a struct definition or a variable name for a struct declaration, \
                 but found {}\n   \
                 note: to define a struct:   `:struct<n> {{ :int field; }};`\n   \
                 note: to declare a variable: `:struct<n> var = {{ field = value }};`",
                Self::opt_token_name(other.as_ref())
            ))),
        }
    }

    fn parse_fields(&mut self) -> PResult<Vec<ParseNode>> {
        let mut fields = Vec::new();
        loop {
            if matches!(self.peek(), Some(TokenType::RBrace) | None) {
                break;
            }
            match self.peek().cloned() {
                Some(t) if Self::is_type_token(&t) => {
                    if matches!(t, TokenType::TypeStruct) {
                        fields.push(self.parse_struct_field()?);
                    } else {
                        let dt = self.parse_datatype()?;
                        let name = self.expect_identifier()?;
                        self.expect(&TokenType::EndL)?;
                        fields.push(ParseNode::Field {
                            data_type: Box::new(dt),
                            name,
                        });
                    }
                }
                _ => break,
            }
        }
        Ok(fields)
    }

    fn parse_struct_field(&mut self) -> PResult<ParseNode> {
        self.expect(&TokenType::TypeStruct)?;
        self.expect(&TokenType::Less)?;
        let type_name = self.parse_struct_type_name()?;
        self.expect(&TokenType::Greater)?;
        let field_name = self.expect_identifier()?;
        self.expect(&TokenType::EndL)?;
        Ok(ParseNode::Field {
            data_type: Box::new(ParseNode::TypeStruct {
                name: type_name,
                line: 0,
            }),
            name: field_name,
        })
    }

    fn parse_struct_lit_fields(&mut self) -> PResult<Vec<(String, ParseNode)>> {
        let mut fields = Vec::new();
        if matches!(self.peek(), Some(TokenType::RBrace)) {
            self.advance();
            return Ok(fields);
        }
        loop {
            let name = self.expect_identifier()?;
            self.expect(&TokenType::Equals)?;
            let val = self.parse_expression()?;
            fields.push((name, val));
            match self.peek().cloned() {
                Some(TokenType::Comma) => {
                    self.advance();
                }
                Some(TokenType::RBrace) => break,
                other => {
                    return Err(self.err(format!(
                        "expected `,` between fields or `}}` to end the struct literal, \
                         but found {}\n   \
                         note: struct literals look like `{{ field1 = val1, field2 = val2 }}`",
                        Self::opt_token_name(other.as_ref())
                    )))
                }
            }
        }
        self.expect(&TokenType::RBrace)?;
        Ok(fields)
    }

    fn parse_block(&mut self) -> PResult<Vec<ParseNode>> {
        self.expect(&TokenType::LBrace)?;
        let stmts = self.parse_stmts()?;
        self.expect(&TokenType::RBrace)?;
        Ok(stmts)
    }

    fn parse_stmts(&mut self) -> PResult<Vec<ParseNode>> {
        let mut stmts = Vec::new();
        loop {
            match self.peek() {
                None | Some(TokenType::RBrace) => break,
                _ => stmts.push(self.parse_stmt_with_endl()?),
            }
        }
        Ok(stmts)
    }

    fn parse_stmt_with_endl(&mut self) -> PResult<ParseNode> {
        let stmt = self.parse_stmt()?;
        if !matches!(
            stmt,
            ParseNode::If { .. }
                | ParseNode::For { .. }
                | ParseNode::While { .. }
                | ParseNode::FuncDef { .. }
                | ParseNode::StructDef { .. }
                | ParseNode::Return { .. }
                | ParseNode::Exit { .. }
                | ParseNode::Break { .. }
                | ParseNode::Continue { .. }
        ) {
            self.expect(&TokenType::EndL)?;
        }
        Ok(stmt)
    }

    fn keyword_hint(name: &str) -> Option<&'static str> {
        match name {
            "if" => Some("`if` is not valid here — did you mean `!if`?"),
            "elif" => Some("`elif` is not valid here — did you mean `!elif`?"),
            "else" => Some("`else` is not valid here — did you mean `!else`?"),
            "for" => Some("`for` is not valid here — did you mean `!for`?"),
            "while" => Some("`while` is not valid here — did you mean `!while`?"),
            "func" => Some("`func` is not valid here — did you mean `!func`?"),
            "return" => Some("`return` is not valid here — did you mean `!return`?"),
            "break" => Some("`break` is not valid here — did you mean `!break`?"),
            "continue" => Some("`continue` is not valid here — did you mean `!continue`?"),
            "import" => Some("`import` is not valid here — did you mean `!import`?"),
            "struct" => Some("`struct` is not valid here — did you mean `:struct`?"),
            "int" => Some("`int` is not valid here — did you mean `:int`?"),
            "float" => Some("`float` is not valid here — did you mean `:float`?"),
            "char" => Some("`char` is not valid here — did you mean `:char`?"),
            "boolean" => Some("`boolean` is not valid here — did you mean `:boolean`?"),
            "void" => Some("`void` is not valid here — did you mean `:void`?"),
            "array" => Some("`array` is not valid here — did you mean `:array`?"),
            "list" => Some("`list` is not valid here — did you mean `:list`?"),
            _ => None,
        }
    }

    fn parse_stmt(&mut self) -> PResult<ParseNode> {
        match self.peek().cloned() {
            Some(TokenType::Func) => Err(self.err(
                "functions cannot be defined inside a block\n   \
                     note: `!func` definitions must appear at the top level, \
                     outside any `!if`, `!for`, `!while`, or `!func` body",
            )),

            Some(TokenType::If) => {
                let line = self.cur_line();
                self.advance();
                self.expect(&TokenType::LParen)?;
                let condition = self.parse_expression()?;
                self.expect(&TokenType::RParen)?;
                let then_block = self.parse_block()?;
                let else_block = self.parse_else_tail()?;
                Ok(ParseNode::If {
                    condition: Box::new(condition),
                    then_block,
                    else_block,
                    line,
                })
            }

            Some(TokenType::For) => {
                let line = self.cur_line();
                self.advance();
                self.expect(&TokenType::LParen)?;

                let (var_type, var_name) = if Self::is_type_token_ref(self.peek()) {
                    let vt = self.parse_datatype()?;
                    let vn = self.expect_identifier()?;
                    (vt, vn)
                } else {
                    let vn = self.expect_identifier()?;
                    (ParseNode::TypeVoid(0), vn)
                };
                self.expect(&TokenType::Comma)?;
                let start = self.parse_expression()?;
                self.expect(&TokenType::Comma)?;
                let stop = self.parse_expression()?;
                self.expect(&TokenType::Comma)?;
                let step = self.parse_expression()?;
                self.expect(&TokenType::RParen)?;
                self.loop_depth += 1;
                let body = self.parse_block()?;
                self.loop_depth -= 1;
                Ok(ParseNode::For {
                    var_type: Box::new(var_type),
                    var_name,
                    start: Box::new(start),
                    stop: Box::new(stop),
                    step: Box::new(step),
                    body,
                    line,
                })
            }

            Some(TokenType::While) => {
                let line = self.cur_line();
                self.advance();
                self.expect(&TokenType::LParen)?;
                let condition = self.parse_expression()?;
                self.expect(&TokenType::RParen)?;
                self.loop_depth += 1;
                let body = self.parse_block()?;
                self.loop_depth -= 1;
                Ok(ParseNode::While {
                    condition: Box::new(condition),
                    body,
                    line,
                })
            }

            Some(TokenType::Return) => {
                let line = self.cur_line();
                let col = self.cur_col();
                self.advance();

                if self.func_depth == 0 {
                    return Err(ParseError::new_at(
                        "`!return` used outside of a function\n   \
                         note: `!return` can only appear inside a `!func` body\n   \
                         hint: did you accidentally place it at the top level?",
                        line,
                        col,
                        &self.source_file,
                    ));
                }
                if matches!(self.peek(), Some(TokenType::EndL)) {
                    return Err(self.err(
                        "bare `!return;` is not valid — a return value is required\n   \
                         note: every non-`:void` function must return a value: `!return <expr>;`\n   \
                         note: to return from a `:void` function use `!return !null;`",
                    ));
                }
                let expr = self.parse_expression()?;
                self.expect(&TokenType::EndL)?;
                Ok(ParseNode::Return {
                    expr: Box::new(expr),
                    line,
                })
            }

            Some(TokenType::Exit) => {
                let line = self.cur_line();
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenType::EndL)?;
                Ok(ParseNode::Exit {
                    expr: Box::new(expr),
                    line,
                })
            }

            Some(TokenType::Break) => {
                let line = self.cur_line();
                let col = self.cur_col();
                if self.loop_depth == 0 {
                    return Err(ParseError::new_at(
                        "`!break` used outside of a loop\n   \
                         note: `!break` can only appear inside a `!for` or `!while` body",
                        line,
                        col,
                        &self.source_file,
                    ));
                }
                self.advance();
                self.expect(&TokenType::EndL)?;
                Ok(ParseNode::Break { line })
            }

            Some(TokenType::Continue) => {
                let line = self.cur_line();
                let col = self.cur_col();
                if self.loop_depth == 0 {
                    return Err(ParseError::new_at(
                        "`!continue` used outside of a loop\n   \
                         note: `!continue` can only appear inside a `!for` or `!while` body",
                        line,
                        col,
                        &self.source_file,
                    ));
                }
                self.advance();
                self.expect(&TokenType::EndL)?;
                Ok(ParseNode::Continue { line })
            }

            Some(TokenType::TypeStruct) => self.parse_struct_item(false),

            Some(ref t) if Self::is_type_token(t) => self.parse_decl(),

            Some(TokenType::Identifier(name)) => {
                if let Some(hint) = Self::keyword_hint(&name) {
                    return Err(self.err(hint.to_string()));
                }
                self.parse_assign_or_expr_stmt()
            }

            _ => {
                let line = self.cur_line();
                let expr = self.parse_expression()?;
                Ok(ParseNode::ExprStmt(Box::new(expr), line))
            }
        }
    }

    fn parse_else_tail(&mut self) -> PResult<Option<Vec<ParseNode>>> {
        match self.peek().cloned() {
            Some(TokenType::Elif) => {
                let line = self.cur_line();
                self.advance();
                self.expect(&TokenType::LParen)?;
                let condition = self.parse_expression()?;
                self.expect(&TokenType::RParen)?;
                let then_block = self.parse_block()?;
                let else_block = self.parse_else_tail()?;
                let elif_node = ParseNode::If {
                    condition: Box::new(condition),
                    then_block,
                    else_block,
                    line,
                };
                Ok(Some(vec![elif_node]))
            }
            Some(TokenType::Else) => {
                self.advance();
                Ok(Some(self.parse_block()?))
            }
            _ => Ok(None),
        }
    }

    fn parse_decl(&mut self) -> PResult<ParseNode> {
        let line = self.cur_line();
        let data_type = self.parse_datatype()?;
        let name = self.expect_identifier()?;

        let compound_op = match self.peek() {
            Some(TokenType::PlusEquals) => Some("+="),
            Some(TokenType::MinusEquals) => Some("-="),
            Some(TokenType::StarEquals) => Some("*="),
            Some(TokenType::SlashEquals) => Some("/="),
            Some(TokenType::PercentEquals) => Some("%="),
            Some(TokenType::AmpersandEquals) => Some("&="),
            Some(TokenType::PipeEquals) => Some("|="),
            Some(TokenType::CaretEquals) => Some("^="),
            _ => None,
        };
        if let Some(op) = compound_op {
            return Err(self.err(format!(
                "`{name}` has not been declared yet, so `{op}` is not valid here\n   \
                 note: `{op}` requires the variable to already exist — \
                 you cannot declare and compound-assign in one step\n   \
                 hint: to declare with an initial value use `=`:  `:int {name} = <expr>;`\n   \
                 hint: if `{name}` was declared earlier, remove the type prefix: `{name} {op} <expr>;`"
            )));
        }

        let init = if matches!(self.peek(), Some(TokenType::Equals)) {
            self.advance();
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Ok(ParseNode::Decl {
            data_type: Box::new(data_type),
            name,
            init,
            line,
        })
    }

    fn parse_assign_or_expr_stmt(&mut self) -> PResult<ParseNode> {
        let line = self.cur_line();
        let saved = self.pos;

        if let Ok(chain) = self.try_parse_lvalue_chain() {
            if let Some(op) = self.try_parse_assignop() {
                let expr = self.parse_expression()?;
                return Ok(ParseNode::Assign {
                    lvalue: Box::new(chain),
                    op,
                    expr: Box::new(expr),
                    line,
                });
            }
        }

        self.pos = saved;
        let expr = self.parse_expression()?;
        Ok(ParseNode::ExprStmt(Box::new(expr), line))
    }

    fn try_parse_lvalue_chain(&mut self) -> PResult<ParseNode> {
        let line = self.cur_line();
        let name = self.expect_identifier()?;
        let steps = self.parse_postfix_steps()?;

        if let Some(AccessStep::Call(_)) = steps.last() {
            return Err(self.err(
                "the result of a function call cannot be assigned to\n   \
                 note: only variables, array elements, and struct fields are valid assignment targets\n   \
                 note: if you need to modify the result, store it in a variable first"
            ));
        }

        Ok(ParseNode::AccessChain {
            base: name,
            steps,
            line,
        })
    }

    fn parse_postfix_steps(&mut self) -> PResult<Vec<AccessStep>> {
        let mut steps = Vec::new();
        loop {
            if steps.len() >= 8 {
                match self.peek() {
                    Some(TokenType::ColonColon | TokenType::LBracket | TokenType::LParen) => {
                        return Err(self.err(
                            "access chain exceeds the maximum depth of 8 steps\n   \
                             note: break the expression into intermediate variables to simplify it",
                        ));
                    }
                    _ => break,
                }
            }
            match self.peek() {
                Some(TokenType::ColonColon) => {
                    self.advance();
                    let field = self.expect_identifier()?;
                    steps.push(AccessStep::Field(field));
                }
                Some(TokenType::LBracket) => {
                    self.advance();
                    let idx = self.parse_expression()?;
                    self.expect(&TokenType::RBracket)?;
                    steps.push(AccessStep::Index(Box::new(idx)));
                }
                Some(TokenType::LParen) => {
                    self.advance();
                    let args = self.parse_args()?;
                    self.expect(&TokenType::RParen)?;
                    steps.push(AccessStep::Call(args));
                }
                _ => break,
            }
        }
        Ok(steps)
    }

    fn try_parse_assignop(&mut self) -> Option<AssignOp> {
        let op = match self.peek()? {
            TokenType::Equals => AssignOp::Eq,
            TokenType::PlusEquals => AssignOp::PlusEq,
            TokenType::MinusEquals => AssignOp::MinusEq,
            TokenType::StarEquals => AssignOp::StarEq,
            TokenType::SlashEquals => AssignOp::SlashEq,
            TokenType::PercentEquals => AssignOp::PercentEq,
            TokenType::AmpersandEquals => AssignOp::AmpEq,
            TokenType::PipeEquals => AssignOp::PipeEq,
            TokenType::CaretEquals => AssignOp::CaretEq,
            _ => return None,
        };
        self.advance();
        Some(op)
    }

    fn parse_datatype(&mut self) -> PResult<ParseNode> {
        match self.peek().cloned() {
            Some(TokenType::TypeInt) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::TypeInt(line))
            }
            Some(TokenType::TypeFloat) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::TypeFloat(line))
            }
            Some(TokenType::TypeChar) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::TypeChar(line))
            }
            Some(TokenType::TypeBoolean) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::TypeBoolean(line))
            }
            Some(TokenType::TypeVoid) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::TypeVoid(line))
            }

            Some(TokenType::TypeArray) => {
                let line = self.cur_line();
                self.advance();
                self.expect(&TokenType::Less)?;
                let elem = self.parse_datatype()?;
                match self.peek() {
                    Some(TokenType::Comma) => { self.advance(); }
                    other => {
                        let found = Self::opt_token_name(other);
                        return Err(self.err(format!(
                            "`:array` requires a size: expected `,` followed by an integer size, but found {}\n   \
                             note: arrays have a fixed size declared at compile time: `:array<:int, 5>`\n   \
                             note: if you want a variable-length collection, use `:list<:int>` instead",
                            found
                        )));
                    }
                }
                let size = self.expect_int_lit()?;
                self.expect(&TokenType::Greater)?;
                Ok(ParseNode::TypeArray {
                    elem: Box::new(elem),
                    size,
                    line,
                })
            }

            Some(TokenType::TypeList) => {
                let line = self.cur_line();
                self.advance();
                self.expect(&TokenType::Less)?;
                let elem = self.parse_datatype()?;
                self.expect(&TokenType::Greater)?;
                Ok(ParseNode::TypeList {
                    elem: Box::new(elem),
                    line,
                })
            }

            Some(TokenType::TypeStruct) => {
                let line = self.cur_line();
                self.advance();
                self.expect(&TokenType::Less)?;
                let name = self.parse_struct_type_name()?;
                self.expect(&TokenType::Greater)?;
                Ok(ParseNode::TypeStruct { name, line })
            }

            other => Err(self.err(format!(
                "expected a type name here, but found {}\n   \
                 note: types must be prefixed with `:`, e.g. `:int`, `:float`, `:char`, `:boolean`, `:void`\n   \
                 note: generic types: `:array<:int, 5>`, `:list<:float>`, `:struct<Name>`",
                Self::opt_token_name(other.as_ref())
            ))),
        }
    }

    fn parse_expression(&mut self) -> PResult<ParseNode> {
        self.parse_logor()
    }

    fn parse_logor(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_logand()?;
        while matches!(self.peek(), Some(TokenType::Or)) {
            let line = self.cur_line();
            self.advance();
            let right = self.parse_logand()?;
            left = ParseNode::LogOr {
                left: Box::new(left),
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_logand(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_lognot()?;
        while matches!(self.peek(), Some(TokenType::And)) {
            let line = self.cur_line();
            self.advance();
            let right = self.parse_lognot()?;
            left = ParseNode::LogAnd {
                left: Box::new(left),
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_lognot(&mut self) -> PResult<ParseNode> {
        if matches!(self.peek(), Some(TokenType::Not)) {
            let line = self.cur_line();
            self.advance();
            let operand = self.parse_lognot()?;
            return Ok(ParseNode::LogNot {
                operand: Box::new(operand),
                line,
            });
        }
        self.parse_cmp()
    }

    fn parse_cmp(&mut self) -> PResult<ParseNode> {
        let left = self.parse_bitor()?;
        let op = match self.peek() {
            Some(TokenType::Greater) => CmpOp::Gt,
            Some(TokenType::Less) => CmpOp::Lt,
            Some(TokenType::GreaterEquals) => CmpOp::Ge,
            Some(TokenType::LessEquals) => CmpOp::Le,
            Some(TokenType::EqualsEquals) => CmpOp::EqEq,
            Some(TokenType::TildeEquals) => CmpOp::Ne,
            _ => return Ok(left),
        };
        let line = self.cur_line();
        self.advance();
        let right = self.parse_bitor()?;
        Ok(ParseNode::Cmp {
            left: Box::new(left),
            op,
            right: Box::new(right),
            line,
        })
    }

    fn parse_bitor(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_bitxor()?;
        while matches!(self.peek(), Some(TokenType::Pipe)) {
            let line = self.cur_line();
            self.advance();
            let right = self.parse_bitxor()?;
            left = ParseNode::BitOr {
                left: Box::new(left),
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_bitxor(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_bitand()?;
        while matches!(self.peek(), Some(TokenType::Caret)) {
            let line = self.cur_line();
            self.advance();
            let right = self.parse_bitand()?;
            left = ParseNode::BitXor {
                left: Box::new(left),
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_bitand(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_shift()?;
        while matches!(self.peek(), Some(TokenType::Ampersand)) {
            let line = self.cur_line();
            self.advance();
            let right = self.parse_shift()?;
            left = ParseNode::BitAnd {
                left: Box::new(left),
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_add()?;
        loop {
            let op = match (
                self.peek(),
                self.tokens.get(self.pos + 1).map(|t| &t.token_type),
            ) {
                (Some(TokenType::Less), Some(TokenType::Less)) => ShiftOp::Left,
                (Some(TokenType::Greater), Some(TokenType::Greater)) => ShiftOp::Right,
                _ => break,
            };
            let line = self.cur_line();
            self.advance();
            self.advance();
            let right = self.parse_add()?;
            left = ParseNode::BitShift {
                left: Box::new(left),
                op,
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(TokenType::Plus) => AddOp::Add,
                Some(TokenType::Minus) => AddOp::Sub,
                _ => break,
            };
            let line = self.cur_line();
            self.advance();
            let right = self.parse_mul()?;
            left = ParseNode::Add {
                left: Box::new(left),
                op,
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(TokenType::Star) => MulOp::Mul,
                Some(TokenType::Slash) => MulOp::Div,
                Some(TokenType::Percent) => MulOp::Mod,
                _ => break,
            };
            let line = self.cur_line();
            self.advance();
            let right = self.parse_unary()?;
            left = ParseNode::Mul {
                left: Box::new(left),
                op,
                right: Box::new(right),
                line,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> PResult<ParseNode> {
        let op = match self.peek() {
            Some(TokenType::Minus) => Some(UnOp::Neg),
            Some(TokenType::Tilde) => Some(UnOp::BitNot),
            Some(TokenType::Plus) => {
                self.advance();
                return self.parse_unary();
            }
            _ => None,
        };
        if let Some(op) = op {
            let line = self.cur_line();
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(ParseNode::Unary {
                op,
                operand: Box::new(operand),
                line,
            });
        }

        if Self::is_type_token_ref(self.peek()) {
            if let Some(node) = self.try_parse_cast()? {
                return Ok(node);
            }
        }

        self.parse_primary()
    }

    fn is_type_token_ref(tt: Option<&TokenType>) -> bool {
        matches!(
            tt,
            Some(TokenType::TypeInt)
                | Some(TokenType::TypeFloat)
                | Some(TokenType::TypeChar)
                | Some(TokenType::TypeBoolean)
                | Some(TokenType::TypeVoid)
                | Some(TokenType::TypeArray)
                | Some(TokenType::TypeList)
                | Some(TokenType::TypeStruct)
        )
    }

    fn try_parse_cast(&mut self) -> PResult<Option<ParseNode>> {
        let saved = self.pos;
        let dt = match self.parse_datatype() {
            Ok(dt) => dt,
            Err(_) => {
                self.pos = saved;
                return Ok(None);
            }
        };
        if !matches!(self.peek(), Some(TokenType::LParen)) {
            self.pos = saved;
            return Ok(None);
        }
        let line = self.cur_line();
        self.advance();
        let expr = self.parse_expression()?;
        self.expect(&TokenType::RParen)?;
        Ok(Some(ParseNode::Cast {
            target_type: Box::new(dt),
            expr: Box::new(expr),
            line,
        }))
    }

    fn parse_primary(&mut self) -> PResult<ParseNode> {
        match self.peek().cloned() {
            Some(TokenType::LParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(&TokenType::RParen)?;
                Ok(expr)
            }

            Some(TokenType::LBracket) => {
                let line = self.cur_line();
                self.advance();
                let elems = self.parse_args()?;
                self.expect(&TokenType::RBracket)?;
                Ok(ParseNode::ArrayLit(elems, line))
            }

            Some(TokenType::LBrace) => {
                let line = self.cur_line();
                self.advance();
                let fields = self.parse_struct_lit_fields()?;
                Ok(ParseNode::StructLit(fields, line))
            }

            Some(TokenType::Identifier(name)) => {
                let line = self.cur_line();
                self.advance();
                let steps = self.parse_postfix_steps()?;
                Ok(ParseNode::AccessChain {
                    base: name,
                    steps,
                    line,
                })
            }

            Some(TokenType::SIntLit(n)) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::IntLit(n, line))
            }
            Some(TokenType::FloatLit(f)) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::FloatLit(f, line))
            }
            Some(TokenType::CharLit(c)) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::CharLit(c, line))
            }
            Some(TokenType::StringLit(s)) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::StringLit(s, line))
            }
            Some(TokenType::BoolLit(b)) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::BoolLit(b, line))
            }
            Some(TokenType::Null) => {
                let line = self.cur_line();
                self.advance();
                Ok(ParseNode::Null(line))
            }

            other => {
                let keyword_note = if let Some(TokenType::Identifier(ref name)) = other {
                    Self::keyword_hint(name)
                        .map(|h| format!("\n   note: {h}"))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                Err(self.err(format!(
                    "expected an expression here, but found {}\n   \
                     note: expressions can be literals (`42`, `3.14`, `'a'`, `true`), \
                     identifiers, function calls, or sub-expressions in `( )`{keyword_note}",
                    Self::opt_token_name(other.as_ref())
                )))
            }
        }
    }

    fn parse_args(&mut self) -> PResult<Vec<ParseNode>> {
        let mut args = Vec::new();
        if matches!(
            self.peek(),
            Some(TokenType::RParen) | Some(TokenType::RBracket) | None
        ) {
            return Ok(args);
        }
        args.push(self.parse_expression()?);
        while matches!(self.peek(), Some(TokenType::Comma)) {
            self.advance();

            if matches!(
                self.peek(),
                Some(TokenType::RParen) | Some(TokenType::RBracket) | None
            ) {
                return Err(self.err(
                    "trailing comma is not allowed in an argument list\n   \
                     note: remove the `,` after the last argument\n   \
                     note: valid call: `func(a, b, c)` — not `func(a, b, c,)`",
                ));
            }
            args.push(self.parse_expression()?);
        }
        Ok(args)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<ParseNode, ParseError> {
    let mut parser = Parser::new(tokens, "<source>");
    parser.parse_program()
}

pub fn parse_with_source(tokens: Vec<Token>, source_file: &str) -> Result<ParseNode, ParseError> {
    let mut parser = Parser::new(tokens, source_file);
    parser.parse_program()
}

pub fn pretty_print(node: &ParseNode) {
    pretty_print_root(node);
}

fn node_label(node: &ParseNode) -> String {
    match node {
        ParseNode::Program(_) => "Program".into(),
        ParseNode::Module { name, .. } => format!("Module  \x1b[33m{}\x1b[0m", name),
        ParseNode::FuncDef {
            name, return_type, ..
        } => format!(
            "FuncDef  \x1b[33m{}\x1b[0m  → {}",
            name,
            type_str(return_type)
        ),
        ParseNode::Param { data_type, name } => {
            format!("Param  \x1b[36m{}\x1b[0m : {}", name, type_str(data_type))
        }
        ParseNode::StructDef { name, .. } => format!("StructDef  \x1b[33m{}\x1b[0m", name),
        ParseNode::StructDecl {
            struct_name,
            var_name,
            ..
        } => format!("StructDecl  \x1b[36m{}\x1b[0m : {}", var_name, struct_name),
        ParseNode::Field { data_type, name } => {
            format!("Field  \x1b[36m{}\x1b[0m : {}", name, type_str(data_type))
        }
        ParseNode::Decl {
            data_type,
            name,
            init,
            ..
        } => format!(
            "Decl  \x1b[36m{}\x1b[0m : {}{}",
            name,
            type_str(data_type),
            if init.is_some() { "  =" } else { "" }
        ),
        ParseNode::Assign { op, .. } => format!("Assign  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::If { .. } => "If".into(),
        ParseNode::For {
            var_type, var_name, ..
        } => format!("For  \x1b[36m{}\x1b[0m : {}", var_name, type_str(var_type)),
        ParseNode::While { .. } => "While".into(),
        ParseNode::Return { .. } => "Return".into(),
        ParseNode::Exit { .. } => "Exit".into(),
        ParseNode::Break { .. } => "\x1b[35mBreak\x1b[0m".into(),
        ParseNode::Continue { .. } => "\x1b[35mContinue\x1b[0m".into(),
        ParseNode::ExprStmt(_, _) => "ExprStmt".into(),
        ParseNode::AccessChain { base, steps, .. } => {
            let chain: String = steps
                .iter()
                .map(|s| match s {
                    AccessStep::Field(f) => format!("::{}", f),
                    AccessStep::Index(_) => "[…]".into(),
                    AccessStep::Call(a) => format!("({})", a.len()),
                })
                .collect();
            format!("AccessChain  \x1b[36m{}{}\x1b[0m", base, chain)
        }
        ParseNode::LogOr { .. } => "LogOr  \x1b[35m!or\x1b[0m".into(),
        ParseNode::LogAnd { .. } => "LogAnd  \x1b[35m!and\x1b[0m".into(),
        ParseNode::LogNot { .. } => "LogNot  \x1b[35m!not\x1b[0m".into(),
        ParseNode::Cmp { op, .. } => format!("Cmp  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::BitOr { .. } => "BitOr  \x1b[35m|\x1b[0m".into(),
        ParseNode::BitXor { .. } => "BitXor  \x1b[35m^\x1b[0m".into(),
        ParseNode::BitAnd { .. } => "BitAnd  \x1b[35m&\x1b[0m".into(),
        ParseNode::BitShift { op, .. } => format!("BitShift  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::Add { op, .. } => format!("Add  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::Mul { op, .. } => format!("Mul  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::Unary { op, .. } => format!("Unary  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::Cast { target_type, .. } => format!("Cast  → {}", type_str(target_type)),
        ParseNode::ArrayLit(elems, _) => format!("ArrayLit  [{}]", elems.len()),
        ParseNode::StructLit(fields, _) => format!(
            "StructLit  {{{}}}",
            fields
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ParseNode::Identifier(s, _) => format!("Identifier  \x1b[32m{}\x1b[0m", s),
        ParseNode::IntLit(n, _) => format!("IntLit  \x1b[32m{}\x1b[0m", n),
        ParseNode::FloatLit(f, _) => format!("FloatLit  \x1b[32m{}\x1b[0m", f),
        ParseNode::CharLit(c, _) => format!("CharLit  \x1b[32m{:?}\x1b[0m", c),
        ParseNode::StringLit(s, _) => format!("StringLit  \x1b[32m{:?}\x1b[0m", s),
        ParseNode::BoolLit(b, _) => format!("BoolLit  \x1b[32m{}\x1b[0m", b),
        ParseNode::Null(_) => "\x1b[32mNull\x1b[0m".into(),
        ParseNode::TypeInt(_) => "TypeInt".into(),
        ParseNode::TypeFloat(_) => "TypeFloat".into(),
        ParseNode::TypeChar(_) => "TypeChar".into(),
        ParseNode::TypeBoolean(_) => "TypeBoolean".into(),
        ParseNode::TypeVoid(_) => "TypeVoid".into(),
        ParseNode::TypeArray { elem, size, .. } => {
            format!("TypeArray<{},{}>", type_str(elem), size)
        }
        ParseNode::TypeList { elem, .. } => format!("TypeList<{}>", type_str(elem)),
        ParseNode::TypeStruct { name, .. } => format!("TypeStruct<{}>", name),
    }
}

fn type_str(node: &ParseNode) -> String {
    match node {
        ParseNode::TypeInt(_) => "int".into(),
        ParseNode::TypeFloat(_) => "float".into(),
        ParseNode::TypeChar(_) => "char".into(),
        ParseNode::TypeBoolean(_) => "bool".into(),
        ParseNode::TypeVoid(_) => "void".into(),
        ParseNode::TypeArray { elem, size, .. } => format!("array<{},{}>", type_str(elem), size),
        ParseNode::TypeList { elem, .. } => format!("list<{}>", type_str(elem)),
        ParseNode::TypeStruct { name, .. } => format!("struct<{}>", name),
        other => format!("{:?}", other),
    }
}

pub fn pretty_print_root(node: &ParseNode) {
    print_node(node, "", true);
}

fn print_node(node: &ParseNode, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    println!("{}{}{}", prefix, connector, node_label(node));

    print_node_children(node, &child_prefix);
}

fn print_node_list(nodes: &[ParseNode], prefix: &str) {
    let n = nodes.len();
    for (i, node) in nodes.iter().enumerate() {
        print_node(node, prefix, i == n - 1);
    }
}

fn print_section_header(title: &str, prefix: &str, is_last: bool) -> String {
    let connector = if is_last { "└── " } else { "├── " };
    println!("{}{}\x1b[2m[{}]\x1b[0m", prefix, connector, title);
    format!("{}{}", prefix, if is_last { "    " } else { "│   " })
}

fn print_node_children(node: &ParseNode, prefix: &str) {
    match node {
        ParseNode::Program(items) => {
            print_node_list(items, prefix);
        }
        ParseNode::Module { items, .. } => {
            print_node_list(items, prefix);
        }
        ParseNode::FuncDef { params, body, .. } => {
            if !params.is_empty() {
                let pp = print_section_header("params", prefix, false);
                print_node_list(params, &pp);
            }
            let bp = print_section_header("body", prefix, true);
            print_node_list(body, &bp);
        }
        ParseNode::StructDef { fields, .. } => {
            print_node_list(fields, prefix);
        }
        ParseNode::StructDecl { init, .. } => {
            if let Some(i) = init {
                let ip = print_section_header("init", prefix, true);
                print_node(i, &ip, true);
            }
        }
        ParseNode::Decl { init, .. } => {
            if let Some(i) = init {
                let ip = print_section_header("init", prefix, true);
                print_node(i, &ip, true);
            }
        }
        ParseNode::Assign { lvalue, expr, .. } => {
            let lp = print_section_header("lvalue", prefix, false);
            print_node(lvalue, &lp, true);
            let ep = print_section_header("expr", prefix, true);
            print_node(expr, &ep, true);
        }
        ParseNode::If {
            condition,
            then_block,
            else_block,
            ..
        } => {
            let cp = print_section_header(
                "condition",
                prefix,
                else_block.is_none() && then_block.is_empty(),
            );
            print_node(condition, &cp, true);
            let has_else = else_block.is_some();
            let tp = print_section_header("then", prefix, !has_else);
            print_node_list(then_block, &tp);
            if let Some(eb) = else_block {
                let ep = print_section_header("else", prefix, true);
                print_node_list(eb, &ep);
            }
        }
        ParseNode::For {
            start,
            stop,
            step,
            body,
            ..
        } => {
            let sp = print_section_header("start", prefix, false);
            print_node(start, &sp, true);
            let sp2 = print_section_header("stop", prefix, false);
            print_node(stop, &sp2, true);
            let sp3 = print_section_header("step", prefix, false);
            print_node(step, &sp3, true);
            let bp = print_section_header("body", prefix, true);
            print_node_list(body, &bp);
        }
        ParseNode::While {
            condition, body, ..
        } => {
            let cp = print_section_header("condition", prefix, false);
            print_node(condition, &cp, true);
            let bp = print_section_header("body", prefix, true);
            print_node_list(body, &bp);
        }
        ParseNode::Return { expr: e, .. } | ParseNode::Exit { expr: e, .. } => {
            print_node(e, prefix, true);
        }
        ParseNode::ExprStmt(e, _) => {
            print_node(e, prefix, true);
        }
        ParseNode::AccessChain { steps, .. } => {
            let n = steps.len();
            for (i, step) in steps.iter().enumerate() {
                let is_last = i == n - 1;
                match step {
                    AccessStep::Index(idx) => {
                        let ip = print_section_header(&format!("[{}]", i), prefix, is_last);
                        print_node(idx, &ip, true);
                    }
                    AccessStep::Call(args) => {
                        let cp = print_section_header(&format!("call({})", i), prefix, is_last);
                        print_node_list(args, &cp);
                    }
                    AccessStep::Field(_) => {}
                }
            }
        }
        ParseNode::LogOr { left, right, .. }
        | ParseNode::LogAnd { left, right, .. }
        | ParseNode::BitOr { left, right, .. }
        | ParseNode::BitXor { left, right, .. }
        | ParseNode::BitAnd { left, right, .. } => {
            print_node(left, prefix, false);
            print_node(right, prefix, true);
        }
        ParseNode::BitShift { left, right, .. } => {
            print_node(left, prefix, false);
            print_node(right, prefix, true);
        }
        ParseNode::LogNot { operand, .. } => {
            print_node(operand, prefix, true);
        }
        ParseNode::Cmp { left, right, .. }
        | ParseNode::Add { left, right, .. }
        | ParseNode::Mul { left, right, .. } => {
            print_node(left, prefix, false);
            print_node(right, prefix, true);
        }
        ParseNode::Unary { operand, .. } => {
            print_node(operand, prefix, true);
        }
        ParseNode::Cast { expr, .. } => {
            print_node(expr, prefix, true);
        }
        ParseNode::ArrayLit(elems, _) => {
            print_node_list(elems, prefix);
        }
        ParseNode::StructLit(fields, _) => {
            let n = fields.len();
            for (i, (name, val)) in fields.iter().enumerate() {
                let is_last = i == n - 1;
                let fp = print_section_header(name, prefix, is_last);
                print_node(val, &fp, true);
            }
        }

        _ => {}
    }
}
