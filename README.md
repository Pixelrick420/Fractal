# Fractal

A beginner-friendly compiled programming language with a built-in desktop IDE.

Fractal bridges the gap between Python's friendliness and C's discipline - strict types, readable syntax, and helpful error messages.

## Language Features

- **Types**: `:int`, `:float`, `:char`, `:boolean`, `:array`, `:list`, `:struct`
- **Control flow**: `!if`, `!elif`, `!else`, `!for`, `!while`, `!break`, `!continue`
- **Functions**: `!func` with typed parameters and return types
- **Modules**: `!module`, `!import`
- **Type casts**: `:int(value)` - explicit and visible

```fractal
!start
    :int age = 25;
    :float score = 9.5;
    :int result = age + :int(score);

    !func fib(:int n) -> :int {
        !if (n <= 1) { !return n; }
        !return fib(n - 1) + fib(n - 2);
    }

    :int result = fib(10);
!end
```

## Installation

### Linux

```bash
# Quick install (recommended)
wget -O install.sh https://raw.githubusercontent.com/Pixelrick420/Fractal/main/executable/install.sh && sudo bash install.sh

# Or build from source
git clone https://github.com/Pixelrick420/Fractal.git
cd Fractal
cargo build --release
sudo cp target/release/fractal-compiler target/release/fractal-editor /usr/bin/
```

### Windows

1. Install [Rust](https://rustup.rs/) (MSVC toolchain)
2. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with C++ workload
3. Build:
```powershell
git clone https://github.com/Pixelrick420/Fractal.git
cd Fractal
cargo build --release
```

## Usage

### Desktop IDE

```bash
fractal-editor
```

### Command Line

```bash
fractal-compiler path/to/file.fr          # compile
fractal-compiler debug path/to/file.fr    # compile with debug info
fractal-compiler --emit-rust path/to/file.fr  # output Rust source
```

Or with Cargo:
```bash
cargo run --bin fractal-editor
cargo run --bin fractal-compiler -- path/to/file.fr
```

## Editor Features

- Syntax highlighting & auto-indentation
- Integrated terminal
- Multi-tab editing
- Search & replace
- Built-in documentation
- Debugger with variable inspection
- AST tree viewer
- Light/dark themes

## Built-in Functions

| Function | Description |
|----------|-------------|
| `print(format, args...)` | Print formatted output |
| `input(prompt, var, ...)` | Read user input into variable(s) |
| `append(list, value)` | Add to list |
| `pop(list)` | Remove last element |
| `insert(list, index, value)` | Insert at position |
| `delete(list, index)` | Remove at position |
| `len(collection)` | Get length |
| `find(collection, value)` | Find index |
| `abs(number)` | Absolute value |
| `sqrt(float)` | Square root |
| `pow(float, float)` | Power |
| `floor(float)` / `ceil(float)` | Round |
| `min(a, b)` / `max(a, b)` | Compare |
| `to_int/to_float/to_str` | Convert type |

## Project Structure

```
Fractal/
├── src/
│   ├── bin/           # Entry points
│   ├── compiler/      # Lexer, parser, codegen, etc.
│   ├── ui/            # Editor components
│   └── files/         # Example programs
├── web/               # Landing page
└── executable/        # Install scripts
```

## License

MIT License