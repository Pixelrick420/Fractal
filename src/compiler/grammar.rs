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
    let epsilon = "EPSILON".to_string();

    // initialize
    for nt in grammar.keys() {
        first.insert(nt.clone(), HashSet::new());
    }

    let mut changed = true;

    while changed {
        changed = false;

        for (lhs, productions) in grammar {
            for production in productions {

                let mut all_nullable = true;

                // Special case: A → EPSILON
                if production.len() == 1 && production[0] == epsilon {
                    if first.get_mut(lhs).unwrap().insert(epsilon.clone()) {
                        changed = true;
                    }
                    continue;
                }

                for symbol in production {

                    // Terminal (and not EPSILON)
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
    grammar: &HashMap<String, Vec<Vec<String>>>,
    first: &HashMap<String, HashSet<String>>,
    start_symbol: &String,
) -> HashMap<String, HashSet<String>> {

    let mut follow: HashMap<String, HashSet<String>> = HashMap::new();
    let epsilon = "EPSILON".to_string();

    for nt in grammar.keys() {
        follow.insert(nt.clone(), HashSet::new());
    }

    // Add $
    follow.get_mut(start_symbol).unwrap().insert("$".to_string());

    let mut changed = true;

    while changed {
        changed = false;

        for (lhs, productions) in grammar {
            for production in productions {

                for i in 0..production.len() {
                    let symbol = &production[i];

                    if !grammar.contains_key(symbol) {
                        continue;
                    }

                    let mut beta_nullable = true;
                    let mut first_of_beta = HashSet::new();

                    for next_symbol in production.iter().skip(i + 1) {

                        if !grammar.contains_key(next_symbol) {
                            first_of_beta.insert(next_symbol.clone());
                            beta_nullable = false;
                            break;
                        }

                        let next_first = first.get(next_symbol).unwrap();

                        for f in next_first {
                            if f != &epsilon {
                                first_of_beta.insert(f.clone());
                            }
                        }

                        if !next_first.contains(&epsilon) {
                            beta_nullable = false;
                            break;
                        }
                    }

                    // Add FIRST(β) - {ε}
                    for f in &first_of_beta {
                        if follow.get_mut(symbol).unwrap().insert(f.clone()) {
                            changed = true;
                        }
                    }

                    // If β nullable → add FOLLOW(lhs)
                    if beta_nullable {
                        let lhs_follow = follow.get(lhs).unwrap().clone();
                        for f in lhs_follow {
                            if f != epsilon {
                                if follow.get_mut(symbol).unwrap().insert(f) {
                                    changed = true;
                                }
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

STRUCTDEF   -> StructKeyword Less IDENT Greater LBrace FIELDS RBrace

FIELDS      -> FIELD EndL FIELDS
             | ε

FIELD       -> DTYPE IDENT

BLK         -> LBrace STMTS RBrace

STMTS       -> STMT EndL STMTS
             | ε


STMT        -> DECL
    
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

EXPR        -> LOGOR EXPR_TAIL
EXPR_TAIL   -> ASSIGNOP EXPR
             | ε

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
    let follow = compute_follow(
    &grammar.productions,
    &first,
    &grammar.start_symbol,
);

    println!("FIRST:");
    for (k, v) in &first {
        println!("{}: {:?}", k, v);
    }

    println!("\nFOLLOW:");
    for (k, v) in &follow {
        println!("{}: {:?}", k, v);
    }
    detect_ll1_conflicts(&grammar, &first, &follow);
    write_first_follow_to_file(&first, &follow, "first_follow_saved.rs");
    generate_procedures(&grammar.productions, "generated_parser.rs");

}


fn compute_first_of_production(
    production: &Vec<String>,
    grammar: &Grammar,
    first: &HashMap<String, HashSet<String>>,
) -> (HashSet<String>, bool) {
    let mut result = HashSet::new();
    let epsilon = "EPSILON".to_string();
    let mut nullable = true;

    for symbol in production {
        if !grammar.productions.contains_key(symbol) {
            result.insert(symbol.clone());
            nullable = false;
            return (result, false);
        }

        let sym_first = first.get(symbol).unwrap();

        for f in sym_first {
            if f != &epsilon {
                result.insert(f.clone());
            }
        }

        if !sym_first.contains(&epsilon) {
            nullable = false;
            return (result, false);
        }
    }

    (result, nullable)
}
fn detect_ll1_conflicts(
    grammar: &Grammar,
    first: &HashMap<String, HashSet<String>>,
    follow: &HashMap<String, HashSet<String>>,
) {
    println!("\n=== LL(1) Conflict Detection ===");

    for (lhs, productions) in &grammar.productions {

        for i in 0..productions.len() {
            for j in i + 1..productions.len() {

                let (first_i, nullable_i) =
                    compute_first_of_production(&productions[i], grammar, first);

                let (first_j, nullable_j) =
                    compute_first_of_production(&productions[j], grammar, first);

                // FIRST/FIRST conflict
                let intersection: HashSet<_> =
                    first_i.intersection(&first_j).cloned().collect();
                    let epsilon = "EPSILON".to_string();

                let first_i_no_eps: HashSet<_> =
                    first_i.iter().filter(|x| *x != &epsilon).cloned().collect();

                let first_j_no_eps: HashSet<_> =
                    first_j.iter().filter(|x| *x != &epsilon).cloned().collect();

                let intersection: HashSet<_> =
                    first_i_no_eps.intersection(&first_j_no_eps).cloned().collect();

                if !intersection.is_empty() {
                    println!(
                        "FIRST/FIRST conflict in {} between productions {:?} and {:?}",
                        lhs, productions[i], productions[j]
                    );
                    println!("Conflict symbols: {:?}\n", intersection);
                }

                // FIRST/FOLLOW conflict (nullable)
                if nullable_i {
                    let follow_lhs = follow.get(lhs).unwrap();
                    let conflict: HashSet<_> =
                        first_j.intersection(follow_lhs).cloned().collect();

                    if !conflict.is_empty() {
                        println!(
                            "FIRST/FOLLOW conflict in {} (production {:?})",
                            lhs, productions[i]
                        );
                        println!("Conflict symbols: {:?}\n", conflict);
                    }
                }

                if nullable_j {
                    let follow_lhs = follow.get(lhs).unwrap();
                    let conflict: HashSet<_> =
                        first_i.intersection(follow_lhs).cloned().collect();

                    if !conflict.is_empty() {
                        println!(
                            "FIRST/FOLLOW conflict in {} (production {:?})",
                            lhs, productions[j]
                        );
                        println!("Conflict symbols: {:?}\n", conflict);
                    }
                }
            }
        }
    }

    println!("=== Detection Finished ===\n");
    
}


fn generate_procedures(
    grammar: &HashMap<String, Vec<Vec<String>>>,
    output_file: &str,
) {
    let mut file = File::create(output_file)
        .expect("Unable to create file");

    for nt in grammar.keys() {

        let function = format!(
            "pub fn parse_{}(&mut self) {{\n    // TODO: implement {}\n}}\n\n",
            nt.to_lowercase(),
            nt
        );

        file.write_all(function.as_bytes())
            .expect("Write failed");
    }

    println!("Procedures saved to {}", output_file);
}