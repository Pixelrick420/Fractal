```
PRGM      -> Start ITEM_LIST End

ITEM_LIST -> ITEM ITEM_LIST
           | ε

ITEM -> MODULE
      | FUNCDEF
      | STRUCTDEF_OR_DECL EndL
      | STMT EndL

MODULE -> ModuleStart ITEM_LIST ModuleEnd EndL?

FUNCDEF -> Func Identifier LParen PARAMS RParen Arrow DATATYPE BLK

PARAMS      -> PARAM PARAMS_TAIL
             | ε
PARAMS_TAIL -> Comma PARAM PARAMS_TAIL
             | ε
PARAM       -> DATATYPE Identifier

STRUCTDEF_OR_DECL -> TypeStruct Less Identifier Greater STRUCT_TAIL
STRUCT_TAIL       -> LBrace FIELDS RBrace
                   | Identifier STRUCTDECL_TAIL
STRUCTDECL_TAIL   -> Equals EXPRESSION
                   | ε
FIELDS -> FIELD FIELDS
        | ε
FIELD  -> DATATYPE Identifier EndL

BLK   -> LBrace STMTS RBrace
STMTS -> STMT EndL? STMTS
       | ε

STMT -> DECL
      | ASSIGN
      | STRUCTDEF_OR_DECL
      | If LParen EXPRESSION RParen BLK ELSEPART
      | For LParen DATATYPE Identifier Comma EXPRESSION Comma EXPRESSION Comma EXPRESSION RParen BLK
      | While LParen EXPRESSION RParen BLK
      | Return EXPRESSION
      | Exit EXPRESSION
      | Break
      | Continue
      | EXPRESSION

ELSEPART -> Elif LParen EXPRESSION RParen BLK ELSEPART
          | Else BLK
          | ε

DECL      -> DATATYPE Identifier DECL_TAIL
DECL_TAIL -> Equals EXPRESSION
           | ε

ASSIGN      -> LVALUE ASSIGNOP EXPRESSION
LVALUE      -> Identifier LVALUE_TAIL
LVALUE_TAIL -> ColonColon Identifier
             | ε

ASSIGNOP -> Equals | PlusEquals | MinusEquals | StarEquals | SlashEquals
          | PercentEquals | AmpersandEquals | PipeEquals | CaretEquals

DATATYPE -> TypeInt
          | TypeFloat
          | TypeChar
          | TypeBoolean
          | TypeArray Less Identifier Comma SIntLit Greater
          | TypeList Less Identifier Greater
          | TypeStruct Less Identifier Greater

EXPRESSION -> LOGOR

LOGOR       -> LOGAND LOGOR_TAIL
LOGOR_TAIL  -> Or LOGAND LOGOR_TAIL
             | ε

LOGAND      -> LOGNOT LOGAND_TAIL
LOGAND_TAIL -> And LOGNOT LOGAND_TAIL
             | ε

LOGNOT -> Not LOGNOT
        | CMP

CMP   -> BITOR CMPOP BITOR
       | BITOR
CMPOP -> Greater | Less | GreaterEquals | LessEquals | EqualsEquals | TildeEquals

BITOR       -> BITXOR BITOR_TAIL
BITOR_TAIL  -> Pipe BITXOR BITOR_TAIL
             | ε

BITXOR      -> BITAND BITXOR_TAIL
BITXOR_TAIL -> Caret BITAND BITXOR_TAIL
             | ε

BITAND      -> ADD BITAND_TAIL
BITAND_TAIL -> Ampersand ADD BITAND_TAIL
             | ε

ADD      -> MUL ADD_TAIL
ADD_TAIL -> Plus  MUL ADD_TAIL
          | Minus MUL ADD_TAIL
          | ε

MUL      -> UNARY MUL_TAIL
MUL_TAIL -> Star    UNARY MUL_TAIL
          | Slash   UNARY MUL_TAIL
          | Percent UNARY MUL_TAIL
          | ε

UNARY -> Minus UNARY
       | Tilde UNARY
       | CAST
       | PRIMARY

CAST -> DATATYPE LParen EXPRESSION RParen

PRIMARY -> LParen EXPRESSION RParen
         | LBracket ARGS RBracket
         | LBrace STRUCT_LIT_FIELDS RBrace
         | Identifier PRIMARY_TAIL
         | SIntLit
         | FloatLit
         | CharLit
         | StringLit
         | BoolLit
         | Null

PRIMARY_TAIL   -> ColonColon Identifier QUALIFIED_TAIL
               | LParen ARGS RParen
               | ε
QUALIFIED_TAIL -> LParen ARGS RParen
               | ε

STRUCT_LIT_FIELDS -> Identifier Equals EXPRESSION STRUCT_LIT_TAIL
                   | ε
STRUCT_LIT_TAIL   -> Comma Identifier Equals EXPRESSION STRUCT_LIT_TAIL
                   | ε

ARGS      -> EXPRESSION ARGS_TAIL
           | ε
ARGS_TAIL -> Comma EXPRESSION ARGS_TAIL
           | ε
```

| Token      | Lexeme       |
|------------|--------------|
| `Start`    | `!start`     |
| `End`      | `!end`       |
| `Func`     | `!func`      |
| `If`       | `!if`        |
| `Elif`     | `!elif`      |
| `Else`     | `!else`      |
| `For`      | `!for`       |
| `While`    | `!while`     |
| `Return`   | `!return`    |
| `Exit`     | `!exit`      |
| `Break`    | `!break`     |
| `Continue` | `!continue`  |
| `And`      | `!and`       |
| `Or`       | `!or`        |
| `Not`      | `!not`       |
| `Struct`   | `!struct`    |
| `Import`   | `!import`    |
| `Module`   | `!module`    |
| `TypeInt`     | `:int`      |
| `TypeFloat`   | `:float`    |
| `TypeChar`    | `:char`     |
| `TypeBoolean` | `:boolean`  |
| `TypeArray`   | `:array`    |
| `TypeList`    | `:list`     |
| `TypeStruct`  | `:struct`   |
| `SIntLit(i64)`     | decimal `42`, hex `0xFF`, binary `0b1010`, octal `0o77`, prefixed decimal `0d42` |
| `FloatLit(f64)`    | `3.14`, `2.0e-5`                                   |
| `CharLit(char)`    | `'a'`, `'\n'`                                      |
| `StringLit(String)`| `"hello\n"`                                        |
| `BoolLit(bool)`    | `true`, `false`                                    |
| `Null`             | `NULL`                                             |
| `Plus`             | `+`    |
| `Minus`            | `-`    |
| `Star`             | `*`    |
| `Slash`            | `/`    |
| `Percent`          | `%`    |
| `Ampersand`        | `&`    |
| `Pipe`             | `\|`   |
| `Caret`            | `^`    |
| `Tilde`            | `~`    |
| `Equals`           | `=`    |
| `PlusEquals`       | `+=`   |
| `MinusEquals`      | `-=`   |
| `StarEquals`       | `*=`   |
| `SlashEquals`      | `/=`   |
| `PercentEquals`    | `%=`   |
| `AmpersandEquals`  | `&=`   |
| `PipeEquals`       | `\|=`  |
| `CaretEquals`      | `^=`   |
| `Greater`          | `>`    |
| `Less`             | `<`    |
| `GreaterEquals`    | `>=`   |
| `LessEquals`       | `<=`   |
| `EqualsEquals`     | `==`   |
| `TildeEquals`      | `~=`   |
| `Arrow`            | `->`   |
| `Dot`              | `.`    |
| `Comma`            | `,`    |
| `ColonColon`       | `::`   |
| `LParen`           | `(`    |
| `RParen`           | `)`    |
| `LBrace`           | `{`    |
| `RBrace`           | `}`    |
| `LBracket`         | `[`    |
| `RBracket`         | `]`    |
| `EndL`             | `;`    |
| `ModuleStart(name)`| `$MODULE_START:n$` |
| `ModuleEnd(name)`  | `$MODULE_END:n$`   |
