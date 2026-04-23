"use client";
import { useEffect, useState } from "react";
import styles from "./page.module.css";

import {
  AlertTriangle,
  GitBranch,
  Cpu,
  Sparkles,
  AlignLeft,
  SquareTerminal,
  PanelTop,
  Search,
  BookOpen,
  Settings,
  Monitor,
  Package,
  CheckCircle,
  XCircle,
  ArrowRight,
  Play,
  X,
} from "lucide-react";

type Token = { text: string; cls: string };
type CodeLine = Token[];

const SAMPLE_CODE: CodeLine[] = [
  [{ text: "// declare variables with a colon prefix", cls: "cmt" }],
  [{ text: "!start", cls: "kw" }],
  [
    { text: "    ", cls: "" },
    { text: ":int", cls: "type" },
    { text: " age ", cls: "var" },
    { text: "=", cls: "op" },
    { text: " 25", cls: "val" },
    { text: ";", cls: "op" },
  ],
  [
    { text: "    ", cls: "" },
    { text: ":float", cls: "type" },
    { text: " score ", cls: "var" },
    { text: "=", cls: "op" },
    { text: " 9.5", cls: "val" },
    { text: ";", cls: "op" },
  ],
  [
    { text: "    ", cls: "" },
    { text: ":int", cls: "type" },
    { text: " result ", cls: "var" },
    { text: "=", cls: "op" },
    { text: " age ", cls: "var" },
    { text: "+", cls: "op" },
    { text: " :int", cls: "type" },
    { text: "(score)", cls: "op" },
    { text: ";", cls: "op" },
  ],
  [{ text: "!end", cls: "kw" }],
];

function CodeWindow({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className={styles.codeWindow}>
      <div className={styles.codeBar}>
        <div className={styles.dots}>
          <span style={{ background: "#ff5f57" }} />
          <span style={{ background: "#febc2e" }} />
          <span style={{ background: "#28c840" }} />
        </div>
        <span className={styles.codeBarTitle}>{title}</span>
      </div>
      <div className={styles.codeBody}>{children}</div>
    </div>
  );
}

function AnimatedCode() {
  const [visibleLines, setVisibleLines] = useState(0);

  useEffect(() => {
    if (visibleLines >= SAMPLE_CODE.length) return;
    const t = setTimeout(() => setVisibleLines((v) => v + 1), 520);
    return () => clearTimeout(t);
  }, [visibleLines]);

  return (
    <CodeWindow title="example.fr">
      <div className={styles.codeLines}>
        {SAMPLE_CODE.map((line, i) => (
          <div
            key={i}
            className={`${styles.codeLine} ${i < visibleLines ? styles.codeLineVisible : ""}`}
          >
            <span className={styles.lineNum}>{i + 1}</span>
            <span className={styles.lineContent}>
              {line.map((tok, j) => (
                <span
                  key={j}
                  className={tok.cls ? styles[`syn_${tok.cls}`] : ""}
                >
                  {tok.text}
                </span>
              ))}
            </span>
          </div>
        ))}
        {visibleLines < SAMPLE_CODE.length && (
          <span className={styles.cursor}>▋</span>
        )}
      </div>
    </CodeWindow>
  );
}

function CompareBlock() {
  const [active, setActive] = useState<"python" | "c" | "fractal">("python");

  const tabs = {
    python: {
      label: "Python",
      badge: "No types",
      code: `x = 10\ny = "hello"\nz = x + y   # runtime error!`,
      note: "No type safety. Errors only appear when code actually runs.",
      good: false,
    },
    c: {
      label: "C",
      badge: "Hard to read errors",
      code: `int x = 10;\nfloat y = 8.0;\nint z = x + y;\n// error: 'z' undeclared ???`,
      note: "Compiler errors are cryptic. Beginners have no idea what went wrong.",
      good: false,
    },
    fractal: {
      label: "Fractal",
      badge: "Best of both",
      code: `!start\n    :int x = 10;\n    :float y = 8.0;\n    :int z = x + :int(y);\n!end`,
      note: "Strict types like C. Clear errors like a tutor. Syntax you can actually read.",
      good: true,
    },
  };

  const current = tabs[active];

  return (
    <div className={styles.compareBlock}>
      <div className={styles.compareTabs}>
        {(Object.keys(tabs) as (keyof typeof tabs)[]).map((k) => (
          <button
            key={k}
            className={`${styles.compareTab} ${active === k ? styles.compareTabActive : ""}`}
            onClick={() => setActive(k)}
          >
            {tabs[k].label}
            <span
              className={`${styles.compareBadge} ${k === "fractal" ? styles.compareBadgeGood : styles.compareBadgeBad}`}
            >
              {tabs[k].badge}
            </span>
          </button>
        ))}
      </div>
      <div className={styles.compareContent}>
        <pre
          className={`${styles.compareCode} ${current.good ? styles.compareCodeGood : styles.compareCodeNeutral}`}
        >
          {current.code}
        </pre>
        <p
          className={`${styles.compareNote} ${current.good ? styles.compareNoteGood : ""}`}
        >
          {current.good ? (
            <CheckCircle
              size={14}
              style={{
                display: "inline",
                verticalAlign: "middle",
                marginRight: 4,
              }}
            />
          ) : (
            <XCircle
              size={14}
              style={{
                display: "inline",
                verticalAlign: "middle",
                marginRight: 4,
              }}
            />
          )}
          {current.note}
        </p>
      </div>
    </div>
  );
}

function ErrorDemo() {
  const [show, setShow] = useState(false);

  return (
    <div className={styles.errorDemo}>
      <div className={styles.errorDemoTop}>
        <CodeWindow title="mistake.fr">
          <div className={styles.codeLines}>
            <div
              className={`${styles.codeLine} ${styles.codeLineVisible} ${styles.codeLineError}`}
            >
              <span className={styles.lineNum}>1</span>
              <span className={styles.lineContent}>
                <span className={styles.syn_type}>:int </span>
                <span className={styles.syn_var}>age</span>
                <span className={styles.syn_op}> = </span>
                <span className={styles.syn_val}>"hello"</span>
                <span className={styles.syn_op}>;</span>
              </span>
              <span className={styles.errorMark}>
                <X size={13} strokeWidth={2.5} />
              </span>
            </div>
          </div>
        </CodeWindow>
        <button className={styles.runBtn} onClick={() => setShow(true)}>
          <Play
            size={13}
            style={{
              display: "inline",
              verticalAlign: "middle",
              marginRight: 5,
            }}
          />
          Compile
        </button>
      </div>

      <div
        className={`${styles.errorOutput} ${show ? styles.errorOutputVisible : ""}`}
      >
        <div className={styles.errorOutputHeader}>
          <span className={styles.errorDot} />
          Fractal says:
        </div>
        <pre className={styles.errorText}>
          {`✗  1 error(s): Semantic Error: cannot initialise \`age\` (type \`:int\`) with expression of type \`:array<:char, 5>`}
        </pre>
      </div>
    </div>
  );
}

function Pill({ label }: { label: string }) {
  return <span className={styles.pill}>{label}</span>;
}

const REPO = "Pixelrick420/Fractal";

type ReleaseAsset = {
  name: string;
  browser_download_url: string;
  size: number;
};
type Release = {
  tag_name: string;
  published_at: string;
  assets: ReleaseAsset[];
};

function useLatestRelease() {
  const [release, setRelease] = useState<Release | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
      headers: { Accept: "application/vnd.github+json" },
    })
      .then((r) => r.json())
      .then((data) => {
        if (data.tag_name) setRelease(data);
        else setError(true);
      })
      .catch(() => setError(true));
  }, []);

  return { release, error };
}

function formatBytes(bytes: number) {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

type LucideIcon = React.FC<{
  size?: number;
  strokeWidth?: number;
  style?: React.CSSProperties;
}>;

const BINARY_META: Record<string, { Icon: LucideIcon; desc: string }> = {
  "fractal-compiler": {
    Icon: Settings as LucideIcon,
    desc: "Compile .fr files from the command line",
  },
  "fractal-editor": {
    Icon: Monitor as LucideIcon,
    desc: "Full GUI editor with syntax highlighting & terminal",
  },
};

const EDITOR_FEATURES: { Icon: LucideIcon; title: string; desc: string }[] = [
  {
    Icon: Sparkles as LucideIcon,
    title: "Syntax Highlighting",
    desc: "Keywords, types, and values are colour-coded to match the language rules you're learning.",
  },
  {
    Icon: AlignLeft as LucideIcon,
    title: "Auto-Indentation",
    desc: "Code formats itself as you type. Focus on logic, not spacing.",
  },
  {
    Icon: SquareTerminal as LucideIcon,
    title: "Integrated Terminal",
    desc: "Write code and see output in the same window. No switching between apps.",
  },
  {
    Icon: PanelTop as LucideIcon,
    title: "Multi-Tab Editing",
    desc: "Work on multiple files at once with a familiar tab interface.",
  },
  {
    Icon: Search as LucideIcon,
    title: "Search & Replace",
    desc: "Find any symbol, variable or keyword across your file instantly.",
  },
  {
    Icon: BookOpen as LucideIcon,
    title: "Built-in Docs",
    desc: "Language reference is one click away - inside the editor, always.",
  },
];

const PROBLEM_CARDS: {
  Icon: LucideIcon;
  name: string;
  body: string;
  tag: string;
  tagColor: string;
  center: boolean;
}[] = [
  {
    Icon: AlertTriangle as LucideIcon,
    name: "Python",
    body: "No types means no discipline. Beginners never learn to think about what data they're working with - until C forces them to, all at once.",
    tag: "Missing: type discipline",
    tagColor: "var(--syn-keyword)",
    center: false,
  },
  {
    Icon: GitBranch as LucideIcon,
    name: "Fractal",
    body: "Types are visible and required. Errors are friendly and instructive. The syntax maps 1:1 to C. When you're ready to move on, you already speak the language.",
    tag: "The bridge you need",
    tagColor: "var(--accent)",
    center: true,
  },
  {
    Icon: Cpu as LucideIcon,
    name: "C",
    body: "Correct and fast, but its error messages are cryptic, its learning curve is steep, and it offers no help when things go wrong.",
    tag: "Missing: learner support",
    tagColor: "var(--syn-keyword)",
    center: false,
  },
];

/* ── Mobile hamburger ─────────────────────────────────────────────────────── */
function MobileMenuIcon({ open }: { open: boolean }) {
  return (
    <svg
      width="22"
      height="22"
      viewBox="0 0 22 22"
      fill="none"
      aria-hidden="true"
    >
      {open ? (
        <>
          <line
            x1="4"
            y1="4"
            x2="18"
            y2="18"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
          <line
            x1="18"
            y1="4"
            x2="4"
            y2="18"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </>
      ) : (
        <>
          <line
            x1="3"
            y1="6"
            x2="19"
            y2="6"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
          <line
            x1="3"
            y1="11"
            x2="19"
            y2="11"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
          <line
            x1="3"
            y1="16"
            x2="19"
            y2="16"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </>
      )}
    </svg>
  );
}

/* ── Page ─────────────────────────────────────────────────────────────────── */
export default function Home() {
  const [scrolled, setScrolled] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const { release, error } = useLatestRelease();

  useEffect(() => {
    const fn = () => setScrolled(window.scrollY > 50);
    window.addEventListener("scroll", fn);
    return () => window.removeEventListener("scroll", fn);
  }, []);

  const closeMenu = () => setMenuOpen(false);

  return (
    <div className={styles.root}>
      <style>{`
        :root {
          --text-body:    #d4d8e2;
          --text-muted:   #8b93a8;
          --text-heading: #eef0f6;
        }
        p, li, td, th { color: var(--text-body); }
        h1, h2, h3, h4, h5, h6 { color: var(--text-heading); }
        nav[class*="nav"] {
          padding-inline: clamp(1rem, 4vw, 2.5rem) !important;
          display: flex;
          align-items: center;
          justify-content: space-between;
        }
        .fractal-hamburger {
          display: none;
          background: none;
          border: none;
          cursor: pointer;
          color: inherit;
          padding: 6px;
          border-radius: 6px;
          transition: background 0.15s ease;
          -webkit-tap-highlight-color: transparent;
        }
        .fractal-hamburger:hover { background: rgba(255,255,255,0.06); }
        .fractal-mobile-menu {
          display: none;
          position: fixed;
          inset: 0;
          top: 56px;
          background: rgba(10, 11, 16, 0.97);
          backdrop-filter: blur(16px) saturate(180%);
          -webkit-backdrop-filter: blur(16px) saturate(180%);
          z-index: 999;
          flex-direction: column;
          align-items: center;
          justify-content: flex-start;
          padding-top: 2.5rem;
          border-top: 1px solid rgba(255,255,255,0.07);
          opacity: 0;
          transform: translateY(-8px);
          transition: opacity 0.2s ease, transform 0.2s ease;
          pointer-events: none;
        }
        .fractal-mobile-menu.open { opacity: 1; transform: translateY(0); pointer-events: auto; }
        .fractal-mobile-menu a {
          display: block;
          width: 100%;
          max-width: 320px;
          text-align: center;
          padding: 1rem 0;
          font-size: 1.1rem;
          font-weight: 500;
          color: var(--text-body) !important;
          text-decoration: none;
          border-bottom: 1px solid rgba(255,255,255,0.05);
          transition: color 0.15s ease;
        }
        .fractal-mobile-menu a:hover { color: var(--text-heading) !important; }
        .fractal-mobile-menu a.cta {
          margin-top: 1.25rem;
          border-bottom: none;
          background: var(--accent, #5b8af0);
          color: #fff !important;
          border-radius: 8px;
          padding: 0.85rem 2.5rem;
          font-weight: 600;
          box-shadow: 0 4px 20px rgba(91,138,240,0.25);
          transition: opacity 0.15s ease, transform 0.15s ease;
        }
        .fractal-mobile-menu a.cta:hover { opacity: 0.9; transform: translateY(-1px); }
        @media (max-width: 768px) {
          .fractal-hamburger { display: flex; align-items: center; justify-content: center; }
          nav[class*="nav"] > div[class*="navLinks"] { display: none !important; }
          .fractal-mobile-menu { display: flex; }
        }
        *:focus-visible { outline: 2px solid var(--accent, #5b8af0); outline-offset: 3px; border-radius: 4px; }
        html { scroll-behavior: smooth; }
      `}</style>

      {/* ── Navbar ── */}
      <nav className={`${styles.nav} ${scrolled ? styles.navScrolled : ""}`}>
        <a href="/" className={styles.logo}>
          <span className={styles.logoKw}>!</span>fractal
        </a>
        <div className={styles.navLinks}>
          <a href="#why">Why Fractal</a>
          <a href="#syntax">Syntax</a>
          <a href="#errors">Errors</a>
          <a href="#editor">Editor</a>
          <a href="/demo">Demo</a>
          <a href="/docs">Docs</a>
          <a href="#download" className={styles.navCta}>
            Download
          </a>
        </div>
        <button
          className="fractal-hamburger"
          aria-label={menuOpen ? "Close menu" : "Open menu"}
          aria-expanded={menuOpen}
          onClick={() => setMenuOpen((v) => !v)}
        >
          <MobileMenuIcon open={menuOpen} />
        </button>
      </nav>

      {/* ── Mobile drawer ── */}
      <div
        className={`fractal-mobile-menu${menuOpen ? " open" : ""}`}
        role="dialog"
        aria-modal="true"
        aria-label="Navigation menu"
      >
        <a href="#why" onClick={closeMenu}>
          Why Fractal
        </a>
        <a href="#syntax" onClick={closeMenu}>
          Syntax
        </a>
        <a href="#errors" onClick={closeMenu}>
          Errors
        </a>
        <a href="#editor" onClick={closeMenu}>
          Editor
        </a>
        <a href="/demo" onClick={closeMenu}>
          Demo
        </a>
        <a href="/docs" onClick={closeMenu}>
          Docs
        </a>
        <a href="#download" onClick={closeMenu} className="cta">
          Download
        </a>
      </div>

      {/* ── Hero ── */}
      <section className={styles.hero}>
        <div className={styles.heroLeft}>
          <div className={styles.heroEyebrow}>
            <span className={styles.eyebrowDot} />A programming language for
            learners
          </div>
          <h1 className={styles.heroTitle}>
            Learn to think
            <br />
            <span className={styles.heroTitleAccent}>like a programmer.</span>
          </h1>
          <p className={styles.heroSubtitle}>
            Fractal bridges the gap between Python&apos;s friendliness and
            C&apos;s discipline. Strict types, readable syntax, and error
            messages that actually teach you something.
          </p>
          <div className={styles.heroPills}>
            <Pill label="Strictly Typed" />
            <Pill label="C-like Syntax" />
            <Pill label="Human Errors" />
            <Pill label="Built-in Editor" />
          </div>
          <div className={styles.heroCta}>
            <a href="#download" className={styles.btnPrimary}>
              Download Fractal
            </a>
            <a
              href="https://github.com/Pixelrick420/Fractal"
              target="_blank"
              rel="noreferrer"
              className={styles.btnGhost}
            >
              View on GitHub →
            </a>
          </div>
        </div>
        <div className={styles.heroRight}>
          <AnimatedCode />
          <div className={styles.heroCodeNote}>
            <span className={styles.heroCodeNoteKw}>:int</span> and
            <span className={styles.heroCodeNoteKw}> :float</span> - types you
            can&apos;t miss
          </div>
        </div>
      </section>

      {/* ── Why Fractal ── */}
      <section id="why" className={styles.section}>
        <div className={styles.sectionLabel}>// THE PROBLEM</div>
        <h2 className={styles.sectionTitle}>
          Between Python and C,
          <br />
          there&apos;s a gap.
        </h2>
        <p className={styles.sectionSubtitle}>
          Python hides too much. C explains too little. Fractal sits exactly in
          the middle.
        </p>
        <div className={styles.problemGrid}>
          {PROBLEM_CARDS.map(({ Icon, name, body, tag, tagColor, center }) => (
            <div
              key={name}
              className={`${styles.problemCard} ${center ? styles.problemCardCenter : ""}`}
            >
              <div className={styles.problemIcon}>
                <Icon size={28} strokeWidth={1.5} />
              </div>
              <h3>{name}</h3>
              <p>{body}</p>
              <div className={styles.problemTag} style={{ color: tagColor }}>
                {tag}
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ── Syntax ── */}
      <section id="syntax" className={styles.section}>
        <div className={styles.sectionLabel}>// SYNTAX</div>
        <h2 className={styles.sectionTitle}>Built to be readable.</h2>
        <p className={styles.sectionSubtitle}>
          Every keyword stands out visually. You can glance at any line and know
          exactly what&apos;s happening.
        </p>
        <div className={styles.syntaxLayout}>
          <div className={styles.syntaxExplainer}>
            <div className={styles.syntaxRule}>
              <span className={styles.syn_kw}>!start</span> /{" "}
              <span className={styles.syn_kw}>!end</span>
              <p>
                Block delimiters you can&apos;t confuse with anything else. The{" "}
                <code>!</code> prefix flags control keywords.
              </p>
            </div>
            <div className={styles.syntaxRule}>
              <span className={styles.syn_type}>:int</span> &nbsp;
              <span className={styles.syn_type}>:float</span> &nbsp;
              <span className={styles.syn_type}>:string</span>
              <p>
                Types always start with <code>:</code>. You always know what a
                variable holds - no guessing.
              </p>
            </div>
            <div className={styles.syntaxRule}>
              <span className={styles.syn_type}>:int</span>
              <span className={styles.syn_op}>(value)</span>
              <p>
                Explicit casting. Converting types is intentional and visible,
                not silent.
              </p>
            </div>
          </div>
          <div className={styles.syntaxRight}>
            <CompareBlock />
          </div>
        </div>
      </section>

      {/* ── Errors ── */}
      <section id="errors" className={styles.section}>
        <div className={styles.sectionLabel}>// ERROR MESSAGES</div>
        <h2 className={styles.sectionTitle}>Errors that teach, not punish.</h2>
        <p className={styles.sectionSubtitle}>
          When something goes wrong, Fractal explains what happened and shows
          you how to fix it - in plain language.
        </p>
        <div className={styles.errorLayout}>
          <ErrorDemo />
          <div className={styles.errorPoints}>
            {[
              {
                n: "01",
                title: "Plain language first",
                body: "No error codes. No pointer arithmetic jargon. Just a sentence that describes the mistake.",
              },
              {
                n: "02",
                title: "Always a suggestion",
                body: "Every error message ends with at least one concrete fix you can copy and try immediately.",
              },
              {
                n: "03",
                title: "Line and column",
                body: "Errors point exactly to where the problem is - no hunting through your whole file.",
              },
            ].map(({ n, title, body }) => (
              <div key={n} className={styles.errorPoint}>
                <span className={styles.errorPointNum}>{n}</span>
                <div>
                  <strong>{title}</strong>
                  <p>{body}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ── Editor ── */}
      <section id="editor" className={styles.section}>
        <div className={styles.sectionLabel}>// THE EDITOR</div>
        <h2 className={styles.sectionTitle}>Everything in one window.</h2>
        <p className={styles.sectionSubtitle}>
          The Fractal editor is built for beginners. No setup, no extensions to
          install, no configuration files.
        </p>
        <div className={styles.editorFeatures}>
          {EDITOR_FEATURES.map(({ Icon, title, desc }) => (
            <div key={title} className={styles.editorFeatureCard}>
              <span className={styles.editorFeatureIcon}>
                <Icon size={22} strokeWidth={1.5} />
              </span>
              <h3>{title}</h3>
              <p>{desc}</p>
            </div>
          ))}
        </div>
      </section>

      {/* ── Download ── */}
      <section id="download" className={styles.dlSection}>
        <div className={styles.sectionLabel}>// DOWNLOAD</div>
        <h2 className={styles.sectionTitle}>Start writing Fractal today.</h2>
        <p className={styles.sectionSubtitle}>
          One command. No manual steps. The installer handles everything for
          you.
        </p>

        <div className={styles.releaseBadge}>
          {release ? (
            <>
              <span className={styles.releaseDot} />
              <span className={styles.releaseTag}>{release.tag_name}</span>
              <span className={styles.releaseDate}>
                · released{" "}
                {new Date(release.published_at).toLocaleDateString("en-US", {
                  year: "numeric",
                  month: "short",
                  day: "numeric",
                })}
              </span>
            </>
          ) : error ? (
            <span className={styles.releaseErr}>
              Could not fetch release info
            </span>
          ) : (
            <span className={styles.releaseFetching}>
              fetching latest release…
            </span>
          )}
        </div>

        <div className={styles.installBlock}>
          <code className={styles.installCmd}>
            wget -O install.sh
            https://raw.githubusercontent.com/Pixelrick420/Fractal/main/executable/install.sh
            &amp;&amp; sudo bash install.sh
          </code>
          <button
            className={styles.installCopyBtn}
            onClick={() => {
              navigator.clipboard
                .writeText(
                  "wget -O install.sh https://raw.githubusercontent.com/Pixelrick420/Fractal/main/executable/install.sh && sudo bash install.sh",
                )
                .then(() => setCopied(true))
                .catch(() => {});
              setTimeout(() => setCopied(false), 2000);
            }}
          >
            {copied ? (
              <>
                <CheckCircle
                  size={13}
                  style={{
                    display: "inline",
                    verticalAlign: "middle",
                    marginRight: 5,
                  }}
                />
                Copied
              </>
            ) : (
              "Copy"
            )}
          </button>
        </div>

        <p className={styles.installNote}>
          Paste this into your terminal. Requires <code>wget</code> and{" "}
          <code>sudo</code> access.
        </p>

        <a
          href={`https://github.com/${REPO}/releases`}
          target="_blank"
          rel="noreferrer"
          className={styles.allReleases}
        >
          All releases on GitHub →
        </a>
      </section>

      {/* ── Footer ── */}
      <footer className={styles.footer}>
        <div className={styles.footerLeft}>
          <span className={styles.logo}>
            <span className={styles.logoKw}>!</span>fractal
          </span>
          <span className={styles.footerTagline}>
            A beginner-friendly compiled language
          </span>
        </div>
        <div className={styles.footerRight}>
          <a
            href="https://github.com/Pixelrick420/Fractal"
            target="_blank"
            rel="noreferrer"
          >
            GitHub
          </a>
          <a
            href="https://github.com/Pixelrick420/Fractal/blob/main/src/compiler/GRAMMAR.md"
            target="_blank"
            rel="noreferrer"
          >
            Grammar
          </a>
          <a
            href="https://github.com/Pixelrick420/Fractal/releases"
            target="_blank"
            rel="noreferrer"
          >
            Releases
          </a>
        </div>
      </footer>
    </div>
  );
}
