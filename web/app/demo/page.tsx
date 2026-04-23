"use client";
import { useState, useRef, useCallback } from "react";
import Editor, { OnMount } from "@monaco-editor/react";
import { Play, RotateCcw, Copy, Check } from "lucide-react";
import Navbar from "../../components/Navbar";
import styles from "./demo.module.css";

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

    if (t.startsWith("}")) {
      depth = Math.max(0, depth - 1);
    }

    output.push("    ".repeat(depth) + t);

    const opens = (t.match(/\{/g) || []).length;
    const closes = (t.match(/\}/g) || []).length;
    depth += opens - closes;
  }

  return output.join("\n");
}

export default function DemoPage() {
  const [code, setCode] = useState(INITIAL_CODE);
  const [output, setOutput] = useState("");
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const editorRef = useRef<any>(null);
  const monacoRef = useRef<any>(null);

  const handleEditorMount: OnMount = useCallback((editor, monaco) => {
    editorRef.current = editor;
    monacoRef.current = monaco;

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
        "editor.background": "#0e1117",
        "editor.foreground": "#e2e8f6",
        "editor.lineHighlightBackground": "#121723",
        "editorCursor.foreground": "#15ce43",
        "editor.selectionBackground": "#252d4480",
      },
    });

    monaco.editor.setTheme("fractal-dark");

    // Disable drag and drop to prevent Monaco issues
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
    try {
      const response = await fetch("/api/run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ code: formatted }),
      });
      const data = await response.json();
      if (data.error) {
        setOutput(`Error: ${data.error}`);
      } else {
        setOutput(data.output || "(no output)");
      }
    } catch {
      setOutput("Error: Failed to run code");
    }
    setLoading(false);
  };

  const handleReset = () => {
    setCode(INITIAL_CODE);
    setOutput("");
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={styles.root}>
      <Navbar />

      <div className={styles.container}>
        <div className={styles.editorPane}>
          <div className={styles.editorHeader}>
            <span className={styles.fileName}>demo.fr</span>
            <div className={styles.editorActions}>
              <button
                className={styles.iconBtn}
                onClick={handleCopy}
                title="Copy code"
              >
                {copied ? <Check size={14} /> : <Copy size={14} />}
              </button>
              <button
                className={styles.iconBtn}
                onClick={handleReset}
                title="Reset code"
              >
                <RotateCcw size={14} />
              </button>
              <button
                className={styles.runBtn}
                onClick={handleRun}
                disabled={loading}
              >
                <Play size={14} />
                {loading ? "Running..." : "Run"}
              </button>
            </div>
          </div>

          <div className={styles.editorContent}>
            <Editor
              height="100%"
              defaultLanguage="fractal"
              value={code}
              onChange={(value) => setCode(value || "")}
              onMount={handleEditorMount}
              loading={<div className={styles.loading}>Loading editor...</div>}
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                fontFamily: '"DM Mono", monospace',
                lineNumbers: "on",
                lineHeight: 24,
                padding: { top: 16, bottom: 16 },
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
              }}
            />
          </div>
        </div>

        <div className={styles.outputPane}>
          <div className={styles.outputHeader}>
            <span className={styles.outputTitle}>Output</span>
          </div>
          <pre className={styles.outputContent}>
            {output || "Click Run to execute your code..."}
          </pre>
        </div>
      </div>
    </div>
  );
}
