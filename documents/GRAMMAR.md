```
# A complete program is a bracketed sequence of top-level items.
PRGM      -> Start ITEM_LIST End

# Zero or more top-level items, collected in order.
ITEM_LIST -> ITEM ITEM_LIST
           | ε

# A top-level item can be a module block, a function, a struct definition,
# a struct variable declaration, a plain variable declaration, or a statement.
ITEM -> MODULE
      | FUNCDEF
      | STRUCTDEF EndL
      | STRUCTDECL EndL
      | DECL EndL
      | STMT EndL

# A named module groups a list of items; the name must match on open and close.
MODULE -> ModuleStart ITEM_LIST ModuleEnd EndL?

# A function definition binds a name to a typed parameter list, a return type,
# and a body block. Functions may only appear at the top level.
FUNCDEF -> Func Identifier LParen PARAMS RParen Arrow DATATYPE BLK

# A parameter list is zero or more comma-separated typed parameters.
PARAMS      -> PARAM PARAMS_TAIL
             | ε
PARAMS_TAIL -> Comma PARAM PARAMS_TAIL
             | ε
# A single parameter is a type annotation followed by its local name.
PARAM       -> DATATYPE Identifier

# A struct definition introduces a new named struct type with a set of fields.
# The semicolon after } is required.
STRUCTDEF -> TypeStruct Less StructTypeName Greater LBrace FIELDS RBrace

# A struct declaration creates a variable of an existing struct type,
# with an optional initialiser expression.
STRUCTDECL -> TypeStruct Less StructTypeName Greater Identifier STRUCTDECL_TAIL
STRUCTDECL_TAIL -> Equals EXPRESSION
                 | ε

# A struct body is zero or more typed field declarations, each terminated by ;
FIELDS -> FIELD FIELDS
        | ε
# A single field is a type and a name - no default value is allowed.
FIELD  -> DATATYPE Identifier EndL

# A block is a scoped sequence of statements wrapped in braces.
BLK   -> LBrace STMTS RBrace
# Statements inside a block are separated by optional semicolons (block-level
# statements like !if/!for/!while don't need a trailing ;).
STMTS -> STMT EndL? STMTS
       | ε

# A statement is one of: variable declaration, struct variable declaration,
# assignment, control flow, or a bare expression (e.g. a function call).
STMT -> DECL
      | STRUCTDECL
      | ASSIGN
      | If LParen EXPRESSION RParen BLK ELSEPART        # conditional branch
      | For LParen DATATYPE Identifier Comma EXPRESSION Comma EXPRESSION Comma EXPRESSION RParen BLK
                                                        # counted loop, declares a new :int variable
      | For LParen Identifier Comma EXPRESSION Comma EXPRESSION Comma EXPRESSION RParen BLK
                                                        # counted loop, reuses an already-declared variable
      | While LParen EXPRESSION RParen BLK              # condition-controlled loop
      | Return EXPRESSION EndL                          # exit current function with a value
      | Exit EXPRESSION EndL                            # terminate the whole program with an exit code
      | Break EndL                                      # exit the nearest enclosing loop
      | Continue EndL                                   # skip to the next iteration of the nearest loop
      | EXPRESSION                                      # expression used for its side effects (e.g. a call)

# Note: !return and !exit consume their own EndL inside parse_stmt.
# Note: the second For form omits the type prefix and reuses a variable already in scope.

# The optional else-chain: zero or more !elif branches, then an optional !else.
# Each !elif is desugared into a nested If node, so the structure is recursive.
ELSEPART -> Elif LParen EXPRESSION RParen BLK ELSEPART
          | Else BLK
          | ε

# A variable declaration introduces a new name with a type; the initialiser
# is optional - omitting it gives a default value for simple types.
DECL      -> DATATYPE Identifier DECL_TAIL
DECL_TAIL -> Equals EXPRESSION
           | ε

# An assignment writes a new value into an existing lvalue.
# The target must be an ACCESS_CHAIN that does not end with a function call.
ASSIGN   -> ACCESS_CHAIN ASSIGNOP EXPRESSION

# All nine assignment operators: plain and compound (read-modify-write).
ASSIGNOP -> Equals | PlusEquals | MinusEquals | StarEquals | SlashEquals
          | PercentEquals | AmpersandEquals | PipeEquals | CaretEquals

# A type annotation - either a primitive, a sized array, a variable-length
# list, or a named struct type.
DATATYPE -> TypeInt
          | TypeFloat
          | TypeChar
          | TypeBoolean
          | TypeVoid
          | TypeArray Less DATATYPE Comma SIntLit Greater   # fixed-size array; size is a compile-time integer
          | TypeList Less DATATYPE Greater                  # variable-length list
          | TypeStruct Less StructTypeName Greater          # user-defined struct by name

# A struct type name is either a plain identifier or a module-qualified one.
StructTypeName -> Identifier
                | Identifier ColonColon Identifier

# The top of the expression hierarchy; all expressions bottom out here.
EXPRESSION -> LOGOR

# Logical OR: short-circuits; both operands must be :boolean.
LOGOR       -> LOGAND LOGOR_TAIL
LOGOR_TAIL  -> Or LOGAND LOGOR_TAIL
             | ε

# Logical AND: short-circuits; both operands must be :boolean.
LOGAND      -> LOGNOT LOGAND_TAIL
LOGAND_TAIL -> And LOGNOT LOGAND_TAIL
             | ε

# Logical NOT: prefix boolean negation; right-associative, may be chained.
LOGNOT -> Not LOGNOT
        | CMP

# A comparison produces a :boolean result. Only one comparison per expression
# is allowed (non-associative) - parentheses are required to chain them.
CMP   -> BITOR CMPOP BITOR
       | BITOR
CMPOP -> Greater | Less | GreaterEquals | LessEquals | EqualsEquals | TildeEquals

# Bitwise OR: integer-only, left-associative.
BITOR       -> BITXOR BITOR_TAIL
BITOR_TAIL  -> Pipe BITXOR BITOR_TAIL
             | ε

# Bitwise XOR: integer-only, left-associative.
BITXOR      -> BITAND BITXOR_TAIL
BITXOR_TAIL -> Caret BITAND BITXOR_TAIL
             | ε

# Bitwise AND: integer-only, left-associative.
BITAND      -> SHIFT BITAND_TAIL
BITAND_TAIL -> Ampersand SHIFT BITAND_TAIL
             | ε

# Bit shifts: integer-only, left-associative.
# << and >> are not single tokens; each is parsed as two consecutive < or > tokens.
SHIFT      -> ADD SHIFT_TAIL
SHIFT_TAIL -> Less Less ADD SHIFT_TAIL       # left shift  <<
            | Greater Greater ADD SHIFT_TAIL  # right shift >>
            | ε

# Additive arithmetic: left-associative, operands must be the same type.
ADD      -> MUL ADD_TAIL
ADD_TAIL -> Plus  MUL ADD_TAIL
          | Minus MUL ADD_TAIL
          | ε

# Multiplicative arithmetic: left-associative. % is integer-only.
MUL      -> UNARY MUL_TAIL
MUL_TAIL -> Star    UNARY MUL_TAIL
          | Slash   UNARY MUL_TAIL
          | Percent UNARY MUL_TAIL
          | ε

# Unary operators: right-associative prefix operators, or a cast, or a primary.
# Unary - negates; ~ is bitwise NOT (integer-only); unary + is a no-op and dropped.
UNARY -> Minus UNARY
       | Tilde UNARY
       | Plus  UNARY     # no-op; parsed and silently dropped
       | CAST
       | PRIMARY

# An explicit type cast converts an expression to the target type.
# Only the casts permitted by the type system are legal (see decisions.txt).
CAST -> DATATYPE LParen EXPRESSION RParen

# A primary is the highest-precedence expression form: a grouped sub-expression,
# a literal, an array/struct literal, an access chain, or null.
PRIMARY -> LParen EXPRESSION RParen         # grouped sub-expression for precedence override
         | LBracket ARGS RBracket           # array or list literal: [e1, e2, ...]
         | LBrace STRUCT_LIT_FIELDS RBrace  # struct literal: { field = val, ... }
         | ACCESS_CHAIN                     # variable, field access, index, or function call
         | SIntLit
         | FloatLit
         | CharLit
         | StringLit
         | BoolLit
         | Null

# A chain starting from a named variable with up to 8 postfix steps.
# Each step can be a field access, an index, or a call. The chain is the
# unified representation for variables, calls, indexing, and member access.
ACCESS_CHAIN -> Identifier POSTFIX*   (max 8 POSTFIX steps)

# The three kinds of postfix step that can follow an identifier or prior step.
POSTFIX -> ColonColon Identifier         # field/member access: x::field
         | LBracket EXPRESSION RBracket  # index access: x[i]
         | LParen ARGS RParen            # function or method call: x(args)

# A struct literal body: zero or more named field initialisers.
# Field order must match the struct definition.
STRUCT_LIT_FIELDS -> Identifier Equals EXPRESSION STRUCT_LIT_TAIL
                   | ε
STRUCT_LIT_TAIL   -> Comma Identifier Equals EXPRESSION STRUCT_LIT_TAIL
                   | ε

# A comma-separated argument list for calls and array/list literals.
# An empty list is valid. Trailing commas are a parse error.
ARGS      -> EXPRESSION ARGS_TAIL
           | ε
ARGS_TAIL -> Comma EXPRESSION ARGS_TAIL
           | ε
```

| Token              | Lexeme                                                                 |
|--------------------|------------------------------------------------------------------------|
| `Start`            | `!start`                                                               |
| `End`              | `!end`                                                                 |
| `Func`             | `!func`                                                                |
| `If`               | `!if`                                                                  |
| `Elif`             | `!elif`                                                                |
| `Else`             | `!else`                                                                |
| `For`              | `!for`                                                                 |
| `While`            | `!while`                                                               |
| `Return`           | `!return`                                                              |
| `Exit`             | `!exit`                                                                |
| `Break`            | `!break`                                                               |
| `Continue`         | `!continue`                                                            |
| `Import`           | `!import`                                                              |
| `Module`           | `!module`                                                              |
| `And`              | `!and`                                                                 |
| `Or`               | `!or`                                                                  |
| `Not`              | `!not`                                                                 |
| `Null`             | `!null`                                                                |
| `TypeInt`          | `:int`                                                                 |
| `TypeFloat`        | `:float`                                                               |
| `TypeChar`         | `:char`                                                                |
| `TypeBoolean`      | `:boolean`                                                             |
| `TypeVoid`         | `:void`                                                                |
| `TypeArray`        | `:array`                                                               |
| `TypeList`         | `:list`                                                                |
| `TypeStruct`       | `:struct`                                                              |
| `SIntLit(i64)`     | decimal `42`, hex `0xFF`, binary `0b1010`, octal `0o77`, prefixed decimal `0d42` |
| `FloatLit(f64)`    | `3.14`, `2.0e-5`                                                       |
| `CharLit(char)`    | `'a'`, `'\n'`                                                          |
| `StringLit(String)`| `"hello\n"`                                                            |
| `BoolLit(bool)`    | `true`, `false`                                                        |
| `Plus`             | `+`                                                                    |
| `Minus`            | `-`                                                                    |
| `Star`             | `*`                                                                    |
| `Slash`            | `/`                                                                    |
| `Percent`          | `%`                                                                    |
| `Ampersand`        | `&`                                                                    |
| `Pipe`             | `\|`                                                                   |
| `Caret`            | `^`                                                                    |
| `Tilde`            | `~`                                                                    |
| `Equals`           | `=`                                                                    |
| `PlusEquals`       | `+=`                                                                   |
| `MinusEquals`      | `-=`                                                                   |
| `StarEquals`       | `*=`                                                                   |
| `SlashEquals`      | `/=`                                                                   |
| `PercentEquals`    | `%=`                                                                   |
| `AmpersandEquals`  | `&=`                                                                   |
| `PipeEquals`       | `\|=`                                                                  |
| `CaretEquals`      | `^=`                                                                   |
| `Greater`          | `>`                                                                    |
| `Less`             | `<`                                                                    |
| `GreaterEquals`    | `>=`                                                                   |
| `LessEquals`       | `<=`                                                                   |
| `EqualsEquals`     | `==`                                                                   |
| `TildeEquals`      | `~=`                                                                   |
| `Arrow`            | `->`                                                                   |
| `Dot`              | `.`  *(reserved/tokenised but not used in any grammar production)*     |
| `Comma`            | `,`                                                                    |
| `ColonColon`       | `::`                                                                   |
| `LParen`           | `(`                                                                    |
| `RParen`           | `)`                                                                    |
| `LBrace`           | `{`                                                                    |
| `RBrace`           | `}`                                                                    |
| `LBracket`         | `[`                                                                    |
| `RBracket`         | `]`                                                                    |
| `EndL`             | `;`                                                                    |
| `ModuleStart(name)`| `$MODULE_START:n$`                                                     |
| `ModuleEnd(name)`  | `$MODULE_END:n$`                                                       |
