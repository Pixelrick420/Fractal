"use client";
import { useState, useRef } from "react";
import Editor from "@monaco-editor/react";
import Navbar from "../../components/Navbar";
import styles from "../docs/docs.module.css";
import {
  BookOpen,
  Code2,
  Variable,
  Hash,
  List,
  Brackets,
  Repeat,
  Import,
  ChevronRight,
  AlertCircle,
  Info,
} from "lucide-react";

const KEYWORDS = [
  "start",
  "end",
  "exit",
  "if",
  "elif",
  "else",
  "for",
  "while",
  "func",
  "return",
  "struct",
  "import",
  "module",
  "break",
  "continue",
  "and",
  "or",
  "not",
  "null",
];

const TYPES = [
  "int",
  "float",
  "char",
  "boolean",
  "array",
  "list",
  "struct",
  "void",
];

type TableData = { title: string; headers: string[]; rows: string[][] };

type Section = {
  id: string;
  title: string;
  icon: any;
  description: string;
  code: string;
  notes?: { type: "note" | "warning"; text: string }[];
  tables?: TableData[];
};

const SECTIONS: Section[] = [
  {
    id: "quick-reference",
    title: "Quick Reference",
    icon: BookOpen,
    description:
      "This page provides a quick syntax overview. Click chapters in the sidebar for detailed documentation.",
    code: `!start
    # your code here
!end`,
    tables: [
      {
        title: "Keywords",
        headers: ["Keyword", "Description"],
        rows: [
          ["!start / !end", "Program delimiters"],
          ["!func", "Declare a function"],
          ["!if / !elif / !else", "Conditional"],
          ["!for", "Counted loop"],
          ["!while", "Condition loop"],
          ["!return", "Return from function"],
          ["!break / !continue", "Loop control"],
          ["!import", "Import another file"],
          ["!module", "Define a module"],
          ["!exit", "Terminate program"],
        ],
      },
      {
        title: "Types",
        headers: ["Type", "Description"],
        rows: [
          [":int", "64-bit integer"],
          [":float", "64-bit float"],
          [":char", "Unicode character"],
          [":boolean", "true or false"],
          [":void", "Null type"],
          [":array<T, N>", "Fixed array"],
          [":list<T>", "Dynamic list"],
          [":struct<Name>", "User struct"],
        ],
      },
      {
        title: "Operators",
        headers: ["Operator", "Description"],
        rows: [
          ["+ - * / %", "Arithmetic"],
          ["& | ^ ~", "Bitwise"],
          ["!not !and !or", "Logical"],
          ["== ~= > < >= <=", "Comparison"],
          ["+= -= *= /= %=", "Compound assign"],
          ["::", "Struct member"],
          ["->", "Return type"],
        ],
      },
    ],
  },
  {
    id: "getting-started",
    title: "Getting Started",
    icon: BookOpen,
    description:
      "Welcome to Fractal - a statically-typed, strongly-typed language designed for clarity and correctness.\n\nEvery program must begin with !start and end with !end.",
    code: `!start
    print("Hello, World!\\n");
!end

# Variables & Functions
!start
    :int x = 10;
    :int y = 20;

    !func sum(:int a, :int b) -> :int {
        !return a + b;
    }

    print("{} + {} = {}", x, y, sum(x, y));
!end

# Loops & Conditionals
!start
    :int total = 0;

    !for (:int i, 1, 6, 1) {
        !if (i % 2 == 0) {
            total = total + i;
        }
    }

    print("Sum of evens 1-5: {}", total);
!end`,
    notes: [
      {
        type: "note",
        text: "Fractal was built around three principles: No implicit casts (every type conversion is explicit with :Type(expr)), Compile-time safety (type errors caught before running), Minimal syntax (keywords use ! prefix, never conflict with variables).",
      },
    ],
  },
  {
    id: "types-variables",
    title: "Types & Variables",
    icon: Variable,
    description:
      "Fractal is statically typed. Every variable must have an explicit type annotation.\n\nType names always start with : (colon).",
    code: `# Simple types
:int     count = 42;
:float   ratio = 0.618;
:char    letter = 'F';
:boolean flag = true;

# Default values
:int zero;     # 0
:float f;      # 0.0

# Collection types
:array<:int, 100> numbers;
:list<:float> floats;

# Type casting
:float f = :float(42);    # 42.0
:int n = :int(3.99);     # 3
:char c = :char(65);      # 'A'

# Indexing
:int first = arr[0];`,
    notes: [
      {
        type: "note",
        text: "Integer literals: 255 (decimal), 0xFF (hexadecimal), 0b1111 (binary), 0o377 (octal).",
      },
      {
        type: "note",
        text: "Float literals: 1.5 (plain), 1.5e6 (scientific).",
      },
      {
        type: "note",
        text: "Default values: :int = 0, :float = 0.0, :char = '\\0', :boolean = false.",
      },
    ],
    tables: [
      {
        title: "Simple Types",
        headers: ["Type", "Description", "Default"],
        rows: [
          [":int", "64-bit signed integer", "0"],
          [":float", "64-bit IEEE 754 float", "0.0"],
          [":char", "Unicode character", "'\\0'"],
          [":boolean", "true or false", "false"],
          [":void", "Null type", "-"],
        ],
      },
      {
        title: "Collection Types",
        headers: ["Type", "Description"],
        rows: [
          [":array<T, N>", "Fixed-size array of N elements"],
          [":list<T>", "Dynamic list"],
        ],
      },
      {
        title: "Type Casting",
        headers: ["Cast", "Effect"],
        rows: [
          [":int(expr)", "Convert to int (truncates)"],
          [":float(expr)", "Convert to float"],
          [":char(expr)", "Convert to char"],
          [":boolean(expr)", "Convert to boolean"],
        ],
      },
    ],
  },
  {
    id: "operators",
    title: "Operators",
    icon: Hash,
    description:
      "Fractal uses ~= for not-equal and ! prefix for logical operators.\n\nAll operators follow standard precedence rules.",
    code: `# Arithmetic
+ - * / %

# Comparison
== ~= < > <= >=

# Logic
!and !or !not

# Bitwise
& | ^ ~

# Assignment
+= -= *= /= %=

# Casting (explicit conversion)
:int x = :int(5.7);  # 5
:float y = :float(5);  # 5.0`,
  },
  {
    id: "control-flow",
    title: "Control Flow",
    icon: Repeat,
    description:
      "Fractal has !func, !if, !for, and !while. All blocks use curly braces.\n\nFunction definitions must appear at the top level, not inside any block.",
    code: `# Functions
!func add(:int a, :int b) -> :int {
    !return a + b;
}

!func greet(:char name) -> :void {
    print("Hello {}\\n", name);
}

# Conditionals
!if (x > 0) {
    print("positive\\n");
}
!elif (x < 0) {
    print("negative\\n");
}
!else {
    print("zero\\n");
}

# For Loop
!for (:int i, 0, 10, 1) {
    print("{}", i);
}

# While Loop
:int n = 10;
!while (n > 0) {
    print("{}", n);
    n = n - 1;
}

# Break & Continue
!for (:int i, 0, 100, 1) {
    !if (i == 42) { !break; }
}

!for (:int i, 0, 10, 1) {
    !if (i % 2 == 0) { !continue; }
    print("{}", i);  # prints 1,3,5,7,9
}

# Recursion
!func factorial(:int n) -> :int {
    !if (n <= 1) { !return 1; }
    !return n * factorial(n - 1);
}

# Variable Scope
:int global = 10;
!if (true) {
    :int local = 20;
    global = local;
}`,
    notes: [
      {
        type: "warning",
        text: "The opening { must be on the same line as the condition.",
      },
      {
        type: "note",
        text: "The loop variable must be :int and must not shadow outer variables.",
      },
    ],
  },
  {
    id: "structs",
    title: "Structs",
    icon: Brackets,
    description:
      "User-defined types that group related fields. Members accessed with ::.\n\nStructs enable creating complex data types for your program.",
    code: `# Defining Structs
:struct<Vec2> {
    :float x;
    :float y;
};

:struct<Rect> {
    :float x;
    :float y;
    :float w;
    :float h;
};

# Nested Structs
:struct<Particle> {
    :struct<Vec2> pos;
    :struct<Vec2> vel;
    :float mass;
};

# Initialisation
:struct<Vec2> origin = { x = 0.0, y = 0.0 };
:struct<Vec2> p = { x = 3.0, y = 4.0 };

# Member Access
:float x = p::x;
:float y = p::y;

# Structs in Functions
!func distance(:struct<Vec2> a, :struct<Vec2> b) -> :float {
    :float dx = b::x - a::x;
    :float dy = b::y - a::y;
    !return :float(dx * dx + dy * dy);
}

# Self-Referential (linked structures)
:struct<Node> {
    :int value;
    :struct<Node> next;
};

:struct<Node> head = { value = 1, next = !null };
head::next = { value = 2, next = !null };`,
    notes: [
      {
        type: "warning",
        text: "Declaring a struct without an initializer produces a warning - structs have no default value.",
      },
    ],
  },
  {
    id: "builtins",
    title: "Standard Library",
    icon: List,
    description:
      "Built-in functions available in every Fractal program.\n\nThese functions are available without importing anything.",
    code: `# Output
print("{}", value);
input()

# List operations
append(list, value)
pop(list)
insert(list, index, value)
delete(list, index)
find(list, value)

# Other
len(array_or_list)
abs(number)
sqrt(number)
pow(base, exp)
floor(float)
ceil(float)
min(a, b)
max(a, b)

# String operations
len(str)
substring(str, start, end)
starts_with(str, prefix)
ends_with(str, suffix)
replace(str, old, new)
split(str, delimiter)

# Type checking
typeof(value)`,
    tables: [
      {
        title: "Output Functions",
        headers: ["Function", "Description"],
        rows: [
          ['print("{}", value)', "Print formatted output"],
          ["input()", "Read line from stdin"],
        ],
      },
      {
        title: "List Operations",
        headers: ["Function", "Description"],
        rows: [
          ["append(list, value)", "Add to end"],
          ["pop(list)", "Remove last element"],
          ["insert(list, i, v)", "Insert at index"],
          ["delete(list, i)", "Delete at index"],
          ["find(list, value)", "Find index of value"],
        ],
      },
      {
        title: "Math Functions",
        headers: ["Function", "Description"],
        rows: [
          ["abs(n)", "Absolute value"],
          ["sqrt(n)", "Square root"],
          ["pow(b, e)", "Base to exponent"],
          ["floor(f)", "Round down"],
          ["ceil(f)", "Round up"],
          ["min(a, b)", "Smaller value"],
          ["max(a, b)", "Larger value"],
        ],
      },
      {
        title: "String Functions",
        headers: ["Function", "Description"],
        rows: [
          ["len(str)", "String length"],
          ["substring(s, start, end)", "Slice string"],
          ["starts_with(s, p)", "Check prefix"],
          ["ends_with(s, p)", "Check suffix"],
          ["replace(s, old, new)", "Replace text"],
          ["split(s, delim)", "Split by delimiter"],
        ],
      },
    ],
  },
];

export default function DocsPage() {
  const [activeSection, setActiveSection] = useState("quick-reference");
  const editorRef = useRef<any>(null);

  const currentSection =
    SECTIONS.find((s) => s.id === activeSection) || SECTIONS[0];

  const handleEditorMount = (editor: any, monaco: any) => {
    editorRef.current = editor;

    monaco.languages.register({ id: "fractal" });
    monaco.languages.setMonarchTokensProvider("fractal", {
      keywords: KEYWORDS,
      typeKeywords: TYPES,
      tokenizer: {
        root: [
          [/#.*$/, "comment"],
          [/"[^"]*"/, "string"],
          [/'[^']*'/, "string"],
          [/::/, "delimiter"],
          [
            /[:!][a-zA-Z_]\w*/,
            {
              cases: {
                "@keywords": "keyword",
                "@typeKeywords": "type",
                "@default": "identifier",
              },
            },
          ],
          [/[+\-*/%=<>&|^~]+/, "operator"],
          [/\d+/, "number"],
        ],
      },
    });

    monaco.editor.defineTheme("fractal-dark", {
      base: "vs-dark",
      inherit: true,
      rules: [
        { token: "keyword", foreground: "ff7b72", fontStyle: "bold" },
        { token: "type", foreground: "79c0ff" },
        { token: "identifier", foreground: "ffa657" },
        { token: "number", foreground: "a5d6a7" },
        { token: "string", foreground: "a5d6ff" },
        { token: "comment", foreground: "6a7f9e", fontStyle: "italic" },
        { token: "operator", foreground: "d2a8ff" },
      ],
      colors: {
        "editor.background": "#0e1117",
        "editor.foreground": "#e2e8f6",
        "editor.lineHighlightBackground": "#121723",
        "editorCursor.foreground": "#15ce43",
      },
    });
    monaco.editor.setTheme("fractal-dark");
  };

  return (
    <div className={styles.root}>
      <Navbar />
      <div className={styles.main}>
        <div className={styles.sidebar}>
          <nav className={styles.nav}>
            {SECTIONS.map(({ id, title, icon: Icon }) => (
              <button
                key={id}
                className={`${styles.navItem} ${activeSection === id ? styles.navItemActive : ""}`}
                onClick={() => setActiveSection(id)}
              >
                <Icon size={16} />
                {title}
                {activeSection === id && (
                  <ChevronRight size={14} className={styles.navArrow} />
                )}
              </button>
            ))}
          </nav>
        </div>

        <div className={styles.content}>
          <div className={styles.titleArea}>
            <h1 className={styles.title}>{currentSection.title}</h1>
            <p className={styles.description}>
              {currentSection.description.split("\n\n").map((line, i, arr) => (
                <span key={i}>
                  {line}
                  {i < arr.length - 1 && <br />}
                  {i < arr.length - 1 && <br />}
                </span>
              ))}
            </p>
          </div>

          {currentSection.tables && currentSection.tables.length > 0 && (
            <div className={styles.tablesArea}>
              {currentSection.tables.map((table, i) => (
                <div key={i} className={styles.tableSection}>
                  <h3 className={styles.tableTitle}>{table.title}</h3>
                  <table className={styles.table}>
                    <thead>
                      <tr>
                        {table.headers.map((h, j) => (
                          <th key={j}>{h}</th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {table.rows.map((row, j) => (
                        <tr key={j}>
                          {row.map((cell, k) => (
                            <td key={k}>{cell}</td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ))}
            </div>
          )}

          {currentSection.notes && currentSection.notes.length > 0 && (
            <div className={styles.notesArea}>
              {currentSection.notes.map((note, i) => (
                <div
                  key={i}
                  className={`${styles.note} ${note.type === "warning" ? styles.noteWarning : styles.noteInfo}`}
                >
                  {note.type === "warning" ? (
                    <AlertCircle size={16} className={styles.noteIcon} />
                  ) : (
                    <Info size={16} className={styles.noteIcon} />
                  )}
                  <span>{note.text}</span>
                </div>
              ))}
            </div>
          )}

          <div className={styles.codeArea}>
            <div className={styles.codeHeader}>
              <span className={styles.fileName}>example.fr</span>
            </div>
            <div className={styles.editorWrapper}>
              <Editor
                height="100%"
                defaultLanguage="fractal"
                value={currentSection.code}
                onMount={handleEditorMount}
                loading={<div className={styles.loading}>Loading...</div>}
                options={{
                  readOnly: true,
                  minimap: { enabled: false },
                  fontSize: 13,
                  fontFamily: '"DM Mono", monospace',
                  lineNumbers: "on",
                  lineHeight: 22,
                  padding: { top: 12, bottom: 12 },
                  scrollBeyondLastLine: false,
                  automaticLayout: true,
                  wordWrap: "on",
                }}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
