PRGM -> BLK

BLK -> { STMTS }
     | STMTS

STMTS -> STMT ; STMTS
       | STMT ;

STMT -> IDENT ( ARGS )
      | IF ( EXPR ) BLK ELSEPART
      | DECL
      | DTYPE IDENT ( PARAMS ) BLK
      | FOR ( DECL , EXPR , EXPR ) BLK
      | WHILE ( EXPR ) BLK
      | BRK
      | CONT
      | RET EXPR
      | ASSIGN

ELSEPART -> ELSE BLK
          | EPSILON

ASSIGN -> IDENT = EXPR


DECL -> DTYPE IDENT
      | DTYPE IDENT = EXPR

ARGS -> EXPR , ARGS
      | EXPR
      | EPSILON

PARAMS -> DECL , PARAMS
        | DECL
        | EPSILON

EXPR -> ADDEXPR

ADDEXPR -> MULEXPR ( ( + | - ) MULEXPR )*

MULEXPR -> UNARYEXPR ( ( * | / | % ) UNARYEXPR )*

UNARYEXPR -> UNOP UNARYEXPR
           | CASTEXPR

CASTEXPR -> ( DTYPE ) UNARYEXPR
          | PRIMARY

PRIMARY -> ( EXPR )
         | IDENT ( ARGS )
         | IDENT
         | LITR




PRGM   -> Start ITEM* End

ITEM   -> MODULE
        | STRUCTDEF
        | FUNCDEF EndL?
        | STMT EndL

MODULE -> ModuleStart MODITEM* ModuleEnd

MODITEM -> MODFUNCDEF | MODDECL | MODSTMT

MODFUNCDEF -> Identifier("pub")? Identifier("fn") IDENT 
              ( PARAMS ) Arrow RETTYPE BLK

MODDECL  -> Identifier("let") IDENT DTYPE Equals EXPR EndL

MODSTMT  -> STMT EndL

FUNCDEF  -> Func IDENT ( PARAMS ) Arrow RETTYPE BLK

RETTYPE  -> DTYPE | TypeVoid | NoMatch     ← NoMatch as fallback until you add void

STRUCTDEF -> TypeStruct Less IDENT Greater LBrace FIELDS RBrace

FIELDS   -> FIELD EndL FIELDS | EPSILON
FIELD    -> DTYPE IDENT

BLK      -> LBrace STMTS RBrace

STMTS    -> STMT EndL STMTS | EPSILON

STMT     -> DECL
          | ASSIGN
          | EXPR                            ← handles standalone calls
          | If ( EXPR ) BLK ELSEPART
          | For ( DTYPE IDENT , EXPR , EXPR , EXPR ) BLK
          | While ( EXPR ) BLK
          | Return EXPR
          | Break
          | Continue

ELSEPART -> Else BLK | EPSILON

DECL  → DTYPE IDENT DECL'
DECL' → Equals EXPR | ε

ASSIGN   -> LVALUE ASSIGNOP EXPR
LVALUE   -> IDENT ( Dot IDENT )*

ASSIGNOP -> Equals | PlusEquals | MinusEquals | StarEquals
          | SlashEquals | PercentEquals
          | AmpersandEquals | PipeEquals | CaretEquals

DTYPE    -> TypeInt | TypeFloat | TypeChar | TypeBoolean
          | TypeArray Less IDENT Comma SIntLit Greater
          | TypeList  Less IDENT Greater
          | TypeStruct Less IDENT Greater

PARAMS   -> DTYPE IDENT ( Comma DTYPE IDENT )* | EPSILON
ARGS     -> EXPR ( Comma EXPR )* | EPSILON

EXPR        -> LOGOREXPR
LOGOREXPR   -> LOGANDEXPR  ( || LOGANDEXPR  )*
LOGANDEXPR  -> CMPEXPR     ( && CMPEXPR     )*
CMPEXPR     -> ADDEXPR     ( CMPOP ADDEXPR  )?
ADDEXPR     -> MULEXPR     ( ( Plus | Minus ) MULEXPR    )*
MULEXPR     -> UNARYEXPR   ( ( Star | Slash | Percent ) UNARYEXPR )*
UNARYEXPR   -> UNOP UNARYEXPR | CASTEXPR
CASTEXPR    -> ( DTYPE ) UNARYEXPR | PRIMARY
PRIMARY     -> ATOM ( Dot IDENT ( LParen ARGS RParen )? )*
ATOM        -> LParen EXPR RParen
             | IDENT LParen ARGS RParen
             | IDENT
             | SIntLit | FloatLit
             | StringLit | CharLit | BoolLit | Null

CMPOP  -> Greater | Less | GreaterEquals | LessEquals
        | EqualsEquals | TildeEquals
UNOP   -> Minus | Tilde | Ampersand