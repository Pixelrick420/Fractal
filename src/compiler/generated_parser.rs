use std::mem::discriminant;

use super::lexer::TokenType;
#[derive(Debug)]
pub struct Node {
    name: String,
    children: Vec<Node>,
}

impl Node {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), children: vec![] }
    }

    fn add(&mut self, n: Node) {
        self.children.push(n);
    }

    pub fn print(&self, depth: usize) {
        for _ in 0..depth { print!("  "); }
        println!("{}", self.name);
        for c in &self.children {
            c.print(depth + 1);
        }
    }
}

use crate::compiler::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenType {
        &self.tokens[self.pos].token_type
    }

    fn advance(&mut self) -> TokenType {
    let t = self.tokens[self.pos].token_type.clone();
    self.pos += 1;
    t
    }

    fn expect(&mut self, expected: TokenType) {
        if discriminant(self.peek()) == discriminant(&expected) {
            self.advance();
        } else {
            panic!("Expected {:?}, found {:?}", expected, self.peek());
        }
    }

    fn match_token(&mut self, t: TokenType) -> bool {
        if discriminant(self.peek()) == discriminant(&t) {
            self.advance();
            true
        } else { false }
    }

    // =====================================================
    // PRGM -> Start ITEM_LIST End
    // =====================================================

    pub fn parse_prgm(&mut self) -> Node {
        let mut n = Node::new("PRGM");
        self.expect(TokenType::Start);
        n.add(self.parse_item_list());
        self.expect(TokenType::End);
        n
    }

    // =====================================================
    // ITEM_LIST
    // =====================================================

    fn parse_item_list(&mut self) -> Node {
        let mut n = Node::new("ITEM_LIST");
        while !matches!(self.peek(), TokenType::End) {
            n.add(self.parse_item());
        }
        n
    }

    fn parse_item(&mut self) -> Node {
        let mut n = Node::new("ITEM");
        match self.peek() {
            TokenType::Struct => n.add(self.parse_structdef()),
            TokenType::Func => {
                n.add(self.parse_funcdef());
                n.add(self.parse_func_end());
            }
            _ => {
                n.add(self.parse_stmt());
                self.expect(TokenType::EndL);
            }
        }
        n
    }

    fn parse_func_end(&mut self) -> Node {
        let mut n = Node::new("FUNC_END");
        self.match_token(TokenType::EndL);
        n
    }

    fn parse_structdef(&mut self) -> Node {
        let mut n = Node::new("STRUCTDEF");
        self.expect(TokenType::Struct);
        n
    }

    fn parse_funcdef(&mut self) -> Node {
        let mut n = Node::new("FUNCDEF");
        self.expect(TokenType::Func);
        self.advance();
        self.expect(TokenType::LParen);
        self.expect(TokenType::RParen);
        self.expect(TokenType::Arrow);
        n.add(self.parse_rettype());
        n.add(self.parse_blk());
        n
    }

    fn parse_rettype(&mut self) -> Node {
        let mut n = Node::new("RETTYPE");
        self.advance();
        n
    }

    fn parse_blk(&mut self) -> Node {
        let mut n = Node::new("BLK");
        self.expect(TokenType::LBrace);
        n.add(self.parse_stmts());
        self.expect(TokenType::RBrace);
        n
    }

    fn parse_stmts(&mut self) -> Node {
        let mut n = Node::new("STMTS");
        while !matches!(self.peek(), TokenType::RBrace) {
            n.add(self.parse_stmt());
            self.expect(TokenType::EndL);
        }
        n
    }

    fn parse_stmt(&mut self) -> Node {
        let mut n = Node::new("STMT");
        match self.peek() {
            TokenType::If => {
                self.advance();
                self.expect(TokenType::LParen);
                n.add(self.parse_expr());
                self.expect(TokenType::RParen);
                n.add(self.parse_blk());
                n.add(self.parse_elsepart());
            }
            TokenType::While => {
                self.advance();
                self.expect(TokenType::LParen);
                n.add(self.parse_expr());
                self.expect(TokenType::RParen);
                n.add(self.parse_blk());
            }
            TokenType::Return => {
                self.advance();
                n.add(self.parse_expr());
            }
            _ => n.add(self.parse_expr()),
        }
        n
    }

    fn parse_elsepart(&mut self) -> Node {
        let mut n = Node::new("ELSEPART");
        if self.match_token(TokenType::Else) {
            n.add(self.parse_blk());
        }
        n
    }

    // =======================
    // EXPRESSION CHAIN
    // =======================

    fn parse_expr(&mut self) -> Node {
        let mut n = Node::new("EXPR");
        n.add(self.parse_logor());
        if matches!(self.peek(),
            TokenType::Equals |
            TokenType::PlusEquals |
            TokenType::MinusEquals |
            TokenType::StarEquals |
            TokenType::SlashEquals |
            TokenType::PercentEquals |
            TokenType::AmpersandEquals |
            TokenType::PipeEquals |
            TokenType::CaretEquals) {
            self.advance();
            n.add(self.parse_expr());
        }
        n
    }

    fn parse_logor(&mut self) -> Node {
        let mut n = Node::new("LOGOR");
        n.add(self.parse_logand());
        while self.match_token(TokenType::Pipe) {
            n.add(self.parse_logand());
        }
        n
    }

    fn parse_logand(&mut self) -> Node {
        let mut n = Node::new("LOGAND");
        n.add(self.parse_cmp());
        while self.match_token(TokenType::Ampersand) {
            n.add(self.parse_cmp());
        }
        n
    }

    fn parse_cmp(&mut self) -> Node {
        let mut n = Node::new("CMP");
        n.add(self.parse_add());
        while matches!(self.peek(),
            TokenType::Greater |
            TokenType::Less |
            TokenType::GreaterEquals |
            TokenType::LessEquals |
            TokenType::EqualsEquals |
            TokenType::TildeEquals) {
            self.advance();
            n.add(self.parse_add());
        }
        n
    }

    fn parse_add(&mut self) -> Node {
        let mut n = Node::new("ADD");
        n.add(self.parse_mul());
        while matches!(self.peek(), TokenType::Plus | TokenType::Minus) {
            self.advance();
            n.add(self.parse_mul());
        }
        n
    }

    fn parse_mul(&mut self) -> Node {
        let mut n = Node::new("MUL");
        n.add(self.parse_unary());
        while matches!(self.peek(), TokenType::Star | TokenType::Slash | TokenType::Percent) {
            self.advance();
            n.add(self.parse_unary());
        }
        n
    }

    fn parse_unary(&mut self) -> Node {
        let mut n = Node::new("UNARY");
        if matches!(self.peek(), TokenType::Minus | TokenType::Tilde | TokenType::Ampersand) {
            self.advance();
            n.add(self.parse_unary());
        } else {
            n.add(self.parse_primary());
        }
        n
    }

    fn parse_primary(&mut self) -> Node {
        let mut n = Node::new("PRIMARY");
        match self.advance() {
            TokenType::Identifier(name) => n.add(Node::new(&format!("IDENT({})", name))),
            TokenType::SIntLit(v) => n.add(Node::new(&format!("INT({})", v))),
            TokenType::FloatLit(v) => n.add(Node::new(&format!("FLOAT({})", v))),
            TokenType::StringLit(v) => n.add(Node::new(&format!("STRING({})", v))),
            TokenType::BoolLit(v) => n.add(Node::new(&format!("BOOL({})", v))),
            TokenType::Null => n.add(Node::new("NULL")),
            TokenType::LParen => {
                n.add(self.parse_expr());
                self.expect(TokenType::RParen);
            }
            other => panic!("Unexpected token in PRIMARY: {:?}", other),
        }
        n
    }
}
