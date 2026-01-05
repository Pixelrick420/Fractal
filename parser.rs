use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    Ident(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Assign,
    Greater,
    Less,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assignment {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    For {
        init: Box<Stmt>,
        condition: Expr,
        increment: Box<Stmt>,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Exit {
        code: Expr,
    },
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

fn current_token<'a>(tokens: &'a Vec<Token>, pos: usize) -> Result<&'a Token, String> {
    if pos < tokens.len() {
        Ok(&tokens[pos])
    } else {
        Err("Unexpected end of tokens".to_string())
    }
}

fn is_token(tokens: &Vec<Token>, pos: usize, expected: &str) -> bool {
    if let Ok(token) = current_token(tokens, pos) {
        match &token.token_type {
            TokenType::Start => expected == "!start",
            TokenType::End => expected == "!end",
            TokenType::Exit => expected == "!exit",
            TokenType::If => expected == "!if",
            TokenType::Else => expected == "!else",
            TokenType::For => expected == "!for",
            TokenType::While => expected == "!while",
            TokenType::EndL => expected == ";",
            TokenType::Plus => expected == "+",
            TokenType::Minus => expected == "-",
            TokenType::Star => expected == "*",
            TokenType::Equals => expected == "=",
            TokenType::LParen => expected == "(",
            TokenType::RParen => expected == ")",
            TokenType::LBrace => expected == "{",
            TokenType::RBrace => expected == "}",
            TokenType::Greater => expected == ">",
            TokenType::Less => expected == "<",
            TokenType::Identifier(_) => expected == "identifier",
            TokenType::SIntLit(_) | TokenType::UIntLit(_) => expected == "number",
            _ => false,
        }
    } else {
        false
    }
}

fn expect(tokens: &Vec<Token>, pos: &mut usize, expected: &str) -> Result<(), String> {
    if is_token(tokens, *pos, expected) {
        *pos += 1;
        Ok(())
    } else {
        let token = current_token(tokens, *pos)?;
        Err(format!(
            "Expected '{}' but found {:?}",
            expected, token.token_type
        ))
    }
}

fn parse_primary(tokens: &Vec<Token>, pos: &mut usize) -> Result<Expr, String> {
    let token = current_token(tokens, *pos)?;

    match &token.token_type {
        TokenType::SIntLit(val) => {
            let val = *val;
            *pos += 1;
            Ok(Expr::IntLit(val))
        }
        TokenType::UIntLit(val) => {
            let val = *val as i64;
            *pos += 1;
            Ok(Expr::IntLit(val))
        }
        TokenType::Identifier(name) => {
            let name = name.clone();
            *pos += 1;
            Ok(Expr::Ident(name))
        }
        TokenType::LParen => {
            *pos += 1;
            let expr = parse_expression(tokens, pos)?;
            expect(tokens, pos, ")")?;
            Ok(expr)
        }
        _ => Err(format!("Expected expression, found {:?}", token.token_type)),
    }
}

fn parse_multiplicative(tokens: &Vec<Token>, pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_primary(tokens, pos)?;

    while is_token(tokens, *pos, "*") {
        *pos += 1;
        let right = parse_primary(tokens, pos)?;
        left = Expr::BinaryOp {
            left: Box::new(left),
            op: BinOp::Mul,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_additive(tokens: &Vec<Token>, pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_multiplicative(tokens, pos)?;

    while is_token(tokens, *pos, "+") || is_token(tokens, *pos, "-") {
        let op = if is_token(tokens, *pos, "+") {
            *pos += 1;
            BinOp::Add
        } else {
            *pos += 1;
            BinOp::Sub
        };

        let right = parse_multiplicative(tokens, pos)?;
        left = Expr::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_comparison(tokens: &Vec<Token>, pos: &mut usize) -> Result<Expr, String> {
    let mut left = parse_additive(tokens, pos)?;

    while is_token(tokens, *pos, ">") || is_token(tokens, *pos, "<") {
        let op = if is_token(tokens, *pos, ">") {
            *pos += 1;
            BinOp::Greater
        } else {
            *pos += 1;
            BinOp::Less
        };

        let right = parse_additive(tokens, pos)?;
        left = Expr::BinaryOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_expression(tokens: &Vec<Token>, pos: &mut usize) -> Result<Expr, String> {
    parse_comparison(tokens, pos)
}

fn parse_assignment(
    tokens: &Vec<Token>,
    pos: &mut usize,
    expect_semicolon: bool,
) -> Result<Stmt, String> {
    let token = current_token(tokens, *pos)?;

    if let TokenType::Identifier(name) = &token.token_type {
        let name = name.clone();
        *pos += 1;
        expect(tokens, pos, "=")?;
        let value = parse_expression(tokens, pos)?;

        if expect_semicolon {
            expect(tokens, pos, ";")?;
        }

        Ok(Stmt::Assignment { name, value })
    } else {
        Err(format!("Expected identifier, found {:?}", token.token_type))
    }
}

fn parse_if(tokens: &Vec<Token>, pos: &mut usize) -> Result<Stmt, String> {
    expect(tokens, pos, "!if")?;
    expect(tokens, pos, "(")?;
    let condition = parse_expression(tokens, pos)?;
    expect(tokens, pos, ")")?;
    expect(tokens, pos, "{")?;

    let mut then_body = Vec::new();
    while !is_token(tokens, *pos, "}") {
        then_body.push(parse_statement(tokens, pos)?);
    }
    expect(tokens, pos, "}")?;

    let else_body = if is_token(tokens, *pos, "!else") {
        *pos += 1;
        expect(tokens, pos, "{")?;
        let mut stmts = Vec::new();
        while !is_token(tokens, *pos, "}") {
            stmts.push(parse_statement(tokens, pos)?);
        }
        expect(tokens, pos, "}")?;
        Some(stmts)
    } else {
        None
    };

    Ok(Stmt::If {
        condition,
        then_body,
        else_body,
    })
}

fn parse_for(tokens: &Vec<Token>, pos: &mut usize) -> Result<Stmt, String> {
    expect(tokens, pos, "!for")?;
    expect(tokens, pos, "(")?;

    let init = Box::new(parse_assignment(tokens, pos, true)?);
    let condition = parse_expression(tokens, pos)?;
    expect(tokens, pos, ";")?;
    let increment = Box::new(parse_assignment(tokens, pos, false)?);

    expect(tokens, pos, ")")?;
    expect(tokens, pos, "{")?;

    let mut body = Vec::new();
    while !is_token(tokens, *pos, "}") {
        body.push(parse_statement(tokens, pos)?);
    }
    expect(tokens, pos, "}")?;

    Ok(Stmt::For {
        init,
        condition,
        increment,
        body,
    })
}

fn parse_while(tokens: &Vec<Token>, pos: &mut usize) -> Result<Stmt, String> {
    expect(tokens, pos, "!while")?;
    expect(tokens, pos, "(")?;
    let condition = parse_expression(tokens, pos)?;
    expect(tokens, pos, ")")?;
    expect(tokens, pos, "{")?;

    let mut body = Vec::new();
    while !is_token(tokens, *pos, "}") {
        body.push(parse_statement(tokens, pos)?);
    }
    expect(tokens, pos, "}")?;

    Ok(Stmt::While { condition, body })
}

fn parse_exit(tokens: &Vec<Token>, pos: &mut usize) -> Result<Stmt, String> {
    expect(tokens, pos, "!exit")?;
    let code = parse_expression(tokens, pos)?;
    expect(tokens, pos, ";")?;
    Ok(Stmt::Exit { code })
}

fn parse_statement(tokens: &Vec<Token>, pos: &mut usize) -> Result<Stmt, String> {
    if is_token(tokens, *pos, "!if") {
        parse_if(tokens, pos)
    } else if is_token(tokens, *pos, "!for") {
        parse_for(tokens, pos)
    } else if is_token(tokens, *pos, "!while") {
        parse_while(tokens, pos)
    } else if is_token(tokens, *pos, "!exit") {
        parse_exit(tokens, pos)
    } else if is_token(tokens, *pos, "identifier") {
        parse_assignment(tokens, pos, true)
    } else {
        let token = current_token(tokens, *pos)?;
        Err(format!("Unexpected token: {:?}", token.token_type))
    }
}

fn parse_program(tokens: &Vec<Token>, pos: &mut usize) -> Result<Program, String> {
    expect(tokens, pos, "!start")?;

    let mut statements = Vec::new();
    while !is_token(tokens, *pos, "!end") {
        statements.push(parse_statement(tokens, pos)?);
    }

    expect(tokens, pos, "!end")?;

    Ok(Program { statements })
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, String> {
    let mut pos = 0;
    return parse_program(&tokens, &mut pos);
}
