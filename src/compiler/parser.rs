#![allow(unused)]
#![allow(dead_code)]

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
    },

    Field {
        data_type: Box<ParseNode>,
        name: String,
    },

    Decl {
        data_type: Box<ParseNode>,
        name: String,
        init: Option<Box<ParseNode>>,
    },

    Assign {
        lvalue: Box<ParseNode>,
        op: AssignOp,
        expr: Box<ParseNode>,
    },

    If {
        condition: Box<ParseNode>,
        then_block: Vec<ParseNode>,
        else_block: Option<Vec<ParseNode>>,
    },

    For {
        var_type: Box<ParseNode>,
        var_name: String,
        start: Box<ParseNode>,
        stop: Box<ParseNode>,
        step: Box<ParseNode>,
        body: Vec<ParseNode>,
    },

    While {
        condition: Box<ParseNode>,
        body: Vec<ParseNode>,
    },

    Return(Box<ParseNode>),

    Exit(Box<ParseNode>),

    Break,
    Continue,

    ExprStmt(Box<ParseNode>),

    LValue {
        name: String,

        member: Option<String>,
    },

    LogOr {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
    },

    LogAnd {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
    },

    Cmp {
        left: Box<ParseNode>,
        op: CmpOp,
        right: Box<ParseNode>,
    },

    Add {
        left: Box<ParseNode>,
        op: AddOp,
        right: Box<ParseNode>,
    },

    Mul {
        left: Box<ParseNode>,
        op: MulOp,
        right: Box<ParseNode>,
    },

    Unary {
        op: UnOp,
        operand: Box<ParseNode>,
    },

    Cast {
        target_type: Box<ParseNode>,
        expr: Box<ParseNode>,
    },

    QualifiedCall {
        module: String,
        member: String,
        args: Option<Vec<ParseNode>>,
    },

    Call {
        name: String,
        args: Vec<ParseNode>,
    },

    ArrayLit(Vec<ParseNode>),

    Identifier(String),
    IntLit(i64),
    FloatLit(f64),
    CharLit(char),
    StringLit(String),
    BoolLit(bool),
    Null,

    TypeInt,
    TypeFloat,
    TypeChar,
    TypeBoolean,

    TypeArray {
        elem: String,
        size: i64,
    },

    TypeList {
        elem: String,
    },

    TypeStruct {
        name: String,
    },
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
pub enum UnOp {
    Neg,
    BitNot,
    Deref,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    fn new(msg: impl Into<String>) -> Self {
        ParseError {
            message: msg.into(),
        }
    }
}

type PResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&TokenType> {
        self.tokens.get(self.pos).map(|t| &t.token_type)
    }

    fn peek2(&self) -> Option<&TokenType> {
        self.tokens.get(self.pos + 1).map(|t| &t.token_type)
    }

    fn advance(&mut self) -> Option<&TokenType> {
        let t = self.tokens.get(self.pos).map(|t| &t.token_type);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &TokenType) -> PResult<()> {
        match self.peek() {
            Some(tt) if tt == expected => {
                self.advance();
                Ok(())
            }
            other => Err(ParseError::new(format!(
                "Expected {:?}, got {:?}",
                expected, other
            ))),
        }
    }

    fn expect_identifier(&mut self) -> PResult<String> {
        match self.peek().cloned() {
            Some(TokenType::Identifier(s)) => {
                self.advance();
                Ok(s)
            }
            other => Err(ParseError::new(format!(
                "Expected identifier, got {:?}",
                other
            ))),
        }
    }

    fn expect_int_lit(&mut self) -> PResult<i64> {
        match self.peek().cloned() {
            Some(TokenType::SIntLit(n)) => {
                self.advance();
                Ok(n)
            }
            other => Err(ParseError::new(format!(
                "Expected integer literal, got {:?}",
                other
            ))),
        }
    }

    fn at_endl(&self) -> bool {
        matches!(self.peek(), Some(TokenType::EndL))
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

                        if self.at_endl() {
                            self.advance();
                        }
                        Ok(ParseNode::Module { name, items })
                    }
                    other => Err(ParseError::new(format!(
                        "Expected ModuleEnd, got {:?}",
                        other
                    ))),
                }
            }

            Some(TokenType::Func) => self.parse_funcdef(),

            Some(TokenType::TypeStruct) => self.parse_struct_item(),

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
                | TokenType::TypeArray
                | TokenType::TypeList
                | TokenType::TypeStruct
        )
    }

    fn parse_funcdef(&mut self) -> PResult<ParseNode> {
        self.expect(&TokenType::Func)?;
        let name = self.expect_identifier()?;
        self.expect(&TokenType::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenType::RParen)?;
        self.expect(&TokenType::Arrow)?;
        let return_type = self.parse_datatype()?;
        let body = self.parse_block()?;
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

    fn parse_struct_item(&mut self) -> PResult<ParseNode> {
        self.expect(&TokenType::TypeStruct)?;
        self.expect(&TokenType::Less)?;
        let type_name = self.expect_identifier()?;
        self.expect(&TokenType::Greater)?;

        match self.peek().cloned() {
            Some(TokenType::LBrace) => {
                self.advance();
                let fields = self.parse_fields()?;
                self.expect(&TokenType::RBrace)?;
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
                self.expect(&TokenType::EndL)?;
                Ok(ParseNode::StructDecl {
                    struct_name: type_name,
                    var_name,
                    init,
                })
            }
            other => Err(ParseError::new(format!(
                "Expected '{{' or identifier after struct<Name>, got {:?}",
                other
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
                        let f = self.parse_struct_field()?;
                        fields.push(f);
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
        let type_name = self.expect_identifier()?;
        self.expect(&TokenType::Greater)?;
        let field_name = self.expect_identifier()?;
        self.expect(&TokenType::EndL)?;
        Ok(ParseNode::Field {
            data_type: Box::new(ParseNode::TypeStruct { name: type_name }),
            name: field_name,
        })
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
            ParseNode::If { .. } | ParseNode::For { .. } | ParseNode::While { .. }
        ) {
            if self.at_endl() {
                self.advance();
            }
        }
        Ok(stmt)
    }

    fn parse_stmt(&mut self) -> PResult<ParseNode> {
        match self.peek().cloned() {
            Some(TokenType::If) => {
                self.advance();
                self.expect(&TokenType::LParen)?;
                let condition = self.parse_expression()?;
                self.expect(&TokenType::RParen)?;
                let then_block = self.parse_block()?;
                let else_block = if matches!(self.peek(), Some(TokenType::Else)) {
                    self.advance();
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(ParseNode::If {
                    condition: Box::new(condition),
                    then_block,
                    else_block,
                })
            }

            Some(TokenType::For) => {
                self.advance();
                self.expect(&TokenType::LParen)?;
                let var_type = self.parse_datatype()?;
                let var_name = self.expect_identifier()?;
                self.expect(&TokenType::Comma)?;
                let start = self.parse_expression()?;
                self.expect(&TokenType::Comma)?;
                let stop = self.parse_expression()?;
                self.expect(&TokenType::Comma)?;
                let step = self.parse_expression()?;
                self.expect(&TokenType::RParen)?;
                let body = self.parse_block()?;
                Ok(ParseNode::For {
                    var_type: Box::new(var_type),
                    var_name,
                    start: Box::new(start),
                    stop: Box::new(stop),
                    step: Box::new(step),
                    body,
                })
            }

            Some(TokenType::While) => {
                self.advance();
                self.expect(&TokenType::LParen)?;
                let condition = self.parse_expression()?;
                self.expect(&TokenType::RParen)?;
                let body = self.parse_block()?;
                Ok(ParseNode::While {
                    condition: Box::new(condition),
                    body,
                })
            }

            Some(TokenType::Return) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(ParseNode::Return(Box::new(expr)))
            }

            Some(TokenType::Exit) => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(ParseNode::Exit(Box::new(expr)))
            }

            Some(TokenType::Break) => {
                self.advance();
                Ok(ParseNode::Break)
            }

            Some(TokenType::Continue) => {
                self.advance();
                Ok(ParseNode::Continue)
            }

            Some(TokenType::TypeStruct) => self.parse_struct_item(),

            Some(ref t) if Self::is_type_token(t) => self.parse_decl(),

            Some(TokenType::Identifier(_)) => self.parse_assign_or_expr_stmt(),

            _ => {
                let expr = self.parse_expression()?;
                Ok(ParseNode::ExprStmt(Box::new(expr)))
            }
        }
    }

    fn parse_decl(&mut self) -> PResult<ParseNode> {
        let data_type = self.parse_datatype()?;
        let name = self.expect_identifier()?;
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
        })
    }

    fn parse_assign_or_expr_stmt(&mut self) -> PResult<ParseNode> {
        let saved = self.pos;

        if let Ok(lval) = self.try_parse_lvalue() {
            if let Some(op) = self.try_parse_assignop() {
                let expr = self.parse_expression()?;
                return Ok(ParseNode::Assign {
                    lvalue: Box::new(lval),
                    op,
                    expr: Box::new(expr),
                });
            }
        }

        self.pos = saved;
        let expr = self.parse_expression()?;
        Ok(ParseNode::ExprStmt(Box::new(expr)))
    }

    fn try_parse_lvalue(&mut self) -> PResult<ParseNode> {
        let name = self.expect_identifier()?;

        if matches!(self.peek(), Some(TokenType::ColonColon)) {
            self.advance();
            let member = self.expect_identifier()?;
            Ok(ParseNode::LValue {
                name,
                member: Some(member),
            })
        } else {
            Ok(ParseNode::LValue { name, member: None })
        }
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
                self.advance();
                Ok(ParseNode::TypeInt)
            }
            Some(TokenType::TypeFloat) => {
                self.advance();
                Ok(ParseNode::TypeFloat)
            }
            Some(TokenType::TypeChar) => {
                self.advance();
                Ok(ParseNode::TypeChar)
            }
            Some(TokenType::TypeBoolean) => {
                self.advance();
                Ok(ParseNode::TypeBoolean)
            }

            Some(TokenType::TypeArray) => {
                self.advance();
                self.expect(&TokenType::Less)?;
                let elem = self.expect_identifier()?;
                self.expect(&TokenType::Comma)?;
                let size = self.expect_int_lit()?;
                self.expect(&TokenType::Greater)?;
                Ok(ParseNode::TypeArray { elem, size })
            }

            Some(TokenType::TypeList) => {
                self.advance();
                self.expect(&TokenType::Less)?;
                let elem = self.expect_identifier()?;
                self.expect(&TokenType::Greater)?;
                Ok(ParseNode::TypeList { elem })
            }

            Some(TokenType::TypeStruct) => {
                self.advance();
                self.expect(&TokenType::Less)?;
                let name = self.expect_identifier()?;
                self.expect(&TokenType::Greater)?;
                Ok(ParseNode::TypeStruct { name })
            }

            other => Err(ParseError::new(format!(
                "Expected datatype, got {:?}",
                other
            ))),
        }
    }

    fn parse_expression(&mut self) -> PResult<ParseNode> {
        self.parse_logor()
    }

    fn parse_logor(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_logand()?;
        while matches!(self.peek(), Some(TokenType::OrOr)) {
            self.advance();
            let right = self.parse_logand()?;
            left = ParseNode::LogOr {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_logand(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_cmp()?;
        while matches!(self.peek(), Some(TokenType::AndAnd)) {
            self.advance();
            let right = self.parse_cmp()?;
            left = ParseNode::LogAnd {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> PResult<ParseNode> {
        let left = self.parse_add()?;
        let op = match self.peek() {
            Some(TokenType::Greater) => CmpOp::Gt,
            Some(TokenType::Less) => CmpOp::Lt,
            Some(TokenType::GreaterEquals) => CmpOp::Ge,
            Some(TokenType::LessEquals) => CmpOp::Le,
            Some(TokenType::EqualsEquals) => CmpOp::EqEq,
            Some(TokenType::TildeEquals) => CmpOp::Ne,
            _ => return Ok(left),
        };
        self.advance();
        let right = self.parse_add()?;
        Ok(ParseNode::Cmp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn parse_add(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(TokenType::Plus) => AddOp::Add,
                Some(TokenType::Minus) => AddOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul()?;
            left = ParseNode::Add {
                left: Box::new(left),
                op,
                right: Box::new(right),
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
            self.advance();
            let right = self.parse_unary()?;
            left = ParseNode::Mul {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> PResult<ParseNode> {
        let op = match self.peek() {
            Some(TokenType::Minus) => Some(UnOp::Neg),
            Some(TokenType::Tilde) => Some(UnOp::BitNot),
            Some(TokenType::Ampersand) => Some(UnOp::Deref),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(ParseNode::Unary {
                op,
                operand: Box::new(operand),
            });
        }

        if Self::is_type_token_ref(self.peek()) {
            if let Some(node) = self.try_parse_cast()? {
                return Ok(node);
            }
        }

        self.parse_postfix()
    }

    fn is_type_token_ref(tt: Option<&TokenType>) -> bool {
        matches!(
            tt,
            Some(TokenType::TypeInt)
                | Some(TokenType::TypeFloat)
                | Some(TokenType::TypeChar)
                | Some(TokenType::TypeBoolean)
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
        self.advance();
        let expr = self.parse_expression()?;
        self.expect(&TokenType::RParen)?;
        Ok(Some(ParseNode::Cast {
            target_type: Box::new(dt),
            expr: Box::new(expr),
        }))
    }

    fn parse_postfix(&mut self) -> PResult<ParseNode> {
        let primary = self.parse_primary()?;

        Ok(primary)
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
                self.advance();
                let elems = self.parse_args()?;
                self.expect(&TokenType::RBracket)?;
                Ok(ParseNode::ArrayLit(elems))
            }

            Some(TokenType::Identifier(name)) => {
                self.advance();

                match self.peek().cloned() {
                    Some(TokenType::ColonColon) => {
                        self.advance();
                        let member = self.expect_identifier()?;
                        if matches!(self.peek(), Some(TokenType::LParen)) {
                            self.advance();
                            let args = self.parse_args()?;
                            self.expect(&TokenType::RParen)?;
                            Ok(ParseNode::QualifiedCall {
                                module: name,
                                member,
                                args: Some(args),
                            })
                        } else {
                            Ok(ParseNode::QualifiedCall {
                                module: name,
                                member,
                                args: None,
                            })
                        }
                    }

                    Some(TokenType::LParen) => {
                        self.advance();
                        let args = self.parse_args()?;
                        self.expect(&TokenType::RParen)?;
                        Ok(ParseNode::Call { name, args })
                    }

                    _ => Ok(ParseNode::Identifier(name)),
                }
            }

            Some(TokenType::SIntLit(n)) => {
                self.advance();
                Ok(ParseNode::IntLit(n))
            }
            Some(TokenType::FloatLit(f)) => {
                self.advance();
                Ok(ParseNode::FloatLit(f))
            }
            Some(TokenType::CharLit(c)) => {
                self.advance();
                Ok(ParseNode::CharLit(c))
            }
            Some(TokenType::StringLit(s)) => {
                self.advance();
                Ok(ParseNode::StringLit(s))
            }
            Some(TokenType::BoolLit(b)) => {
                self.advance();
                Ok(ParseNode::BoolLit(b))
            }
            Some(TokenType::Null) => {
                self.advance();
                Ok(ParseNode::Null)
            }

            other => Err(ParseError::new(format!(
                "Unexpected token in expression: {:?}",
                other
            ))),
        }
    }

    fn parse_args(&mut self) -> PResult<Vec<ParseNode>> {
        let mut args = Vec::new();

        if matches!(
            self.peek(),
            Some(TokenType::RParen) | Some(TokenType::RBracket)
        ) {
            return Ok(args);
        }
        args.push(self.parse_expression()?);
        while matches!(self.peek(), Some(TokenType::Comma)) {
            self.advance();
            args.push(self.parse_expression()?);
        }
        Ok(args)
    }
}

pub fn pretty_print(node: &ParseNode, indent: usize) {
    let pad = "  ".repeat(indent);
    match node {
        ParseNode::Program(items) => {
            println!("{}Program", pad);
            for item in items {
                pretty_print(item, indent + 1);
            }
        }
        ParseNode::Module { name, items } => {
            println!("{}Module({})", pad, name);
            for item in items {
                pretty_print(item, indent + 1);
            }
        }
        ParseNode::FuncDef {
            name,
            params,
            return_type,
            body,
        } => {
            println!("{}FuncDef: {}", pad, name);
            println!("{}  ReturnType:", pad);
            pretty_print(return_type, indent + 2);
            if !params.is_empty() {
                println!("{}  Params:", pad);
                for p in params {
                    pretty_print(p, indent + 2);
                }
            }
            println!("{}  Body:", pad);
            for s in body {
                pretty_print(s, indent + 2);
            }
        }
        ParseNode::Param { data_type, name } => {
            println!("{}Param: {}", pad, name);
            pretty_print(data_type, indent + 1);
        }
        ParseNode::StructDef { name, fields } => {
            println!("{}StructDef: {}", pad, name);
            for f in fields {
                pretty_print(f, indent + 1);
            }
        }
        ParseNode::StructDecl {
            struct_name,
            var_name,
            init,
        } => {
            println!("{}StructDecl: {} : {}", pad, var_name, struct_name);
            if let Some(i) = init {
                pretty_print(i, indent + 1);
            }
        }
        ParseNode::Field { data_type, name } => {
            println!("{}Field: {}", pad, name);
            pretty_print(data_type, indent + 1);
        }
        ParseNode::Decl {
            data_type,
            name,
            init,
        } => {
            println!("{}Decl: {}", pad, name);
            pretty_print(data_type, indent + 1);
            if let Some(i) = init {
                println!("{}  = ", pad);
                pretty_print(i, indent + 2);
            }
        }
        ParseNode::Assign { lvalue, op, expr } => {
            println!("{}Assign ({:?})", pad, op);
            pretty_print(lvalue, indent + 1);
            pretty_print(expr, indent + 1);
        }
        ParseNode::If {
            condition,
            then_block,
            else_block,
        } => {
            println!("{}If", pad);
            println!("{}  Condition:", pad);
            pretty_print(condition, indent + 2);
            println!("{}  Then:", pad);
            for s in then_block {
                pretty_print(s, indent + 2);
            }
            if let Some(eb) = else_block {
                println!("{}  Else:", pad);
                for s in eb {
                    pretty_print(s, indent + 2);
                }
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
            println!("{}For: {}", pad, var_name);
            pretty_print(var_type, indent + 1);
            println!("{}  Start:", pad);
            pretty_print(start, indent + 2);
            println!("{}  Stop:", pad);
            pretty_print(stop, indent + 2);
            println!("{}  Step:", pad);
            pretty_print(step, indent + 2);
            println!("{}  Body:", pad);
            for s in body {
                pretty_print(s, indent + 2);
            }
        }
        ParseNode::While { condition, body } => {
            println!("{}While", pad);
            println!("{}  Condition:", pad);
            pretty_print(condition, indent + 2);
            println!("{}  Body:", pad);
            for s in body {
                pretty_print(s, indent + 2);
            }
        }
        ParseNode::Return(e) => {
            println!("{}Return", pad);
            pretty_print(e, indent + 1);
        }
        ParseNode::Exit(e) => {
            println!("{}Exit", pad);
            pretty_print(e, indent + 1);
        }
        ParseNode::Break => println!("{}Break", pad),
        ParseNode::Continue => println!("{}Continue", pad),
        ParseNode::ExprStmt(e) => {
            println!("{}ExprStmt", pad);
            pretty_print(e, indent + 1);
        }
        ParseNode::LValue { name, member } => match member {
            Some(m) => println!("{}LValue: {}::{}", pad, name, m),
            None => println!("{}LValue: {}", pad, name),
        },
        ParseNode::LogOr { left, right } => {
            println!("{}LogOr", pad);
            pretty_print(left, indent + 1);
            pretty_print(right, indent + 1);
        }
        ParseNode::LogAnd { left, right } => {
            println!("{}LogAnd", pad);
            pretty_print(left, indent + 1);
            pretty_print(right, indent + 1);
        }
        ParseNode::Cmp { left, op, right } => {
            println!("{}Cmp ({:?})", pad, op);
            pretty_print(left, indent + 1);
            pretty_print(right, indent + 1);
        }
        ParseNode::Add { left, op, right } => {
            println!("{}Add ({:?})", pad, op);
            pretty_print(left, indent + 1);
            pretty_print(right, indent + 1);
        }
        ParseNode::Mul { left, op, right } => {
            println!("{}Mul ({:?})", pad, op);
            pretty_print(left, indent + 1);
            pretty_print(right, indent + 1);
        }
        ParseNode::Unary { op, operand } => {
            println!("{}Unary ({:?})", pad, op);
            pretty_print(operand, indent + 1);
        }
        ParseNode::Cast { target_type, expr } => {
            println!("{}Cast", pad);
            println!("{}  To:", pad);
            pretty_print(target_type, indent + 2);
            println!("{}  Expr:", pad);
            pretty_print(expr, indent + 2);
        }
        ParseNode::QualifiedCall {
            module,
            member,
            args,
        } => {
            println!("{}QualifiedCall: {}::{}", pad, module, member);
            if let Some(args) = args {
                for a in args {
                    pretty_print(a, indent + 1);
                }
            }
        }
        ParseNode::Call { name, args } => {
            println!("{}Call: {}", pad, name);
            for a in args {
                pretty_print(a, indent + 1);
            }
        }
        ParseNode::ArrayLit(elems) => {
            println!("{}ArrayLit", pad);
            for e in elems {
                pretty_print(e, indent + 1);
            }
        }
        ParseNode::Identifier(s) => println!("{}Identifier: {}", pad, s),
        ParseNode::IntLit(n) => println!("{}IntLit: {}", pad, n),
        ParseNode::FloatLit(f) => println!("{}FloatLit: {}", pad, f),
        ParseNode::CharLit(c) => println!("{}CharLit: {:?}", pad, c),
        ParseNode::StringLit(s) => println!("{}StringLit: {:?}", pad, s),
        ParseNode::BoolLit(b) => println!("{}BoolLit: {}", pad, b),
        ParseNode::Null => println!("{}Null", pad),
        ParseNode::TypeInt => println!("{}TypeInt", pad),
        ParseNode::TypeFloat => println!("{}TypeFloat", pad),
        ParseNode::TypeChar => println!("{}TypeChar", pad),
        ParseNode::TypeBoolean => println!("{}TypeBoolean", pad),
        ParseNode::TypeArray { elem, size } => println!("{}TypeArray<{}, {}>", pad, elem, size),
        ParseNode::TypeList { elem } => println!("{}TypeList<{}>", pad, elem),
        ParseNode::TypeStruct { name } => println!("{}TypeStruct<{}>", pad, name),
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<ParseNode, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}
