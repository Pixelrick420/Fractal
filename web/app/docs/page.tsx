"use client";
import { useState, useCallback } from "react";
import Editor, { OnMount } from "@monaco-editor/react";
import { Info, AlertTriangle, Lightbulb } from "lucide-react";
import Navbar from "../../components/Navbar";
import styles from "./docs.module.css";
import type { languages } from "monaco-editor";

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

// ── Docs data ─────────────────────────────────────────────
type NoteKind = "info" | "warning" | "tip";

interface Note {
  kind: NoteKind;
  text: string;
}

interface TableRow {
  col1: string;
  col2: string;
  col3?: string;
}

interface DocSection {
  id: string;
  label: string;
  tag: string;
  title: string;
  description: string;
  tables: { title: string; headers: string[]; rows: TableRow[] }[];
  notes: Note[];
  codeFile: string;
  code: string;
}

const DOCS: DocSection[] = [
  {
    id: "variables",
    label: "Variables",
    tag: "Core",
    title: "Variables & Types",
    description:
      "Fractal uses a prefix-based declaration syntax. Variables are declared with a colon followed by the type, making the type immediately visible at the declaration site.",
    tables: [
      {
        title: "Primitive Types",
        headers: ["Type", "Description", "Example"],
        rows: [
          { col1: ":int", col2: "32-bit signed integer", col3: ":int x = 42;" },
          {
            col1: ":float",
            col2: "64-bit floating point",
            col3: ":float pi = 3.14;",
          },
          {
            col1: ":char",
            col2: "Single UTF-8 character",
            col3: ":char c = 'a';",
          },
          {
            col1: ":boolean",
            col2: "true or false",
            col3: ":boolean ok = true;",
          },
        ],
      },
      {
        title: "Collection Types",
        headers: ["Type", "Description", "Example"],
        rows: [
          {
            col1: ":array",
            col2: "Fixed-size typed array",
            col3: ":array<int> nums = [1,2,3];",
          },
          {
            col1: ":list",
            col2: "Dynamic growable list",
            col3: ":list<int> xs = [];",
          },
          {
            col1: ":struct",
            col2: "Named field composite",
            col3: "struct Point { ... }",
          },
        ],
      },
    ],
    notes: [
      {
        kind: "info",
        text: "All variables must be declared before use. Fractal has no implicit type coercion.",
      },
      {
        kind: "tip",
        text: "Use :int for general counting and indexing; :float when fractional precision matters.",
      },
    ],
    codeFile: "variables.fr",
    code: `!start
    # integer arithmetic
    :int x = 10;
    :int y = 20;
    :int sum = x + y;
    print("Sum: {}", sum);

    # floats
    :float pi = 3.14159;
    :float area = pi * 5.0 * 5.0;
    print("Area: {:.2}", area);

    # boolean
    :boolean big = sum > 25;
    print("Big? {}", big);
!end`,
  },
  {
    id: "control",
    label: "Control Flow",
    tag: "Core",
    title: "Control Flow",
    description:
      "Fractal supports familiar control flow constructs — if/elif/else, while loops, and for loops — with a clean block syntax using curly braces.",
    tables: [
      {
        title: "Branching",
        headers: ["Construct", "Description"],
        rows: [
          {
            col1: "if (cond) { }",
            col2: "Execute block when condition is true",
          },
          { col1: "elif (cond) { }", col2: "Else-if branch" },
          { col1: "else { }", col2: "Fallback branch" },
        ],
      },
      {
        title: "Loops",
        headers: ["Construct", "Description"],
        rows: [
          { col1: "while (cond) { }", col2: "Repeat while condition holds" },
          { col1: "for i in 0..n { }", col2: "Iterate over a range" },
          { col1: "break", col2: "Exit the nearest loop immediately" },
          { col1: "continue", col2: "Skip to the next loop iteration" },
        ],
      },
    ],
    notes: [
      {
        kind: "info",
        text: "Conditions do not require parentheses but they are accepted. The body braces are always required.",
      },
      {
        kind: "warning",
        text: "Fractal does not have a switch/match statement yet. Use if/elif chains instead.",
      },
    ],
    codeFile: "control.fr",
    code: `!start
    :int n = 10;
    :int i = 0;

    while (i < n) {
        if (i % 2 == 0) {
            print("{} is even", i);
        } elif (i == 7) {
            print("lucky seven!");
        } else {
            print("{} is odd", i);
        }
        i = i + 1;
    }
!end`,
  },
  {
    id: "functions",
    label: "Functions",
    tag: "Core",
    title: "Functions",
    description:
      "Functions are declared with the func keyword. Parameters are typed inline and the return type follows a thin arrow. Fractal functions are first-class values.",
    tables: [
      {
        title: "Syntax",
        headers: ["Element", "Description"],
        rows: [
          { col1: "func name(...)", col2: "Function declaration" },
          { col1: "-> type", col2: "Explicit return type (void if omitted)" },
          { col1: "return val;", col2: "Return a value from the function" },
          { col1: ":void", col2: "Functions that produce no value" },
        ],
      },
    ],
    notes: [
      {
        kind: "tip",
        text: "Functions can be declared anywhere inside a !start/!end block and called before their textual position.",
      },
      {
        kind: "info",
        text: "Recursive functions are fully supported. Stack depth is limited by the system stack.",
      },
    ],
    codeFile: "functions.fr",
    code: `!start
    func add(a: int, b: int) -> int {
        return a + b;
    }

    func greet(name: char) -> void {
        print("Hello, {}!", name);
    }

    func factorial(n: int) -> int {
        if (n <= 1) { return 1; }
        return n * factorial(n - 1);
    }

    :int result = add(3, 4);
    print("3 + 4 = {}", result);
    print("10! = {}", factorial(10));
    greet('W');
!end`,
  },
  {
    id: "structs",
    label: "Structs",
    tag: "Core",
    title: "Structs",
    description:
      "Structs group related data under a single named type. Fields are accessed with the dot operator. Structs are value types — assignment copies the entire struct.",
    tables: [
      {
        title: "Struct Operations",
        headers: ["Syntax", "Description"],
        rows: [
          { col1: "struct Name { }", col2: "Declare a new struct type" },
          {
            col1: "field: type",
            col2: "Typed field declaration inside struct",
          },
          { col1: "val.field", col2: "Field access" },
          { col1: "val.field = x;", col2: "Field assignment" },
        ],
      },
    ],
    notes: [
      {
        kind: "info",
        text: "Structs can be nested — a field may itself be a struct type.",
      },
      {
        kind: "warning",
        text: "Circular struct references are not supported in the current version.",
      },
    ],
    codeFile: "structs.fr",
    code: `!start
    struct Point {
        x: int,
        y: int,
    }

    struct Rect {
        origin: Point,
        width: int,
        height: int,
    }

    :Point p = Point { x: 10, y: 20 };
    print("Point: ({}, {})", p.x, p.y);

    :Rect r = Rect {
        origin: p,
        width: 100,
        height: 50,
    };
    :int area = r.width * r.height;
    print("Area: {}", area);
!end`,
  },
  {
    id: "modules",
    label: "Modules",
    tag: "Advanced",
    title: "Modules & Imports",
    description:
      "Fractal organises code into modules. Use the import keyword to bring in another file. Symbols are accessed via the module name with the :: operator.",
    tables: [
      {
        title: "Module System",
        headers: ["Keyword", "Description"],
        rows: [
          { col1: "module name", col2: "Declare this file as a named module" },
          { col1: "import path", col2: "Import another Fractal file by path" },
          { col1: "mod::symbol", col2: "Access an exported symbol" },
        ],
      },
    ],
    notes: [
      {
        kind: "info",
        text: "Circular imports are detected at compile time and result in a hard error.",
      },
      {
        kind: "tip",
        text: "Keep one module per file and name the file to match the module name for clarity.",
      },
    ],
    codeFile: "main.fr",
    code: `# math.fr
module math

func square(n: int) -> int {
    return n * n;
}

# main.fr
import "math.fr"

!start
    :int x = math::square(9);
    print("9² = {}", x);
!end`,
  },
];

// ── Component ─────────────────────────────────────────────
export default function DocsPage() {
  const [active, setActive] = useState<string>(DOCS[0].id);

  const doc = DOCS.find((d) => d.id === active) ?? DOCS[0];

  const handleEditorMount: OnMount = useCallback((_editor, monaco) => {
    if (
      monaco.languages
        .getLanguages()
        .some((l: languages.ILanguageExtensionPoint) => l.id === "fractal")
    )
      return;

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

    monaco.editor.defineTheme("fractal-docs", {
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
        { token: "delimiter", foreground: "d2a8ff" },
      ],
      colors: {
        "editor.background": "#080b10",
        "editor.foreground": "#e2e8f6",
        "editor.lineHighlightBackground": "#0d1219",
        "editorCursor.foreground": "#15ce43",
        "editor.selectionBackground": "#252d4480",
        "editorLineNumber.foreground": "#2a3348",
        "editorLineNumber.activeForeground": "#4a5568",
        "editorGutter.background": "#080b10",
      },
    });

    monaco.editor.setTheme("fractal-docs");
  }, []);

  const noteIcon = (kind: NoteKind) => {
    if (kind === "warning") return <AlertTriangle size={14} />;
    if (kind === "tip") return <Lightbulb size={14} />;
    return <Info size={14} />;
  };

  const noteClass = (kind: NoteKind) =>
    kind === "warning"
      ? styles.noteWarning
      : kind === "tip"
        ? styles.noteTip
        : styles.noteInfo;

  const Items = DOCS;

  return (
    <div className={styles.root}>
      <Navbar />

      <div className={styles.main}>
        {/* Sidebar */}
        <aside className={styles.sidebar}>
          <div className={styles.sidebarHeader}>
            <span className={styles.sidebarLabel}>Documentation</span>
          </div>
          <nav className={styles.nav}>
            <div className={styles.navGroup}>
              {Items.map((d) => (
                <button
                  key={d.id}
                  className={`${styles.navItem} ${active === d.id ? styles.navItemActive : ""}`}
                  onClick={() => setActive(d.id)}
                >
                  <span className={styles.navDot} />
                  {d.label}
                </button>
              ))}
            </div>
          </nav>
        </aside>

        {/* Content */}
        <main className={styles.content}>
          {/* Title */}
          <div className={styles.titleArea}>
            <div className={styles.titleMeta}>
              <span className={styles.titleTag}>{doc.tag}</span>
            </div>
            <h1 className={styles.title}>{doc.title}</h1>
            <p className={styles.description}>{doc.description}</p>
          </div>

          <div className={styles.body}>
            {/* Tables */}
            {doc.tables.map((t) => (
              <div key={t.title} className={styles.section}>
                <h2 className={styles.sectionTitle}>{t.title}</h2>
                <div className={styles.tableWrap}>
                  <table className={styles.table}>
                    <thead>
                      <tr>
                        {t.headers.map((h) => (
                          <th key={h}>{h}</th>
                        ))}
                      </tr>
                    </thead>
                    <tbody>
                      {t.rows.map((row, i) => (
                        <tr key={i}>
                          <td>{row.col1}</td>
                          <td>{row.col2}</td>
                          {row.col3 !== undefined && <td>{row.col3}</td>}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            ))}

            {/* Notes */}
            {doc.notes.length > 0 && (
              <div className={styles.section}>
                <h2 className={styles.sectionTitle}>Notes</h2>
                <div className={styles.notes}>
                  {doc.notes.map((n, i) => (
                    <div
                      key={i}
                      className={`${styles.note} ${noteClass(n.kind)}`}
                    >
                      <span className={styles.noteIcon}>
                        {noteIcon(n.kind)}
                      </span>
                      <span className={styles.noteText}>{n.text}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Code example */}
            <div className={styles.codeSection}>
              <h2 className={styles.sectionTitle}>Example</h2>
              <div className={styles.codeHeader}>
                <span className={styles.codeFileName}>{doc.codeFile}</span>
                <span className={styles.codeLang}>Fractal</span>
              </div>
              <div className={styles.editorWrapper}>
                <Editor
                  height="100%"
                  defaultLanguage="fractal"
                  value={doc.code}
                  onMount={handleEditorMount}
                  loading={<div className={styles.loading}>Loading…</div>}
                  options={{
                    readOnly: true,
                    minimap: { enabled: false },
                    fontSize: 13,
                    fontFamily: '"DM Mono", "Fira Code", monospace',
                    lineNumbers: "on",
                    lineHeight: 22,
                    padding: { top: 16, bottom: 16 },
                    scrollBeyondLastLine: false,
                    automaticLayout: true,
                    tabSize: 4,
                    wordWrap: "on",
                    renderLineHighlight: "line",
                    scrollbar: { verticalScrollbarSize: 4 },
                    domReadOnly: true,
                    cursorStyle: "line",
                    renderValidationDecorations: "off",
                  }}
                />
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}
