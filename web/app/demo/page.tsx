"use client";
import { useState, useRef, useCallback, useEffect } from "react";
import Editor, { OnMount } from "@monaco-editor/react";
import {
  Play,
  RotateCcw,
  Copy,
  Check,
  Terminal,
  AlertTriangle,
} from "lucide-react";
import Navbar from "../../components/Navbar";
import styles from "./demo.module.css";
import { parseAnsi } from "../lib/ansi";

const STORAGE_KEY = "fractal_demo_code";

const INITIAL_CODE = `!start
    :int x = 10;
    :int y = 20;
    # compute sum
    :int sum = x + y;
    print("Sum: {}", sum);
!end`;

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

function getSavedCode(): string {
  if (typeof window === "undefined") return INITIAL_CODE;
  try {
    return localStorage.getItem(STORAGE_KEY) ?? INITIAL_CODE;
  } catch {
    return INITIAL_CODE;
  }
}

function formatCode(src: string): string {
  const expanded: string[] = [];
  for (const raw of src.split("\n")) {
    const trimmed = raw.trim();
    if (trimmed === "") {
      expanded.push("");
    } else {
      const parts = trimmed.split(/\s+/);
      expanded.push(parts.join(" "));
    }
  }

  const normalised: string[] = expanded
    .map((line) => line.trim())
    .filter((line, i) => i > 0 || line !== "");

  let depth = 0;
  const output: string[] = [];

  for (const line of normalised) {
    const t = line.trim();
    if (t === "") {
      output.push("");
      continue;
    }
    if (t === "!start") {
      output.push("    ".repeat(depth) + t);
      depth++;
      continue;
    }
    if (t === "!end") {
      depth = Math.max(0, depth - 1);
      output.push("    ".repeat(depth) + t);
      continue;
    }
    if (t.startsWith("}")) depth = Math.max(0, depth - 1);
    output.push("    ".repeat(depth) + t);
    const opens = (t.match(/\{/g) || []).length;
    const closes = (t.match(/\}/g) || []).length;
    depth += opens - closes;
  }

  return output.join("\n");
}

export default function DemoPage() {
  const [code, setCode] = useState<string>(INITIAL_CODE);
  const [output, setOutput] = useState("");
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [hasRun, setHasRun] = useState(false);
  const [isError, setIsError] = useState(false);
  const editorRef = useRef<any>(null);

  const usesInput = /\binput\s*\(/.test(code);

  // Hydrate from localStorage after mount (avoids SSR mismatch)
  useEffect(() => {
    const saved = getSavedCode();
    if (saved !== INITIAL_CODE) {
      setCode(saved);
    }
  }, []);

  // Persist to localStorage whenever code changes (debounced 500 ms)
  useEffect(() => {
    const timer = setTimeout(() => {
      try {
        localStorage.setItem(STORAGE_KEY, code);
      } catch {
        // localStorage unavailable — silently ignore
      }
    }, 500);
    return () => clearTimeout(timer);
  }, [code]);

  const handleEditorMount: OnMount = useCallback((editor, monaco) => {
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

    monaco.editor.setTheme("fractal-dark");
    editor.updateOptions({
      dragAndDrop: false,
      cursorBlinking: "smooth",
      smoothScrolling: true,
    });
  }, []);

  const handleRun = async () => {
    const formatted = formatCode(code);
    setCode(formatted);
    setLoading(true);
    setOutput("");
    setIsError(false);
    setHasRun(true);

    try {
      const response = await fetch("/api/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ code: formatted }),
      });
      const data = await response.json();
      if (data.error) {
        setOutput(data.error);
        setIsError(true);
      } else {
        setOutput(data.output || "(no output)");
      }
    } catch {
      setOutput("Failed to connect to runtime.");
      setIsError(true);
    }
    setLoading(false);
  };

  const handleReset = () => {
    setCode(INITIAL_CODE);
    setOutput("");
    setHasRun(false);
    setIsError(false);
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch {
      // ignore
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={styles.root}>
      <Navbar />

      <div className={styles.mobileToolbar}>
        <div className={styles.mobileLeft}>
          <span className={styles.fileName}>demo.fr</span>
        </div>
        <div className={styles.actions}>
          <button
            className={styles.iconBtn}
            onClick={handleCopy}
            title="Copy code"
          >
            {copied ? (
              <Check size={13} strokeWidth={2.5} />
            ) : (
              <Copy size={13} />
            )}
          </button>
          <button
            className={styles.iconBtn}
            onClick={handleReset}
            title="Reset"
          >
            <RotateCcw size={13} />
          </button>
          <button
            className={styles.runBtn}
            onClick={handleRun}
            disabled={loading}
          >
            <Play
              size={13}
              strokeWidth={2.5}
              fill={loading ? "transparent" : "currentColor"}
            />
            <span className={styles.runLabel}>
              {loading ? "Running…" : "Run"}
            </span>
          </button>
        </div>
      </div>

      {usesInput && (
        <div className={styles.inputWarning}>
          <AlertTriangle size={14} className={styles.inputWarningIcon} />
          <span>
            <strong>No input device detected.</strong> Run the compiler locally
            to use <code>input()</code>.
          </span>
        </div>
      )}

      <div className={styles.workspace}>
        <div className={styles.pane}>
          <div className={styles.paneHeader}>
            <div className={styles.paneHeaderLeft}>
              <span className={styles.fileName}>demo.fr</span>
            </div>
            <div className={styles.actions}>
              <button
                className={styles.iconBtn}
                onClick={handleCopy}
                title="Copy code"
              >
                {copied ? (
                  <Check size={13} strokeWidth={2.5} />
                ) : (
                  <Copy size={13} />
                )}
              </button>
              <button
                className={styles.iconBtn}
                onClick={handleReset}
                title="Reset"
              >
                <RotateCcw size={13} />
              </button>
              <button
                className={styles.runBtn}
                onClick={handleRun}
                disabled={loading}
              >
                <Play
                  size={13}
                  strokeWidth={2.5}
                  fill={loading ? "transparent" : "currentColor"}
                />
                <span className={styles.runLabel}>
                  {loading ? "Running…" : "Run"}
                </span>
              </button>
            </div>
          </div>

          <div className={styles.editorBody}>
            <Editor
              height="100%"
              defaultLanguage="fractal"
              value={code}
              onChange={(value) => setCode(value || "")}
              onMount={handleEditorMount}
              loading={
                <div className={styles.editorLoading}>Loading editor…</div>
              }
              options={{
                minimap: { enabled: false },
                fontSize: 13,
                fontFamily: '"DM Mono", "Fira Code", monospace',
                lineNumbers: "on",
                lineHeight: 22,
                padding: { top: 20, bottom: 20 },
                scrollBeyondLastLine: false,
                automaticLayout: true,
                tabSize: 4,
                wordWrap: "on",
                cursorBlinking: "smooth",
                cursorSmoothCaretAnimation: "on",
                smoothScrolling: true,
                renderLineHighlight: "line",
                bracketPairColorization: { enabled: true },
                fixedOverflowWidgets: true,
                scrollbar: {
                  verticalScrollbarSize: 4,
                  horizontalScrollbarSize: 4,
                },
              }}
            />
          </div>
        </div>

        {/* Output pane */}
        <div className={styles.pane}>
          <div className={styles.paneHeader}>
            <div className={styles.paneHeaderLeft}>
              <Terminal size={13} className={styles.termIcon} />
              <span className={styles.fileName}>output</span>
            </div>
            {hasRun && !loading && (
              <span
                className={`${styles.statusBadge} ${isError ? styles.statusErr : styles.statusOk}`}
              >
                {isError ? "error" : "success"}
              </span>
            )}
          </div>

          <div
            className={`${styles.terminal} ${loading ? styles.terminalLoading : ""}`}
          >
            {loading ? (
              <div className={styles.loadingState}>
                <div className={styles.spinner}>
                  <div className={styles.spinnerRing} />
                </div>
                <div className={styles.loadingText}>
                  <span className={styles.loadingTitle}>
                    Compiling & running
                  </span>
                  <span className={styles.loadingDots}>
                    <span />
                    <span />
                    <span />
                  </span>
                </div>
              </div>
            ) : hasRun ? (
              <pre
                className={`${styles.outputPre} ${isError ? styles.outputErr : styles.outputOk}`}
              >
                {parseAnsi(output).map((span, i) => (
                  <span
                    key={i}
                    style={{
                      color: span.color,
                      backgroundColor: span.bgColor,
                      fontWeight: span.bold ? "bold" : undefined,
                      fontStyle: span.italic ? "italic" : undefined,
                      opacity: span.dim ? 0.6 : undefined,
                      textDecoration: span.underline ? "underline" : undefined,
                    }}
                  >
                    {span.text}
                  </span>
                ))}
              </pre>
            ) : (
              <div className={styles.emptyState}>
                <div className={styles.emptyIcon}>
                  <Play size={18} />
                </div>
                <p className={styles.emptyText}>
                  Hit <kbd>Run</kbd> to execute your code
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
