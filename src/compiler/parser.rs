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
    LogNot {
        operand: Box<ParseNode>,
    },

    Cmp {
        left: Box<ParseNode>,
        op: CmpOp,
        right: Box<ParseNode>,
    },

    BitOr {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
    },
    BitXor {
        left: Box<ParseNode>,
        right: Box<ParseNode>,
    },
    BitAnd {
        left: Box<ParseNode>,
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

    StructLit(Vec<(String, ParseNode)>),

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
    Index {
        target: Box<ParseNode>,
        index: Box<ParseNode>,
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
                    Some(TokenType::ModuleEnd(_end_name)) => {
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

    fn parse_struct_item(&mut self, consume_endl: bool) -> PResult<ParseNode> {
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
                if consume_endl {
                    self.expect(&TokenType::EndL)?;
                }
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
        let type_name = self.expect_identifier()?;
        self.expect(&TokenType::Greater)?;
        let field_name = self.expect_identifier()?;
        self.expect(&TokenType::EndL)?;
        Ok(ParseNode::Field {
            data_type: Box::new(ParseNode::TypeStruct { name: type_name }),
            name: field_name,
        })
    }

    fn parse_struct_lit(&mut self) -> PResult<ParseNode> {
        self.expect(&TokenType::LBrace)?;
        let mut fields: Vec<(String, ParseNode)> = Vec::new();

        if matches!(self.peek(), Some(TokenType::RBrace)) {
            self.advance();
            return Ok(ParseNode::StructLit(fields));
        }

        loop {
            let name = self.expect_identifier()?;
            self.expect(&TokenType::Equals)?;
            let val = self.parse_expression()?;
            fields.push((name, val));

            match self.peek() {
                Some(TokenType::Comma) => {
                    self.advance();
                }
                Some(TokenType::RBrace) => break,
                other => {
                    return Err(ParseError::new(format!(
                        "Expected ',' or '}}' in struct literal, got {:?}",
                        other
                    )))
                }
            }
        }
        self.expect(&TokenType::RBrace)?;
        Ok(ParseNode::StructLit(fields))
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
                let else_block = self.parse_else_tail()?;
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

            Some(TokenType::TypeStruct) => self.parse_struct_item(false),

            Some(ref t) if Self::is_type_token(t) => self.parse_decl(),

            Some(TokenType::Identifier(_)) => self.parse_assign_or_expr_stmt(),

            _ => {
                let expr = self.parse_expression()?;
                Ok(ParseNode::ExprStmt(Box::new(expr)))
            }
        }
    }

    fn parse_else_tail(&mut self) -> PResult<Option<Vec<ParseNode>>> {
        match self.peek().cloned() {
            Some(TokenType::Elif) => {
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

            if matches!(self.peek(), Some(TokenType::LParen)) {
                return Err(ParseError::new("Qualified call is not a valid lvalue"));
            }
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
        while matches!(self.peek(), Some(TokenType::Or)) {
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
        let mut left = self.parse_lognot()?;
        while matches!(self.peek(), Some(TokenType::And)) {
            self.advance();
            let right = self.parse_lognot()?;
            left = ParseNode::LogAnd {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_lognot(&mut self) -> PResult<ParseNode> {
        if matches!(self.peek(), Some(TokenType::Not)) {
            self.advance();
            let operand = self.parse_lognot()?;
            return Ok(ParseNode::LogNot {
                operand: Box::new(operand),
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
        self.advance();
        let right = self.parse_bitor()?;
        Ok(ParseNode::Cmp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn parse_bitor(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_bitxor()?;
        while matches!(self.peek(), Some(TokenType::Pipe)) {
            self.advance();
            let right = self.parse_bitxor()?;
            left = ParseNode::BitOr {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_bitxor(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_bitand()?;
        while matches!(self.peek(), Some(TokenType::Caret)) {
            self.advance();
            let right = self.parse_bitand()?;
            left = ParseNode::BitXor {
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_bitand(&mut self) -> PResult<ParseNode> {
        let mut left = self.parse_add()?;
        while matches!(self.peek(), Some(TokenType::Ampersand)) {
            self.advance();
            let right = self.parse_add()?;
            left = ParseNode::BitAnd {
                left: Box::new(left),
                right: Box::new(right),
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

        self.parse_primary()
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

            Some(TokenType::LBrace) => self.parse_struct_lit(),

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
                    Some(TokenType::LBracket) => {        
                        let idx = self.parse_expression()?;
                        self.expect(&TokenType::RBracket)?;
                        Ok(ParseNode::Index {
                            target: Box::new(ParseNode::Identifier(name)),
                            index: Box::new(idx),
                        })
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
            Some(TokenType::RParen) | Some(TokenType::RBracket) | None
        ) {
            return Ok(args);
        }
        args.push(self.parse_expression()?);
        while matches!(self.peek(), Some(TokenType::Comma)) {
            self.advance();
            // handle trailing comma gracefully
            if matches!(
                self.peek(),
                Some(TokenType::RParen) | Some(TokenType::RBracket) | None
            ) {
                break;
            }
            args.push(self.parse_expression()?);
        }
        Ok(args)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<ParseNode, ParseError> {
    let mut parser = Parser::new(tokens);
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
        ParseNode::Return(_) => "Return".into(),
        ParseNode::Exit(_) => "Exit".into(),
        ParseNode::Break => "\x1b[35mBreak\x1b[0m".into(),
        ParseNode::Continue => "\x1b[35mContinue\x1b[0m".into(),
        ParseNode::ExprStmt(_) => "ExprStmt".into(),
        ParseNode::LValue { name, member } => match member {
            Some(m) => format!("LValue  \x1b[36m{}::{}\x1b[0m", name, m),
            None => format!("LValue  \x1b[36m{}\x1b[0m", name),
        },
        ParseNode::LogOr { .. } => "LogOr  \x1b[35m!or\x1b[0m".into(),
        ParseNode::LogAnd { .. } => "LogAnd  \x1b[35m!and\x1b[0m".into(),
        ParseNode::LogNot { .. } => "LogNot  \x1b[35m!not\x1b[0m".into(),
        ParseNode::Cmp { op, .. } => format!("Cmp  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::BitOr { .. } => "BitOr  \x1b[35m|\x1b[0m".into(),
        ParseNode::BitXor { .. } => "BitXor  \x1b[35m^\x1b[0m".into(),
        ParseNode::BitAnd { .. } => "BitAnd  \x1b[35m&\x1b[0m".into(),
        ParseNode::Add { op, .. } => format!("Add  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::Mul { op, .. } => format!("Mul  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::Unary { op, .. } => format!("Unary  \x1b[35m{:?}\x1b[0m", op),
        ParseNode::Cast { target_type, .. } => format!("Cast  → {}", type_str(target_type)),
        ParseNode::QualifiedCall {
            module,
            member,
            args,
        } => format!(
            "QualifiedCall  \x1b[33m{}::{}\x1b[0m  args={}",
            module,
            member,
            args.as_ref().map(|a| a.len()).unwrap_or(0)
        ),
        ParseNode::Call { name, args } => {
            format!("Call  \x1b[33m{}\x1b[0m  args={}", name, args.len())
        }
        ParseNode::ArrayLit(elems) => format!("ArrayLit  [{}]", elems.len()),
        ParseNode::StructLit(fields) => format!(
            "StructLit  {{{}}}",
            fields
                .iter()
                .map(|(k, _)| k.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ParseNode::Identifier(s) => format!("Identifier  \x1b[32m{}\x1b[0m", s),
        ParseNode::IntLit(n) => format!("IntLit  \x1b[32m{}\x1b[0m", n),
        ParseNode::FloatLit(f) => format!("FloatLit  \x1b[32m{}\x1b[0m", f),
        ParseNode::CharLit(c) => format!("CharLit  \x1b[32m{:?}\x1b[0m", c),
        ParseNode::StringLit(s) => format!("StringLit  \x1b[32m{:?}\x1b[0m", s),
        ParseNode::BoolLit(b) => format!("BoolLit  \x1b[32m{}\x1b[0m", b),
        ParseNode::Null => "\x1b[32mNull\x1b[0m".into(),
        ParseNode::TypeInt => "TypeInt".into(),
        ParseNode::TypeFloat => "TypeFloat".into(),
        ParseNode::TypeChar => "TypeChar".into(),
        ParseNode::TypeBoolean => "TypeBoolean".into(),
        ParseNode::TypeArray { elem, size } => format!("TypeArray<{},{}>", elem, size),
        ParseNode::TypeList { elem } => format!("TypeList<{}>", elem),
        ParseNode::TypeStruct { name } => format!("TypeStruct<{}>", name),
        ParseNode::Index { .. } => "Index".into(),
    }
}

fn type_str(node: &ParseNode) -> String {
    match node {
        ParseNode::TypeInt => "int".into(),
        ParseNode::TypeFloat => "float".into(),
        ParseNode::TypeChar => "char".into(),
        ParseNode::TypeBoolean => "bool".into(),
        ParseNode::TypeArray { elem, size } => format!("array<{},{}>", elem, size),
        ParseNode::TypeList { elem } => format!("list<{}>", elem),
        ParseNode::TypeStruct { name } => format!("struct<{}>", name),
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
                print_node(&i, &ip, true);
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
        ParseNode::While { condition, body } => {
            let cp = print_section_header("condition", prefix, false);
            print_node(condition, &cp, true);
            let bp = print_section_header("body", prefix, true);
            print_node_list(body, &bp);
        }
        ParseNode::Return(e) | ParseNode::Exit(e) => {
            print_node(e, prefix, true);
        }
        ParseNode::ExprStmt(e) => {
            print_node(e, prefix, true);
        }
        ParseNode::LogOr { left, right }
        | ParseNode::LogAnd { left, right }
        | ParseNode::BitOr { left, right }
        | ParseNode::BitXor { left, right }
        | ParseNode::BitAnd { left, right } => {
            print_node(left, prefix, false);
            print_node(right, prefix, true);
        }
        ParseNode::LogNot { operand } => {
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
        ParseNode::Call { args, .. } => {
            print_node_list(args, prefix);
        }
        ParseNode::QualifiedCall {
            args: Some(args), ..
        } => {
            print_node_list(args, prefix);
        }
        ParseNode::ArrayLit(elems) => {
            print_node_list(elems, prefix);
        }
        ParseNode::StructLit(fields) => {
            let n = fields.len();
            for (i, (name, val)) in fields.iter().enumerate() {
                let is_last = i == n - 1;
                let fp = print_section_header(name, prefix, is_last);
                print_node(val, &fp, true);
            }
        }
        ParseNode::Index { target, index } => {
            print_node(target, prefix, false);
            print_node(index, prefix, true);
        }

        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::lexer::tokenize;

    fn parse_prog(src: &str) -> ParseNode {
        let tokens = tokenize(src);
        parse(tokens).expect("parse failed")
    }

    fn wrap(body: &str) -> String {
        format!("!start\n{}\n!end", body)
    }

    #[test]
    fn test_decl_int_float() {
        let src = wrap(":int x = 42;\n:float y = 3.14;");
        let tree = parse_prog(&src);
        match &tree {
            ParseNode::Program(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    ParseNode::Decl {
                        name,
                        init: Some(i),
                        ..
                    } => {
                        assert_eq!(name, "x");
                        assert!(matches!(**i, ParseNode::IntLit(42)));
                    }
                    _ => panic!("expected Decl"),
                }
                match &items[1] {
                    ParseNode::Decl {
                        name,
                        init: Some(i),
                        ..
                    } => {
                        assert_eq!(name, "y");
                        assert!(matches!(**i, ParseNode::FloatLit(_)));
                    }
                    _ => panic!("expected Decl"),
                }
            }
            _ => panic!("expected Program"),
        }
    }

    #[test]
    fn test_if_elif_else() {
        let src = wrap("!if (a == 1) { x = 2; } !elif (a == 2) { x = 3; } !else { x = 4; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::If {
                else_block: Some(eb),
                ..
            } => {
                assert_eq!(eb.len(), 1);
                match &eb[0] {
                    ParseNode::If {
                        else_block: Some(eb2),
                        ..
                    } => {
                        assert_eq!(eb2.len(), 1);
                    }
                    _ => panic!("expected elif node"),
                }
            }
            _ => panic!("expected If"),
        }
    }

    #[test]
    fn test_for_loop() {
        let src = wrap("!for (:int i, 0, 10, 1) { x = i; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::For {
                var_name,
                start,
                stop,
                step,
                body,
                ..
            } => {
                assert_eq!(var_name, "i");
                assert!(matches!(**start, ParseNode::IntLit(0)));
                assert!(matches!(**stop, ParseNode::IntLit(10)));
                assert!(matches!(**step, ParseNode::IntLit(1)));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected For"),
        }
    }

    #[test]
    fn test_while_loop() {
        let src = wrap("!while (x > 0) { x -= 1; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::While { condition, body } => {
                assert!(matches!(**condition, ParseNode::Cmp { op: CmpOp::Gt, .. }));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected While"),
        }
    }

    #[test]
    fn test_funcdef() {
        let src = wrap("!func add(:int a, :int b) -> :int { !return a + b; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::FuncDef {
                name,
                params,
                return_type,
                body,
            } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert!(matches!(**return_type, ParseNode::TypeInt));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected FuncDef"),
        }
    }

    #[test]
    fn test_struct_def_and_decl() {
        let src = wrap(
            ":struct<Node> { :int val; :struct<Node> next; }\n\
             :struct<Node> n = {val = 1, next = NULL};",
        );
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::StructDef { name, fields } => {
                assert_eq!(name, "Node");
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("expected StructDef"),
        }
        match &items[1] {
            ParseNode::StructDecl {
                struct_name,
                var_name,
                init: Some(i),
            } => {
                assert_eq!(struct_name, "Node");
                assert_eq!(var_name, "n");
                assert!(matches!(**i, ParseNode::StructLit(_)));
            }
            _ => panic!("expected StructDecl with init"),
        }
    }

    #[test]
    fn test_logical_operators() {
        let src = wrap(":boolean r = a !and b !or !not c;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(expr), ..
            } => {
                assert!(
                    matches!(**expr, ParseNode::LogOr { .. }),
                    "expected LogOr at top, got {:?}",
                    expr
                );
            }
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_bitwise_operators() {
        let src = wrap(":int r = a | b ^ c & d;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(expr), ..
            } => match expr.as_ref() {
                ParseNode::BitOr { left, right } => {
                    assert!(matches!(**left, ParseNode::Identifier(_)));
                    assert!(matches!(**right, ParseNode::BitXor { .. }));
                }
                _ => panic!("expected BitOr at top"),
            },
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_assign_ops() {
        let src = wrap("x += 1;\nx -= 2;\nx *= 3;\nx /= 4;\nx %= 5;\nx &= 6;\nx |= 7;\nx ^= 8;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        assert_eq!(items.len(), 8);
        let expected_ops = [
            "PlusEq",
            "MinusEq",
            "StarEq",
            "SlashEq",
            "PercentEq",
            "AmpEq",
            "PipeEq",
            "CaretEq",
        ];
        for (item, _op_name) in items.iter().zip(expected_ops.iter()) {
            assert!(matches!(item, ParseNode::Assign { .. }), "expected Assign");
        }
    }

    #[test]
    fn test_cast_qualified_call_array() {
        let src = wrap(
            ":int x = :int(3.7);\n\
             math::sin(x);\n\
             :array<int, 3> arr = [1, 2, 3];",
        );
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        assert_eq!(items.len(), 3);

        match &items[0] {
            ParseNode::Decl { init: Some(i), .. } => {
                assert!(matches!(**i, ParseNode::Cast { .. }));
            }
            _ => panic!("expected Decl with Cast init"),
        }

        match &items[1] {
            ParseNode::ExprStmt(e) => {
                assert!(matches!(**e, ParseNode::QualifiedCall { .. }));
            }
            _ => panic!("expected ExprStmt with QualifiedCall"),
        }

        match &items[2] {
            ParseNode::Decl { init: Some(i), .. } => match i.as_ref() {
                ParseNode::ArrayLit(elems) => assert_eq!(elems.len(), 3),
                _ => panic!("expected ArrayLit"),
            },
            _ => panic!("expected Decl with ArrayLit"),
        }
    }
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use crate::compiler::lexer::tokenize;

    fn parse_prog(src: &str) -> ParseNode {
        let tokens = tokenize(src);
        parse(tokens).expect("parse failed")
    }

    fn wrap(body: &str) -> String {
        format!("!start\n{}\n!end", body)
    }

    fn matches_cmp(a: &CmpOp, b: &CmpOp) -> bool {
        matches!(
            (a, b),
            (CmpOp::Gt, CmpOp::Gt)
                | (CmpOp::Lt, CmpOp::Lt)
                | (CmpOp::Ge, CmpOp::Ge)
                | (CmpOp::Le, CmpOp::Le)
                | (CmpOp::EqEq, CmpOp::EqEq)
                | (CmpOp::Ne, CmpOp::Ne)
        )
    }

    #[test]
    fn test_decl_char_bool_noinit() {
        let src = wrap(":char c = 'z';\n:boolean flag = true;\n:int uninit;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        assert_eq!(items.len(), 3);

        match &items[0] {
            ParseNode::Decl {
                name,
                data_type,
                init: Some(i),
            } => {
                assert_eq!(name, "c");
                assert!(matches!(**data_type, ParseNode::TypeChar));
                assert!(matches!(**i, ParseNode::CharLit('z')));
            }
            _ => panic!("expected char Decl"),
        }
        match &items[1] {
            ParseNode::Decl {
                name,
                data_type,
                init: Some(i),
            } => {
                assert_eq!(name, "flag");
                assert!(matches!(**data_type, ParseNode::TypeBoolean));
                assert!(matches!(**i, ParseNode::BoolLit(true)));
            }
            _ => panic!("expected bool Decl"),
        }
        match &items[2] {
            ParseNode::Decl {
                name, init: None, ..
            } => assert_eq!(name, "uninit"),
            _ => panic!("expected uninitialised Decl"),
        }
    }

    #[test]
    fn test_decl_list() {
        let src = wrap(":list<int> nums;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                name, data_type, ..
            } => {
                assert_eq!(name, "nums");
                match data_type.as_ref() {
                    ParseNode::TypeList { elem } => assert_eq!(elem, "int"),
                    _ => panic!("expected TypeList"),
                }
            }
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_decl_array_string_init() {
        let src = wrap(":array<char, 10> s = \"hello\";");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                name,
                data_type,
                init: Some(i),
            } => {
                assert_eq!(name, "s");
                match data_type.as_ref() {
                    ParseNode::TypeArray { elem, size } => {
                        assert_eq!(elem, "char");
                        assert_eq!(*size, 10);
                    }
                    _ => panic!("expected TypeArray"),
                }
                assert!(matches!(**i, ParseNode::StringLit(_)));
            }
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_unary_neg_bitnot() {
        let src = wrap(":int a = -5;\n:int b = ~a;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };

        match &items[0] {
            ParseNode::Decl { init: Some(i), .. } => match i.as_ref() {
                ParseNode::Unary {
                    op: UnOp::Neg,
                    operand,
                } => {
                    assert!(matches!(**operand, ParseNode::IntLit(5)));
                }
                _ => panic!("expected Unary Neg"),
            },
            _ => panic!("expected Decl"),
        }
        match &items[1] {
            ParseNode::Decl { init: Some(i), .. } => {
                assert!(matches!(
                    **i,
                    ParseNode::Unary {
                        op: UnOp::BitNot,
                        ..
                    }
                ));
            }
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_lognot_right_assoc() {
        let src = wrap(":boolean r = !not !not x;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(outer), ..
            } => match outer.as_ref() {
                ParseNode::LogNot { operand: inner } => {
                    assert!(
                        matches!(**inner, ParseNode::LogNot { .. }),
                        "expected inner LogNot, got {:?}",
                        inner
                    );
                }
                _ => panic!("expected outer LogNot"),
            },
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_cmp_ops() {
        let src = wrap(
            ":boolean a = x > 1;\n\
             :boolean b = x < 2;\n\
             :boolean c = x >= 3;\n\
             :boolean d = x <= 4;\n\
             :boolean e = x == 5;\n\
             :boolean f = x ~= 6;",
        );
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        assert_eq!(items.len(), 6);

        let expected = [
            CmpOp::Gt,
            CmpOp::Lt,
            CmpOp::Ge,
            CmpOp::Le,
            CmpOp::EqEq,
            CmpOp::Ne,
        ];
        for (item, expected_op) in items.iter().zip(expected.iter()) {
            match item {
                ParseNode::Decl {
                    init: Some(expr), ..
                } => match expr.as_ref() {
                    ParseNode::Cmp { op, .. } => {
                        assert!(
                            matches_cmp(op, expected_op),
                            "expected {:?}, got {:?}",
                            expected_op,
                            op
                        );
                    }
                    _ => panic!("expected Cmp, got {:?}", expr),
                },
                _ => panic!("expected Decl"),
            }
        }
    }

    #[test]
    fn test_arith_precedence() {
        let src = wrap(":int r = 2 + 3 * 4;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(expr), ..
            } => match expr.as_ref() {
                ParseNode::Add { left, right, .. } => {
                    assert!(matches!(**left, ParseNode::IntLit(2)));
                    assert!(matches!(**right, ParseNode::Mul { .. }));
                }
                _ => panic!("expected Add at top, got {:?}", expr),
            },
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_paren_precedence() {
        let src = wrap(":int r = (2 + 3) * 4;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(expr), ..
            } => match expr.as_ref() {
                ParseNode::Mul { left, right, .. } => {
                    assert!(matches!(**left, ParseNode::Add { .. }));
                    assert!(matches!(**right, ParseNode::IntLit(4)));
                }
                _ => panic!("expected Mul at top, got {:?}", expr),
            },
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_logical_vs_bitwise_precedence() {
        let src = wrap(":boolean r = a !and b | c;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(expr), ..
            } => match expr.as_ref() {
                ParseNode::LogAnd { left, right } => {
                    assert!(matches!(**left, ParseNode::Identifier(_)));
                    assert!(
                        matches!(**right, ParseNode::BitOr { .. }),
                        "expected BitOr on right of !and, got {:?}",
                        right
                    );
                }
                _ => panic!("expected LogAnd at top, got {:?}", expr),
            },
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_plain_assign() {
        let src = wrap("x = 99;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Assign { lvalue, op, expr } => {
                assert!(matches!(**lvalue, ParseNode::LValue { member: None, .. }));
                assert!(matches!(op, AssignOp::Eq));
                assert!(matches!(**expr, ParseNode::IntLit(99)));
            }
            _ => panic!("expected Assign"),
        }
    }

    #[test]
    fn test_struct_member_assign() {
        let src = wrap("node::next = NULL;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Assign { lvalue, op, expr } => {
                match lvalue.as_ref() {
                    ParseNode::LValue {
                        name,
                        member: Some(m),
                    } => {
                        assert_eq!(name, "node");
                        assert_eq!(m, "next");
                    }
                    _ => panic!("expected LValue with member"),
                }
                assert!(matches!(op, AssignOp::Eq));
                assert!(matches!(**expr, ParseNode::Null));
            }
            _ => panic!("expected Assign"),
        }
    }

    #[test]
    fn test_qualified_field_access() {
        let src = wrap(":float x = math::pi;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(expr), ..
            } => match expr.as_ref() {
                ParseNode::QualifiedCall {
                    module,
                    member,
                    args: None,
                } => {
                    assert_eq!(module, "math");
                    assert_eq!(member, "pi");
                }
                _ => panic!("expected QualifiedCall (no args), got {:?}", expr),
            },
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_funcdef_no_params() {
        let src = wrap("!func noop() -> :boolean { !return true; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::FuncDef {
                name,
                params,
                return_type,
                body,
            } => {
                assert_eq!(name, "noop");
                assert!(params.is_empty());
                assert!(matches!(**return_type, ParseNode::TypeBoolean));
                assert_eq!(body.len(), 1);
            }
            _ => panic!("expected FuncDef"),
        }
    }

    #[test]
    fn test_funcdef_struct_param_return() {
        let src = wrap("!func clone(:struct<Node> n) -> :struct<Node> { !return n; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::FuncDef {
                name,
                params,
                return_type,
                ..
            } => {
                assert_eq!(name, "clone");
                assert_eq!(params.len(), 1);
                match &params[0] {
                    ParseNode::Param {
                        data_type,
                        name: pname,
                    } => {
                        assert_eq!(pname, "n");
                        assert!(matches!(**data_type, ParseNode::TypeStruct { .. }));
                    }
                    _ => panic!("expected Param"),
                }
                assert!(matches!(**return_type, ParseNode::TypeStruct { .. }));
            }
            _ => panic!("expected FuncDef"),
        }
    }

    #[test]
    fn test_if_no_else() {
        let src = wrap("!if (x == 0) { x = 1; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::If {
                condition,
                then_block,
                else_block,
            } => {
                assert!(matches!(
                    **condition,
                    ParseNode::Cmp {
                        op: CmpOp::EqEq,
                        ..
                    }
                ));
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_none());
            }
            _ => panic!("expected If"),
        }
    }

    #[test]
    fn test_nested_if_in_while() {
        let src = wrap("!while (x > 0) { !if (x == 1) { x = 0; } }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::While { body, .. } => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], ParseNode::If { .. }));
            }
            _ => panic!("expected While"),
        }
    }

    #[test]
    fn test_break_continue_in_for() {
        let src = wrap("!for (:int i, 0, 10, 1) { !break; !continue; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::For { body, .. } => {
                assert_eq!(body.len(), 2);
                assert!(matches!(body[0], ParseNode::Break));
                assert!(matches!(body[1], ParseNode::Continue));
            }
            _ => panic!("expected For"),
        }
    }

    #[test]
    fn test_exit_stmt() {
        let src = wrap("!exit 0;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Exit(expr) => {
                assert!(matches!(**expr, ParseNode::IntLit(0)));
            }
            _ => panic!("expected Exit"),
        }
    }

    #[test]
    fn test_return_complex_expr() {
        let src = wrap("!func f(:int a, :int b) -> :int { !return (a + b) * 2; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::FuncDef { body, .. } => match &body[0] {
                ParseNode::Return(expr) => {
                    assert!(matches!(**expr, ParseNode::Mul { .. }));
                }
                _ => panic!("expected Return"),
            },
            _ => panic!("expected FuncDef"),
        }
    }

    #[test]
    fn test_funcdef_empty_body() {
        let src = wrap("!func empty() -> :int {}");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::FuncDef { name, body, .. } => {
                assert_eq!(name, "empty");
                assert!(body.is_empty());
            }
            _ => panic!("expected FuncDef"),
        }
    }

    #[test]
    fn test_for_empty_body() {
        let src = wrap("!for (:int i, 0, 5, 1) {}");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::For { var_name, body, .. } => {
                assert_eq!(var_name, "i");
                assert!(body.is_empty());
            }
            _ => panic!("expected For"),
        }
    }

    #[test]
    fn test_empty_array_lit() {
        let src = wrap(":list<int> x = [];");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(expr), ..
            } => match expr.as_ref() {
                ParseNode::ArrayLit(elems) => assert!(elems.is_empty()),
                _ => panic!("expected ArrayLit"),
            },
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_struct_decl_no_init() {
        let src = wrap(":struct<Node> n;");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::StructDecl {
                struct_name,
                var_name,
                init,
            } => {
                assert_eq!(struct_name, "Node");
                assert_eq!(var_name, "n");
                assert!(init.is_none());
            }
            _ => panic!("expected StructDecl"),
        }
    }

    #[test]
    fn test_struct_def_primitives() {
        let src = wrap(":struct<Vec3> { :float x; :float y; :float z; }");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::StructDef { name, fields } => {
                assert_eq!(name, "Vec3");
                assert_eq!(fields.len(), 3);
                for f in fields {
                    match f {
                        ParseNode::Field { data_type, .. } => {
                            assert!(matches!(**data_type, ParseNode::TypeFloat));
                        }
                        _ => panic!("expected Field"),
                    }
                }
            }
            _ => panic!("expected StructDef"),
        }
    }

    #[test]
    fn test_three_elif_chain() {
        let src = wrap(
            "!if (x == 1) { a = 1; } \
             !elif (x == 2) { a = 2; } \
             !elif (x == 3) { a = 3; } \
             !elif (x == 4) { a = 4; } \
             !else { a = 5; }",
        );
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };

        let mut node = &items[0];
        let mut depth = 0usize;
        loop {
            match node {
                ParseNode::If {
                    else_block: Some(eb),
                    ..
                } => {
                    depth += 1;
                    if !matches!(eb[0], ParseNode::If { .. }) {
                        break;
                    }
                    node = &eb[0];
                }
                _ => panic!("expected If at depth {}", depth),
            }
        }

        assert_eq!(depth, 4);
    }

    #[test]
    fn test_nested_arithmetic() {
        let src = wrap(":int r = (a + b) * (c - d);");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::Decl {
                init: Some(expr), ..
            } => match expr.as_ref() {
                ParseNode::Mul { left, right, .. } => {
                    assert!(matches!(**left, ParseNode::Add { op: AddOp::Add, .. }));
                    assert!(matches!(**right, ParseNode::Add { op: AddOp::Sub, .. }));
                }
                _ => panic!("expected Mul at top, got {:?}", expr),
            },
            _ => panic!("expected Decl"),
        }
    }

    #[test]
    fn test_call_multi_args() {
        let src = wrap("foo(1, 2, 3);");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::ExprStmt(e) => match e.as_ref() {
                ParseNode::Call { name, args } => {
                    assert_eq!(name, "foo");
                    assert_eq!(args.len(), 3);
                    assert!(matches!(args[0], ParseNode::IntLit(1)));
                    assert!(matches!(args[1], ParseNode::IntLit(2)));
                    assert!(matches!(args[2], ParseNode::IntLit(3)));
                }
                _ => panic!("expected Call"),
            },
            _ => panic!("expected ExprStmt"),
        }
    }

    #[test]
    fn test_call_no_args() {
        let src = wrap("tick();");
        let tree = parse_prog(&src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        match &items[0] {
            ParseNode::ExprStmt(e) => match e.as_ref() {
                ParseNode::Call { name, args } => {
                    assert_eq!(name, "tick");
                    assert!(args.is_empty());
                }
                _ => panic!("expected Call"),
            },
            _ => panic!("expected ExprStmt"),
        }
    }

    #[test]
    fn test_module_block() {
        let src = "!start\n$MODULE_START:mymod$\n:int x = 1;\n$MODULE_END:mymod$;\n!end";
        let tree = parse_prog(src);
        let items = match &tree {
            ParseNode::Program(i) => i,
            _ => panic!(),
        };
        assert_eq!(items.len(), 1);
        match &items[0] {
            ParseNode::Module {
                name,
                items: mod_items,
            } => {
                assert_eq!(name, "mymod");
                assert_eq!(mod_items.len(), 1);
                assert!(matches!(mod_items[0], ParseNode::Decl { .. }));
            }
            _ => panic!("expected Module"),
        }
    }

    #[test]
    fn test_parse_error_bad_expr() {
        let tokens = tokenize("!start\n:int x = == 1;\n!end");
        let result = parse(tokens);
        assert!(result.is_err(), "expected ParseError, got Ok");
    }
}
