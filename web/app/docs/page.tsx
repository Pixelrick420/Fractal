"use client";
import { useState, useCallback, useMemo, useEffect, useRef } from "react";
import Editor, { OnMount, BeforeMount } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import { Info, AlertTriangle, Lightbulb } from "lucide-react";
import Navbar from "../../components/Navbar";
import styles from "./docs.module.css";
import docsJson from "./docs.json";

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

type NoteKind = "info" | "warning" | "tip";

interface Note {
  kind: NoteKind;
  text: string;
}

interface DocTable {
  headers: string[];
  rows: string[][];
}

interface DocSection {
  title: string;
  description?: string;
  code?: string;
  table?: DocTable;
  note?: Note;
  subsections?: DocSection[];
  subcontent?: string;
  subcontent2?: string;
  code2?: string;
  code3?: string;
}

interface Chapter {
  id: string;
  label: string;
  sections: DocSection[];
}

const chapters = (docsJson as any).chapters as Chapter[];

let fractalReady = false;
const handleBeforeMount: BeforeMount = (monaco) => {
  if (fractalReady) {
    monaco.editor.setTheme("fractal-docs");
    return;
  }
  fractalReady = true;

  if (
    !monaco.languages
      .getLanguages()
      .some((l: monaco.languages.ILanguageExtensionPoint) => l.id === "fractal")
  ) {
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
  }

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
};

// ── CodeExample ───────────────────────────────────────────────────────────────

interface CodeExampleProps {
  code: string;
  title?: string;
}

function CodeExample({ code, title }: CodeExampleProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [editorHeight, setEditorHeight] = useState(80);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const calculate = () => {
      const width = el.clientWidth;
      const charsPerLine = Math.floor(width / 7.8);
      const lines = code.split("\n");
      const totalLines = lines.reduce((acc, line) => {
        return acc + Math.max(1, Math.ceil(line.length / charsPerLine));
      }, 0);
      setEditorHeight(Math.min(Math.max(totalLines * 22 + 32, 80), 900));
    };

    const observer = new ResizeObserver(calculate);
    observer.observe(el);
    calculate();
    return () => observer.disconnect();
  }, [code]);

  // onMount is only needed to guarantee the theme is applied even if
  // beforeMount somehow ran before defineTheme completed (edge-case safety).
  const handleMount: OnMount = useCallback((_editor, monaco) => {
    monaco.editor.setTheme("fractal-docs");
  }, []);

  return (
    <div className={styles.codeSection}>
      {title && <h2 className={styles.sectionTitle}>{title}</h2>}
      <div className={styles.codeHeader}>
        <span className={styles.codeFileName}>example.fr</span>
        <span className={styles.codeLang}>Fractal</span>
      </div>
      <div
        ref={containerRef}
        className={styles.editorWrapper}
        style={{ height: editorHeight }}
      >
        <Editor
          height="100%"
          defaultLanguage="fractal"
          value={code}
          theme="fractal-docs" // ← explicit theme: no flash
          beforeMount={handleBeforeMount} // ← defines theme before first paint
          onMount={handleMount} // ← safety net
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
            scrollbar: {
              verticalScrollbarSize: 4,
              alwaysConsumeMouseWheel: false,
            },
            domReadOnly: true,
            cursorStyle: "line",
            renderValidationDecorations: "off",
          }}
        />
      </div>
    </div>
  );
}

// ── DocsPage ──────────────────────────────────────────────────────────────────

export default function DocsPage() {
  const [active, setActive] = useState<string>(
    chapters[0]?.id ?? "quick_reference",
  );

  const activeChapter = useMemo(
    () => chapters.find((c) => c.id === active) ?? chapters[0],
    [active],
  );

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

  const renderTable = (table: DocTable) => (
    <div className={styles.tableWrap}>
      <table className={styles.table}>
        <thead>
          <tr>
            {table.headers.map((h) => (
              <th key={h}>{h}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {table.rows.map((row, i) => (
            <tr key={i}>
              {row.map((cell, j) => (
                <td key={j}>{cell}</td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );

  const renderSection = (section: DocSection, index: number) => (
    <div key={index} className={styles.section}>
      <h2 className={styles.sectionTitle}>{section.title}</h2>

      {section.description && (
        <p className={styles.description}>{section.description}</p>
      )}

      {section.table && renderTable(section.table)}

      {section.code && <CodeExample code={section.code} />}

      {section.note && (
        <div className={`${styles.note} ${noteClass(section.note.kind)}`}>
          <span className={styles.noteIcon}>{noteIcon(section.note.kind)}</span>
          <span className={styles.noteText}>{section.note.text}</span>
        </div>
      )}

      {section.subcontent && <p>{section.subcontent}</p>}
      {section.code2 && <CodeExample code={section.code2} />}
      {section.subcontent2 && <p>{section.subcontent2}</p>}
      {section.code3 && <CodeExample code={section.code3} />}

      {section.subsections?.map((sub, subIndex) => (
        <div key={subIndex} className={styles.subsection}>
          <h3>{sub.title}</h3>
          {sub.code && <CodeExample code={sub.code} />}
        </div>
      ))}
    </div>
  );

  return (
    <div className={styles.root}>
      <Navbar />

      <div className={styles.main}>
        <aside className={styles.sidebar}>
          <div className={styles.sidebarHeader}>
            <span className={styles.sidebarLabel}>Documentation</span>
          </div>
          <nav className={styles.nav}>
            <div className={styles.navGroup}>
              {chapters.map((chapter) => (
                <button
                  key={chapter.id}
                  className={`${styles.navItem} ${
                    active === chapter.id ? styles.navItemActive : ""
                  }`}
                  onClick={() => setActive(chapter.id)}
                >
                  <span className={styles.navDot} />
                  {chapter.label}
                </button>
              ))}
            </div>
          </nav>
        </aside>

        <main className={styles.content}>
          <div className={styles.titleArea}>
            <h1 className={styles.title}>{activeChapter.label}</h1>
          </div>

          <div className={styles.body}>
            {activeChapter.sections.map((section, index) =>
              renderSection(section, index),
            )}
          </div>
        </main>
      </div>
    </div>
  );
}
