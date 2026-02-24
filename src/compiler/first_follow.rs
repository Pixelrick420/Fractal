use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
type Symbol = String;

#[derive(Debug)]
struct Grammar {
    productions: HashMap<Symbol, Vec<Vec<Symbol>>>, 
    start_symbol: Symbol,
}
fn parse_grammar(input: &str) -> Grammar {
    let mut productions: HashMap<Symbol, Vec<Vec<Symbol>>> = HashMap::new();
    let mut current_lhs = String::new();
    let mut start_symbol = String::new();

    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        if line.contains("->") {
            let parts: Vec<&str> = line.split("->").collect();
            current_lhs = parts[0].trim().to_string();
            if start_symbol.is_empty() {
                start_symbol = current_lhs.clone();
            }

            let rhs = parts[1].trim();
            let alternatives = rhs.split('|');

            for alt in alternatives {
                let symbols: Vec<Symbol> =
                    alt.trim().split_whitespace().map(|s| s.to_string()).collect();
                productions
                    .entry(current_lhs.clone())
                    .or_default()
                    .push(symbols);
            }
        } else if line.starts_with('|') {
            let rhs = line[1..].trim();
            let symbols: Vec<Symbol> =
                rhs.split_whitespace().map(|s| s.to_string()).collect();

            productions
                .entry(current_lhs.clone())
                .or_default()
                .push(symbols);
        }
    }

    Grammar {
        productions,
        start_symbol,
    }
}
fn is_non_terminal(symbol: &str, grammar: &Grammar) -> bool {
    grammar.productions.contains_key(symbol)
}

fn compute_first(
    grammar: &HashMap<String, Vec<Vec<String>>>,
) -> HashMap<String, HashSet<String>> {

    let mut first: HashMap<String, HashSet<String>> = HashMap::new();

    // Initialize empty sets
    for nt in grammar.keys() {
        first.insert(nt.clone(), HashSet::new());
    }

    let epsilon = "EPSILON".to_string();
    let mut changed = true;

    while changed {
        changed = false;

        for (lhs, productions) in grammar {
            for production in productions {

                let mut all_nullable = true;

                for symbol in production {

                    // Terminal
                    if !grammar.contains_key(symbol) {
                        if first.get_mut(lhs).unwrap().insert(symbol.clone()) {
                            changed = true;
                        }
                        all_nullable = false;
                        break;
                    }

                    // Non-terminal
                    let sym_first = first.get(symbol).unwrap().clone();

                    for f in &sym_first {
                        if f != &epsilon {
                            if first.get_mut(lhs).unwrap().insert(f.clone()) {
                                changed = true;
                            }
                        }
                    }

                    if !sym_first.contains(&epsilon) {
                        all_nullable = false;
                        break;
                    }
                }

                if all_nullable {
                    if first.get_mut(lhs).unwrap().insert(epsilon.clone()) {
                        changed = true;
                    }
                }
            }
        }
    }

    first
}

fn compute_follow(
    grammar: &Grammar,
    first: &HashMap<Symbol, HashSet<Symbol>>,
) -> HashMap<Symbol, HashSet<Symbol>> {

    let mut follow: HashMap<Symbol, HashSet<Symbol>> = HashMap::new();

    for nt in grammar.productions.keys() {
        follow.insert(nt.clone(), HashSet::new());
    }

    // Add $ to start symbol
    follow
        .get_mut(&grammar.start_symbol)
        .unwrap()
        .insert("$".to_string());

    let epsilon = "EPSILON".to_string();
    let mut changed = true;

    while changed {
        changed = false;

        for (lhs, alternatives) in &grammar.productions {
            for production in alternatives {
                for i in 0..production.len() {
                    let symbol = &production[i];

                    if !is_non_terminal(symbol, grammar) {
                        continue;
                    }

                    let mut first_of_beta: HashSet<Symbol> = HashSet::new();
                    let mut beta_can_be_empty = true;

                    for next_symbol in production.iter().skip(i + 1) {
                        if !is_non_terminal(next_symbol, grammar) {
                            first_of_beta.insert(next_symbol.clone());
                            beta_can_be_empty = false;
                            break;
                        } else {
                            let next_first = first.get(next_symbol).unwrap();

                            for s in next_first {
                                if s != &epsilon {
                                    first_of_beta.insert(s.clone());
                                }
                            }

                            if !next_first.contains(&epsilon) {
                                beta_can_be_empty = false;
                                break;
                            }
                        }
                    }

                    // Add FIRST(β) - {ε}
                    for s in first_of_beta {
                        if follow.get_mut(symbol).unwrap().insert(s) {
                            changed = true;
                        }
                    }

                    // If β ⇒* ε
                    if beta_can_be_empty {
                        let lhs_follow = follow.get(lhs).unwrap().clone();
                        for s in lhs_follow {
                            if follow.get_mut(symbol).unwrap().insert(s) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    follow
}

fn write_first_follow_to_file(
    first: &std::collections::HashMap<String, std::collections::HashSet<String>>,
    follow: &std::collections::HashMap<String, std::collections::HashSet<String>>,
    path: &str,
) {
    let mut file = File::create(path).expect("Unable to create file");

    writeln!(file, "use std::collections::{{HashMap, HashSet}};").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "pub fn get_first() -> HashMap<String, HashSet<String>> {{").unwrap();
    writeln!(file, "    let mut first = HashMap::new();").unwrap();

    for (nt, set) in first {
        writeln!(
            file,
            "    first.insert(\"{}\".to_string(), vec!{:?}.into_iter().map(|s| s.to_string()).collect());",
            nt,
            set.iter().collect::<Vec<_>>()
        ).unwrap();
    }

    writeln!(file, "    first").unwrap();
    writeln!(file, "}}").unwrap();
    writeln!(file).unwrap();

    writeln!(file, "pub fn get_follow() -> HashMap<String, HashSet<String>> {{").unwrap();
    writeln!(file, "    let mut follow = HashMap::new();").unwrap();

    for (nt, set) in follow {
        writeln!(
            file,
            "    follow.insert(\"{}\".to_string(), vec!{:?}.into_iter().map(|s| s.to_string()).collect());",
            nt,
            set.iter().collect::<Vec<_>>()
        ).unwrap();
    }

    writeln!(file, "    follow").unwrap();
    writeln!(file, "}}").unwrap();
}
fn main() {
    
    let productions = r#"
PRGM        -> Start ITEM_LIST End

ITEM_LIST   -> ITEM ITEM_LIST
             | ε

ITEM        -> MODULE
             | STRUCTDEF
             | FUNCDEF FUNC_END
             | STMT EndL

FUNC_END    -> EndL
             | ε

MODULE      -> ModuleStart MODITEM_LIST ModuleEnd

MODITEM_LIST -> MODITEM MODITEM_LIST
              | ε

MODITEM     -> MODFUNCDEF
             | MODDECL
             | MODSTMT

MODFUNCDEF  -> PUBOPT Fn IDENT LParen PARAMS RParen Arrow RETTYPE BLK

PUBOPT      -> Pub
             | ε

MODDECL     -> Let IDENT DTYPE Equals EXPR EndL

MODSTMT     -> STMT EndL

FUNCDEF     -> Func IDENT LParen PARAMS RParen Arrow RETTYPE BLK

RETTYPE     -> DTYPE
             | TypeVoid

STRUCTDEF   -> TypeStruct Less IDENT Greater LBrace FIELDS RBrace

FIELDS      -> FIELD EndL FIELDS
             | ε

FIELD       -> DTYPE IDENT

BLK         -> LBrace STMTS RBrace

STMTS       -> STMT EndL STMTS
             | ε


STMT        -> DECL
             | ASSIGN
             | EXPR
             | If LParen EXPR RParen BLK ELSEPART
             | For LParen DTYPE IDENT Comma EXPR Comma EXPR Comma EXPR RParen BLK
             | While LParen EXPR RParen BLK
             | Return EXPR
             | Break
             | Continue

ELSEPART    -> Else BLK
             | ε

DECL        -> DTYPE IDENT DECLTAIL

DECLTAIL    -> Equals EXPR
             | ε

ASSIGN      -> LVALUE ASSIGNOP EXPR

LVALUE      -> IDENT LVALUE_TAIL

LVALUE_TAIL -> Dot IDENT LVALUE_TAIL
             | ε

ASSIGNOP    -> Equals
             | PlusEquals
             | MinusEquals
             | StarEquals
             | SlashEquals
             | PercentEquals
             | AmpersandEquals
             | PipeEquals
             | CaretEquals

DTYPE       -> TypeInt
             | TypeFloat
             | TypeChar
             | TypeBoolean
             | TypeArray Less IDENT Comma SIntLit Greater
             | TypeList  Less IDENT Greater
             | TypeStruct Less IDENT Greater

PARAMS      -> PARAM PARAMS_TAIL
             | ε

PARAM       -> DTYPE IDENT

PARAMS_TAIL -> Comma PARAM PARAMS_TAIL
             | ε

ARGS        -> EXPR ARGS_TAIL
             | ε

ARGS_TAIL   -> Comma EXPR ARGS_TAIL
             | ε

EXPR        -> LOGOR

LOGOR       -> LOGAND LOGOR_TAIL

LOGOR_TAIL  -> OrOr LOGAND LOGOR_TAIL
             | ε
LOGAND      -> CMP LOGAND_TAIL

LOGAND_TAIL -> AndAnd CMP LOGAND_TAIL
             | ε
CMP         -> ADD CMP_TAIL

CMP_TAIL    -> CMPOP ADD CMP_TAIL
             | ε
ADD         -> MUL ADD_TAIL

ADD_TAIL    -> Plus MUL ADD_TAIL
             | Minus MUL ADD_TAIL
             | ε
MUL         -> UNARY MUL_TAIL

MUL_TAIL    -> Star UNARY MUL_TAIL
             | Slash UNARY MUL_TAIL
             | Percent UNARY MUL_TAIL
             | ε
UNARY       -> UNOP UNARY
             | POSTFIX

POSTFIX     -> PRIMARY POSTFIX_TAIL
POSTFIX_TAIL -> Dot IDENT CALL_OPT POSTFIX_TAIL
              | As DTYPE POSTFIX_TAIL
              | ε

CALL_OPT    -> LParen ARGS RParen
             | ε
PRIMARY     -> LParen EXPR RParen
             | IDENT PRIMARY_TAIL
             | SIntLit
             | FloatLit
             | StringLit
             | CharLit
             | BoolLit
             | Null
PRIMARY_TAIL -> LParen ARGS RParen
              | ε

CMPOP       -> Greater
             | Less
             | GreaterEquals
             | LessEquals
             | EqualsEquals
             | TildeEquals

UNOP        -> Minus
             | Tilde
             | Ampersand

"#;



    let grammar = parse_grammar(productions);
    let first = compute_first(&grammar.productions);
    let follow = compute_follow(&grammar, &first);

    println!("FIRST:");
    for (k, v) in &first {
        println!("{}: {:?}", k, v);
    }

    println!("\nFOLLOW:");
    for (k, v) in &follow {
        println!("{}: {:?}", k, v);
    }
    write_first_follow_to_file(&first, &follow, "first_follow_saved.rs");
}

