use super::lexer::Token;
pub struct Node{
    pub name: String,
    pub children: Vec<Node>
}
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum Symbol {
    NonTerminal(String), // just store the name, look up in the map
    Terminal(String),
}

#[derive(Debug, Clone)]
struct NonTerminal {
    name: String,
    productions: Vec<Vec<Symbol>>, // each Vec<Symbol> is one alternative
}

// helper macros to reduce verbosity
macro_rules! nt {
    ($name:expr) => {
        Symbol::NonTerminal($name.to_string())
    };
}
macro_rules! t {
    ($name:expr) => {
        Symbol::Terminal($name.to_string())
    };
}
macro_rules! prod {
    ($($sym:expr),*) => {
        vec![$($sym),*]
    };
}
struct Tokens{
    tokens: Vec<Token>,
    index: usize,
}
fn create_productions() -> HashMap<String, NonTerminal> {
    let mut map: HashMap<String, NonTerminal> = HashMap::new();

    // -------------------------------------------------------------------------
    // PRGM -> Start ITEM* End
    // We model ITEM* as ITEMS (a recursive non-terminal)
    // -------------------------------------------------------------------------
    map.insert("PRGM".to_string(), NonTerminal {
        name: "PRGM".to_string(),
        productions: vec![
            prod![t!("Start"), nt!("ITEMS"), t!("End")],
        ],
    });

    // ITEMS -> ITEM ITEMS | epsilon
    map.insert("ITEMS".to_string(), NonTerminal {
        name: "ITEMS".to_string(),
        productions: vec![
            prod![nt!("ITEM"), nt!("ITEMS")],
            prod![t!("EPSILON")],
        ],
    });

    // ITEM -> MODULE | STRUCTDEF | FUNCDEF | STMT EndL
    map.insert("ITEM".to_string(), NonTerminal {
        name: "ITEM".to_string(),
        productions: vec![
            prod![nt!("MODULE")],
            prod![nt!("STRUCTDEF")],
            prod![nt!("FUNCDEF")],
            prod![nt!("STMT"), t!("EndL")],
        ],
    });

    // -------------------------------------------------------------------------
    // MODULE -> ModuleStart MODITEMS ModuleEnd
    // -------------------------------------------------------------------------
    map.insert("MODULE".to_string(), NonTerminal {
        name: "MODULE".to_string(),
        productions: vec![
            prod![t!("ModuleStart"), nt!("MODITEMS"), t!("ModuleEnd")],
        ],
    });

    // MODITEMS -> MODITEM MODITEMS | epsilon
    map.insert("MODITEMS".to_string(), NonTerminal {
        name: "MODITEMS".to_string(),
        productions: vec![
            prod![nt!("MODITEM"), nt!("MODITEMS")],
            prod![t!("EPSILON")],
        ],
    });

    // MODITEM -> MODFUNCDEF | MODDECL | STMT EndL
    map.insert("MODITEM".to_string(), NonTerminal {
        name: "MODITEM".to_string(),
        productions: vec![
            prod![nt!("MODFUNCDEF")],
            prod![nt!("MODDECL")],
            prod![nt!("STMT"), t!("EndL")],
        ],
    });

    // MODFUNCDEF -> pub? fn IDENT ( PARAMS ) -> RETTYPE BLK
    // pub and fn are identifiers in your lexer
    map.insert("MODFUNCDEF".to_string(), NonTerminal {
        name: "MODFUNCDEF".to_string(),
        productions: vec![
            prod![t!("Identifier(pub)"), t!("Identifier(fn)"), t!("Identifier"), t!("LParen"), nt!("PARAMS"), t!("RParen"), t!("Arrow"), nt!("RETTYPE"), nt!("BLK")],
            prod![t!("Identifier(fn)"), t!("Identifier"), t!("LParen"), nt!("PARAMS"), t!("RParen"), t!("Arrow"), nt!("RETTYPE"), nt!("BLK")],
        ],
    });

    // MODDECL -> let IDENT DTYPE = EXPR EndL
    map.insert("MODDECL".to_string(), NonTerminal {
        name: "MODDECL".to_string(),
        productions: vec![
            prod![t!("Identifier(let)"), t!("Identifier"), nt!("DTYPE"), t!("Equals"), nt!("EXPR"), t!("EndL")],
        ],
    });

    // -------------------------------------------------------------------------
    // FUNCDEF -> func IDENT ( PARAMS ) -> RETTYPE BLK
    // -------------------------------------------------------------------------
    map.insert("FUNCDEF".to_string(), NonTerminal {
        name: "FUNCDEF".to_string(),
        productions: vec![
            prod![t!("Func"), t!("Identifier"), t!("LParen"), nt!("PARAMS"), t!("RParen"), t!("Arrow"), nt!("RETTYPE"), nt!("BLK")],
        ],
    });

    // RETTYPE -> DTYPE | TypeVoid | NoMatch
    map.insert("RETTYPE".to_string(), NonTerminal {
        name: "RETTYPE".to_string(),
        productions: vec![
            prod![nt!("DTYPE")],
            prod![t!("TypeVoid")],
            prod![t!("NoMatch")],
        ],
    });

    // -------------------------------------------------------------------------
    // STRUCTDEF -> struct < IDENT > { FIELDS }
    // -------------------------------------------------------------------------
    map.insert("STRUCTDEF".to_string(), NonTerminal {
        name: "STRUCTDEF".to_string(),
        productions: vec![
            prod![t!("TypeStruct"), t!("Less"), t!("Identifier"), t!("Greater"), t!("LBrace"), nt!("FIELDS"), t!("RBrace")],
        ],
    });

    // FIELDS -> FIELD EndL FIELDS | epsilon
    map.insert("FIELDS".to_string(), NonTerminal {
        name: "FIELDS".to_string(),
        productions: vec![
            prod![nt!("FIELD"), t!("EndL"), nt!("FIELDS")],
            prod![t!("EPSILON")],
        ],
    });

    // FIELD -> DTYPE IDENT
    map.insert("FIELD".to_string(), NonTerminal {
        name: "FIELD".to_string(),
        productions: vec![
            prod![nt!("DTYPE"), t!("Identifier")],
        ],
    });

    // -------------------------------------------------------------------------
    // BLK -> { STMTS }
    // -------------------------------------------------------------------------
    map.insert("BLK".to_string(), NonTerminal {
        name: "BLK".to_string(),
        productions: vec![
            prod![t!("LBrace"), nt!("STMTS"), t!("RBrace")],
        ],
    });

    // STMTS -> STMT EndL STMTS | epsilon
    map.insert("STMTS".to_string(), NonTerminal {
        name: "STMTS".to_string(),
        productions: vec![
            prod![nt!("STMT"), t!("EndL"), nt!("STMTS")],
            prod![t!("EPSILON")],
        ],
    });

    // -------------------------------------------------------------------------
    // STMT -> DECL | ASSIGN | EXPR | IF | FOR | WHILE | RETURN | BREAK | CONT
    // -------------------------------------------------------------------------
    map.insert("STMT".to_string(), NonTerminal {
        name: "STMT".to_string(),
        productions: vec![
            prod![nt!("DECL")],
            prod![nt!("ASSIGN")],
            prod![nt!("IFSTMT")],
            prod![nt!("FORSTMT")],
            prod![nt!("WHILESTMT")],
            prod![nt!("RETSTMT")],
            prod![t!("Break")],
            prod![t!("Continue")],
            prod![nt!("EXPR")],  // standalone calls like foo();
        ],
    });

    // -------------------------------------------------------------------------
    // IF -> if ( EXPR ) BLK ELSEPART
    // -------------------------------------------------------------------------
    map.insert("IFSTMT".to_string(), NonTerminal {
        name: "IFSTMT".to_string(),
        productions: vec![
            prod![t!("If"), t!("LParen"), nt!("EXPR"), t!("RParen"), nt!("BLK"), nt!("ELSEPART")],
        ],
    });

    // ELSEPART -> else BLK | epsilon
    map.insert("ELSEPART".to_string(), NonTerminal {
        name: "ELSEPART".to_string(),
        productions: vec![
            prod![t!("Else"), nt!("BLK")],
            prod![t!("EPSILON")],
        ],
    });

    // -------------------------------------------------------------------------
    // FOR -> for ( DTYPE IDENT , EXPR , EXPR , EXPR ) BLK
    // 4 parts: variable, start, end, step
    // -------------------------------------------------------------------------
    map.insert("FORSTMT".to_string(), NonTerminal {
        name: "FORSTMT".to_string(),
        productions: vec![
            prod![
                t!("For"),
                t!("LParen"),
                nt!("DTYPE"), t!("Identifier"),   // loop variable declaration
                t!("Comma"), nt!("EXPR"),           // start
                t!("Comma"), nt!("EXPR"),           // end/limit
                t!("Comma"), nt!("EXPR"),           // step
                t!("RParen"),
                nt!("BLK")
            ],
        ],
    });

    // -------------------------------------------------------------------------
    // WHILE -> while ( EXPR ) BLK
    // -------------------------------------------------------------------------
    map.insert("WHILESTMT".to_string(), NonTerminal {
        name: "WHILESTMT".to_string(),
        productions: vec![
            prod![t!("While"), t!("LParen"), nt!("EXPR"), t!("RParen"), nt!("BLK")],
        ],
    });

    // -------------------------------------------------------------------------
    // RETURN -> return EXPR
    // -------------------------------------------------------------------------
    map.insert("RETSTMT".to_string(), NonTerminal {
        name: "RETSTMT".to_string(),
        productions: vec![
            prod![t!("Return"), nt!("EXPR")],
        ],
    });

    // -------------------------------------------------------------------------
    // DECL -> DTYPE IDENT | DTYPE IDENT = EXPR
    // -------------------------------------------------------------------------
    map.insert("DECL".to_string(), NonTerminal {
        name: "DECL".to_string(),
        productions: vec![
            prod![nt!("DTYPE"), t!("Identifier"), t!("Equals"), nt!("EXPR")],
            prod![nt!("DTYPE"), t!("Identifier")],
        ],
    });

    // -------------------------------------------------------------------------
    // ASSIGN -> LVALUE ASSIGNOP EXPR
    // -------------------------------------------------------------------------
    map.insert("ASSIGN".to_string(), NonTerminal {
        name: "ASSIGN".to_string(),
        productions: vec![
            prod![nt!("LVALUE"), nt!("ASSIGNOP"), nt!("EXPR")],
        ],
    });

    // LVALUE -> IDENT | IDENT . IDENT | IDENT . IDENT . IDENT ...
    // modelled as IDENT LVALUETAIL
    map.insert("LVALUE".to_string(), NonTerminal {
        name: "LVALUE".to_string(),
        productions: vec![
            prod![t!("Identifier"), nt!("LVALUETAIL")],
        ],
    });

    // LVALUETAIL -> . IDENT LVALUETAIL | epsilon
    map.insert("LVALUETAIL".to_string(), NonTerminal {
        name: "LVALUETAIL".to_string(),
        productions: vec![
            prod![t!("Dot"), t!("Identifier"), nt!("LVALUETAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // ASSIGNOP
    map.insert("ASSIGNOP".to_string(), NonTerminal {
        name: "ASSIGNOP".to_string(),
        productions: vec![
            prod![t!("Equals")],
            prod![t!("PlusEquals")],
            prod![t!("MinusEquals")],
            prod![t!("StarEquals")],
            prod![t!("SlashEquals")],
            prod![t!("PercentEquals")],
            prod![t!("AmpersandEquals")],
            prod![t!("PipeEquals")],
            prod![t!("CaretEquals")],
        ],
    });

    // -------------------------------------------------------------------------
    // DTYPE
    // -------------------------------------------------------------------------
    map.insert("DTYPE".to_string(), NonTerminal {
        name: "DTYPE".to_string(),
        productions: vec![
            prod![t!("TypeInt")],
            prod![t!("TypeFloat")],
            prod![t!("TypeChar")],
            prod![t!("TypeBoolean")],
            prod![t!("TypeVoid")],
            // array<int, 5>
            prod![t!("TypeArray"), t!("Less"), t!("Identifier"), t!("Comma"), t!("SIntLit"), t!("Greater")],
            // list<int>
            prod![t!("TypeList"), t!("Less"), t!("Identifier"), t!("Greater")],
            // struct<NAME>
            prod![t!("TypeStruct"), t!("Less"), t!("Identifier"), t!("Greater")],
        ],
    });

    // -------------------------------------------------------------------------
    // PARAMS -> DTYPE IDENT PARAMSTAIL | epsilon
    // -------------------------------------------------------------------------
    map.insert("PARAMS".to_string(), NonTerminal {
        name: "PARAMS".to_string(),
        productions: vec![
            prod![nt!("DTYPE"), t!("Identifier"), nt!("PARAMSTAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // PARAMSTAIL -> , DTYPE IDENT PARAMSTAIL | epsilon
    map.insert("PARAMSTAIL".to_string(), NonTerminal {
        name: "PARAMSTAIL".to_string(),
        productions: vec![
            prod![t!("Comma"), nt!("DTYPE"), t!("Identifier"), nt!("PARAMSTAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // -------------------------------------------------------------------------
    // ARGS -> EXPR ARGSTAIL | epsilon
    // -------------------------------------------------------------------------
    map.insert("ARGS".to_string(), NonTerminal {
        name: "ARGS".to_string(),
        productions: vec![
            prod![nt!("EXPR"), nt!("ARGSTAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // ARGSTAIL -> , EXPR ARGSTAIL | epsilon
    map.insert("ARGSTAIL".to_string(), NonTerminal {
        name: "ARGSTAIL".to_string(),
        productions: vec![
            prod![t!("Comma"), nt!("EXPR"), nt!("ARGSTAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // -------------------------------------------------------------------------
    // EXPRESSION PRECEDENCE HIERARCHY
    // EXPR -> LOGOREXPR
    // -------------------------------------------------------------------------
    map.insert("EXPR".to_string(), NonTerminal {
        name: "EXPR".to_string(),
        productions: vec![
            prod![nt!("LOGOREXPR")],
        ],
    });

    // LOGOREXPR -> LOGANDEXPR LOGOREXPR_TAIL
    map.insert("LOGOREXPR".to_string(), NonTerminal {
        name: "LOGOREXPR".to_string(),
        productions: vec![
            prod![nt!("LOGANDEXPR"), nt!("LOGOREXPR_TAIL")],
        ],
    });
    // LOGOREXPR_TAIL -> || LOGANDEXPR LOGOREXPR_TAIL | epsilon
    map.insert("LOGOREXPR_TAIL".to_string(), NonTerminal {
        name: "LOGOREXPR_TAIL".to_string(),
        productions: vec![
            prod![t!("||"), nt!("LOGANDEXPR"), nt!("LOGOREXPR_TAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // LOGANDEXPR -> CMPEXPR LOGANDEXPR_TAIL
    map.insert("LOGANDEXPR".to_string(), NonTerminal {
        name: "LOGANDEXPR".to_string(),
        productions: vec![
            prod![nt!("CMPEXPR"), nt!("LOGANDEXPR_TAIL")],
        ],
    });
    // LOGANDEXPR_TAIL -> && CMPEXPR LOGANDEXPR_TAIL | epsilon
    map.insert("LOGANDEXPR_TAIL".to_string(), NonTerminal {
        name: "LOGANDEXPR_TAIL".to_string(),
        productions: vec![
            prod![t!("&&"), nt!("CMPEXPR"), nt!("LOGANDEXPR_TAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // CMPEXPR -> ADDEXPR CMPEXPR_TAIL
    map.insert("CMPEXPR".to_string(), NonTerminal {
        name: "CMPEXPR".to_string(),
        productions: vec![
            prod![nt!("ADDEXPR"), nt!("CMPEXPR_TAIL")],
        ],
    });
    // CMPEXPR_TAIL -> CMPOP ADDEXPR | epsilon
    map.insert("CMPEXPR_TAIL".to_string(), NonTerminal {
        name: "CMPEXPR_TAIL".to_string(),
        productions: vec![
            prod![nt!("CMPOP"), nt!("ADDEXPR")],
            prod![t!("EPSILON")],
        ],
    });

    // ADDEXPR -> MULEXPR ADDEXPR_TAIL
    map.insert("ADDEXPR".to_string(), NonTerminal {
        name: "ADDEXPR".to_string(),
        productions: vec![
            prod![nt!("MULEXPR"), nt!("ADDEXPR_TAIL")],
        ],
    });
    // ADDEXPR_TAIL -> + MULEXPR ADDEXPR_TAIL | - MULEXPR ADDEXPR_TAIL | epsilon
    map.insert("ADDEXPR_TAIL".to_string(), NonTerminal {
        name: "ADDEXPR_TAIL".to_string(),
        productions: vec![
            prod![t!("Plus"),  nt!("MULEXPR"), nt!("ADDEXPR_TAIL")],
            prod![t!("Minus"), nt!("MULEXPR"), nt!("ADDEXPR_TAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // MULEXPR -> UNARYEXPR MULEXPR_TAIL
    map.insert("MULEXPR".to_string(), NonTerminal {
        name: "MULEXPR".to_string(),
        productions: vec![
            prod![nt!("UNARYEXPR"), nt!("MULEXPR_TAIL")],
        ],
    });
    // MULEXPR_TAIL -> * UNARYEXPR MULEXPR_TAIL | / UNARYEXPR MULEXPR_TAIL | % UNARYEXPR MULEXPR_TAIL | epsilon
    map.insert("MULEXPR_TAIL".to_string(), NonTerminal {
        name: "MULEXPR_TAIL".to_string(),
        productions: vec![
            prod![t!("Star"),    nt!("UNARYEXPR"), nt!("MULEXPR_TAIL")],
            prod![t!("Slash"),   nt!("UNARYEXPR"), nt!("MULEXPR_TAIL")],
            prod![t!("Percent"), nt!("UNARYEXPR"), nt!("MULEXPR_TAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // UNARYEXPR -> UNOP UNARYEXPR | CASTEXPR
    map.insert("UNARYEXPR".to_string(), NonTerminal {
        name: "UNARYEXPR".to_string(),
        productions: vec![
            prod![nt!("UNOP"), nt!("UNARYEXPR")],
            prod![nt!("CASTEXPR")],
        ],
    });

    // CASTEXPR -> ( DTYPE ) UNARYEXPR | PRIMARY
    map.insert("CASTEXPR".to_string(), NonTerminal {
        name: "CASTEXPR".to_string(),
        productions: vec![
            prod![t!("LParen"), nt!("DTYPE"), t!("RParen"), nt!("UNARYEXPR")],
            prod![nt!("PRIMARY")],
        ],
    });

    // PRIMARY -> ATOM PRIMARY_TAIL
    map.insert("PRIMARY".to_string(), NonTerminal {
        name: "PRIMARY".to_string(),
        productions: vec![
            prod![nt!("ATOM"), nt!("PRIMARY_TAIL")],
        ],
    });

    // PRIMARY_TAIL -> . IDENT ( ARGS ) PRIMARY_TAIL   <- method call
    //              | . IDENT PRIMARY_TAIL              <- member access
    //              | epsilon
    map.insert("PRIMARY_TAIL".to_string(), NonTerminal {
        name: "PRIMARY_TAIL".to_string(),
        productions: vec![
            prod![t!("Dot"), t!("Identifier"), t!("LParen"), nt!("ARGS"), t!("RParen"), nt!("PRIMARY_TAIL")],
            prod![t!("Dot"), t!("Identifier"), nt!("PRIMARY_TAIL")],
            prod![t!("EPSILON")],
        ],
    });

    // ATOM -> ( EXPR )
    //       | IDENT ( ARGS )      <- function call
    //       | IDENT               <- variable
    //       | SIntLit | FloatLit | StringLit | CharLit | BoolLit | Null
    map.insert("ATOM".to_string(), NonTerminal {
        name: "ATOM".to_string(),
        productions: vec![
            prod![t!("LParen"), nt!("EXPR"), t!("RParen")],
            prod![t!("Identifier"), t!("LParen"), nt!("ARGS"), t!("RParen")],
            prod![t!("Identifier")],
            prod![t!("SIntLit")],
            prod![t!("FloatLit")],
            prod![t!("StringLit")],
            prod![t!("CharLit")],
            prod![t!("BoolLit")],
            prod![t!("Null")],
        ],
    });

    // -------------------------------------------------------------------------
    // CMPOP
    // -------------------------------------------------------------------------
    map.insert("CMPOP".to_string(), NonTerminal {
        name: "CMPOP".to_string(),
        productions: vec![
            prod![t!("Greater")],
            prod![t!("Less")],
            prod![t!("GreaterEquals")],
            prod![t!("LessEquals")],
            prod![t!("EqualsEquals")],
            prod![t!("TildeEquals")],
        ],
    });

    // -------------------------------------------------------------------------
    // UNOP
    // -------------------------------------------------------------------------
    map.insert("UNOP".to_string(), NonTerminal {
        name: "UNOP".to_string(),
        productions: vec![
            prod![t!("Minus")],
            prod![t!("Tilde")],
            prod![t!("Ampersand")],
        ],
    });

    return map;
}
fn expand_nt(node: &mut Node, tokens_struct: &mut Tokens, productions: HashMap<String, NonTerminal>) {
    let token = tokens_struct.tokens.get(tokens_struct.index).unwrap();
    let rhs = productions.get(&node.name);
   
    for production in rhs.unwrap().productions.iter() {
        if production.len() > 0 {
            let first_symbol = &production[0];
            
           
            for sym in production.iter()  {
                match sym {
                    Symbol::Terminal(t) => {   
                        let res = match_symbols(sym, tokens_struct, productions.clone(), node, production.clone());
                        if !res {
                            delete_branch(node, tokens_struct, productions.clone());
                            return;// stop trying this production
                        }
                        }
                    Symbol::NonTerminal(nt) => {
                        let mut child_node = Node {
                            name: nt.clone(),
                            children: Vec::new()
                        };
                        expand_nt(&mut child_node, tokens_struct, productions.clone());
                        node.children.push(child_node);
                },
        
            }

        }
        }
    }

}
fn delete_branch(node: &mut Node, tokens_struct: &mut Tokens, productions: HashMap<String, NonTerminal>) {
    for child in node.children.iter_mut() {
        if child.children.len() == 0 {
            tokens_struct.index -= 1; // backtrack token consumption
        } else {
            delete_branch(child, tokens_struct, productions.clone());
        }
    }
    
}
fn iterate_productions(production: Vec<Symbol>, parent: &mut Node, tokens_struct: &mut Tokens, productions: HashMap<String, NonTerminal>) {
    for sym in production.iter()  {
        match sym {
            Symbol::Terminal(t) => {   
                let res = match_symbols(sym, tokens_struct, productions.clone(), parent, production.clone());
                if !res {
        
                    delete_branch(parent, tokens_struct, productions.clone());
                    break; // stop trying this production, move to next one

                }
            }
            Symbol::NonTerminal(nt) => {
                let mut child_node = Node {
                    name: nt.clone(),
                    children: Vec::new()
                };
                expand_nt(&mut child_node, tokens_struct, productions.clone());
                parent.children.push(child_node);
        },
    }
}}
fn match_symbols(symbol: &Symbol, tokens_struct: &mut Tokens, productions: HashMap<String, NonTerminal>, parent: &mut Node, production: Vec<Symbol>) -> bool {
    let token = tokens_struct.tokens.get(tokens_struct.index).unwrap();
    match symbol {
        Symbol::Terminal(term) => {
      
        if term == &format!("{:?}", token.token_type) {
           
            tokens_struct.index += 1; // consume the token
            parent.children.push(Node {
                name: term.clone(),
                children: Vec::new(),

            });
            return true;
         

        } else {
          
            return false; // terminal did not match
        }
        }
        Symbol:: NonTerminal(nt)=>{
            println!("Unexpected non-terminal {} when trying to match terminal in production {:?} for parent {}", nt, production, parent.name);
            return false;
        }
    }
}
fn dfs(parent: &mut Node, depth: usize, tokens_struct: &mut Tokens, productions:HashMap<String, NonTerminal>){
   
    let token = tokens_struct.tokens.get(tokens_struct.index).unwrap();
    let rhs = productions.get(&parent.name);
    iterate_productions(rhs.unwrap().productions[0].clone(), parent, tokens_struct, productions.clone());
}
pub fn create_tree(lexer_output: Vec<Token>) -> Node {
    let mut tokens_struct = Tokens {
        tokens: lexer_output,
        index: 0,
    };
    let productions = create_productions();

    let mut root = Node {
        name: "PRGM".to_string(),
        children: Vec::new()
    };

    let mut current_node = &mut root;
    
    dfs(current_node, 0, &mut tokens_struct, productions.clone());
    
    return root;
}
