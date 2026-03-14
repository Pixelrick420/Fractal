"use client";
import { useEffect, useRef, useState } from "react";
import styles from "./page.module.css";

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

const ERROR_EXAMPLE = {
  bad: `:int age = "hello";`,
  good: ":int age = 25;",
  msg: "Type mismatch — cannot assign string to :int\n→ Did you mean: :int age = 25;",
};

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
          {current.good ? "✓" : "✗"} {current.note}
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
              <span className={styles.errorMark}>✕</span>
            </div>
          </div>
        </CodeWindow>
        <button className={styles.runBtn} onClick={() => setShow(true)}>
          ▶ Compile
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
          {`Error on line 1: Type mismatch
  Cannot assign string to :int variable 'age'

  → Fix: :int age = 25;
  → Or if you want text: :string age = "hello";`}
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

const BINARY_META: Record<string, { icon: string; desc: string }> = {
  "fractal-compiler": {
    icon: "⚙",
    desc: "Compile .fr files from the command line",
  },
  "fractal-editor": {
    icon: "🖥",
    desc: "Full GUI editor with syntax highlighting & terminal",
  },
};

function DownloadCard({ asset }: { asset: ReleaseAsset }) {
  const meta = BINARY_META[asset.name] ?? { icon: "📦", desc: "Binary" };
  return (
    <a href={asset.browser_download_url} className={styles.dlCard} download>
      <span className={styles.dlIcon}>{meta.icon}</span>
      <div className={styles.dlInfo}>
        <span className={styles.dlName}>{asset.name}</span>
        <span className={styles.dlDesc}>{meta.desc}</span>
        <span className={styles.dlSize}>{formatBytes(asset.size)}</span>
      </div>
      <span className={styles.dlArrow}>↓</span>
    </a>
  );
}

function DownloadCardSkeleton() {
  return (
    <div className={styles.dlCardSkeleton}>
      <span className={styles.dlIcon}>⋯</span>
      <div className={styles.dlInfo}>
        <span
          className={`${styles.dlName} ${styles.skeletonBar}`}
          style={{ width: "140px" }}
        />
        <span
          className={`${styles.dlDesc} ${styles.skeletonBar}`}
          style={{ width: "200px" }}
        />
      </div>
    </div>
  );
}

export default function Home() {
  const [scrolled, setScrolled] = useState(false);
  const { release, error } = useLatestRelease();

  useEffect(() => {
    const fn = () => setScrolled(window.scrollY > 50);
    window.addEventListener("scroll", fn);
    return () => window.removeEventListener("scroll", fn);
  }, []);

  return (
    <div className={styles.root}>
      {}
      <nav className={`${styles.nav} ${scrolled ? styles.navScrolled : ""}`}>
        <a href="/" className={styles.logo}>
          <span className={styles.logoKw}>!</span>fractal
        </a>
        <div className={styles.navLinks}>
          <a href="#why">Why Fractal</a>
          <a href="#syntax">Syntax</a>
          <a href="#errors">Errors</a>
          <a href="#editor">Editor</a>
          <a href="#download" className={styles.navCta}>
            Download
          </a>
        </div>
      </nav>

      {}
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
            <span className={styles.heroCodeNoteKw}> :float</span> — types you
            can&apos;t miss
          </div>
        </div>
      </section>

      {}
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
          <div className={styles.problemCard}>
            <div className={styles.problemIcon}>🐍</div>
            <h3>Python</h3>
            <p>
              No types means no discipline. Beginners never learn to think about
              what data they&apos;re working with — until C forces them to, all
              at once.
            </p>
            <div
              className={styles.problemTag}
              style={{ color: "var(--syn-keyword)" }}
            >
              Missing: type discipline
            </div>
          </div>
          <div className={`${styles.problemCard} ${styles.problemCardCenter}`}>
            <div className={styles.problemIcon}>🌉</div>
            <h3>Fractal</h3>
            <p>
              Types are visible and required. Errors are friendly and
              instructive. The syntax maps 1:1 to C. When you&apos;re ready to
              move on, you already speak the language.
            </p>
            <div
              className={styles.problemTag}
              style={{ color: "var(--accent)" }}
            >
              The bridge you need
            </div>
          </div>
          <div className={styles.problemCard}>
            <div className={styles.problemIcon}>⚙️</div>
            <h3>C</h3>
            <p>
              Correct and fast, but its error messages are cryptic, its learning
              curve is steep, and it offers no help when things go wrong.
            </p>
            <div
              className={styles.problemTag}
              style={{ color: "var(--syn-keyword)" }}
            >
              Missing: learner support
            </div>
          </div>
        </div>
      </section>

      {}
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
                variable holds — no guessing.
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

      {}
      <section id="errors" className={styles.section}>
        <div className={styles.sectionLabel}>// ERROR MESSAGES</div>
        <h2 className={styles.sectionTitle}>Errors that teach, not punish.</h2>
        <p className={styles.sectionSubtitle}>
          When something goes wrong, Fractal explains what happened and shows
          you how to fix it — in plain language.
        </p>

        <div className={styles.errorLayout}>
          <ErrorDemo />
          <div className={styles.errorPoints}>
            <div className={styles.errorPoint}>
              <span className={styles.errorPointNum}>01</span>
              <div>
                <strong>Plain language first</strong>
                <p>
                  No error codes. No pointer arithmetic jargon. Just a sentence
                  that describes the mistake.
                </p>
              </div>
            </div>
            <div className={styles.errorPoint}>
              <span className={styles.errorPointNum}>02</span>
              <div>
                <strong>Always a suggestion</strong>
                <p>
                  Every error message ends with at least one concrete fix you
                  can copy and try immediately.
                </p>
              </div>
            </div>
            <div className={styles.errorPoint}>
              <span className={styles.errorPointNum}>03</span>
              <div>
                <strong>Line and column</strong>
                <p>
                  Errors point exactly to where the problem is — no hunting
                  through your whole file.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {}
      <section id="editor" className={styles.section}>
        <div className={styles.sectionLabel}>// THE EDITOR</div>
        <h2 className={styles.sectionTitle}>Everything in one window.</h2>
        <p className={styles.sectionSubtitle}>
          The Fractal editor is built for beginners. No setup, no extensions to
          install, no configuration files.
        </p>

        <div className={styles.editorFeatures}>
          {[
            {
              icon: "✦",
              title: "Syntax Highlighting",
              desc: "Keywords, types, and values are colour-coded to match the language rules you're learning.",
            },
            {
              icon: "⇥",
              title: "Auto-Indentation",
              desc: "Code formats itself as you type. Focus on logic, not spacing.",
            },
            {
              icon: "⬛",
              title: "Integrated Terminal",
              desc: "Write code and see output in the same window. No switching between apps.",
            },
            {
              icon: "📑",
              title: "Multi-Tab Editing",
              desc: "Work on multiple files at once with a familiar tab interface.",
            },
            {
              icon: "🔍",
              title: "Search & Replace",
              desc: "Find any symbol, variable or keyword across your file instantly.",
            },
            {
              icon: "📖",
              title: "Built-in Docs",
              desc: "Language reference is one click away — inside the editor, always.",
            },
          ].map((f) => (
            <div key={f.title} className={styles.editorFeatureCard}>
              <span className={styles.editorFeatureIcon}>{f.icon}</span>
              <h3>{f.title}</h3>
              <p>{f.desc}</p>
            </div>
          ))}
        </div>
      </section>

      {}
      <section id="download" className={styles.dlSection}>
        <div className={styles.sectionLabel}>// DOWNLOAD</div>
        <h2 className={styles.sectionTitle}>Start writing Fractal today.</h2>
        <p className={styles.sectionSubtitle}>
          Two binaries. No installer. Just download, make executable, and run.
        </p>

        {}
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

        <div className={styles.dlCards}>
          {release ? (
            release.assets
              .filter(
                (a) =>
                  a.name === "fractal-compiler" || a.name === "fractal-editor",
              )
              .map((a) => <DownloadCard key={a.name} asset={a} />)
          ) : error ? (
            <a
              href={`https://github.com/${REPO}/releases/latest`}
              target="_blank"
              rel="noreferrer"
              className={styles.dlFallback}
            >
              Open latest release on GitHub →
            </a>
          ) : (
            <>
              <DownloadCardSkeleton />
              <DownloadCardSkeleton />
            </>
          )}
        </div>

        <div className={styles.dlSteps}>
          <div className={styles.dlStep}>
            <span className={styles.dlStepN}>1</span>
            <div>
              <strong>Download both files above</strong>
              <code>fractal-compiler &nbsp; fractal-editor</code>
            </div>
          </div>
          <div className={styles.dlStepArrow}>→</div>
          <div className={styles.dlStep}>
            <span className={styles.dlStepN}>2</span>
            <div>
              <strong>Make them executable</strong>
              <code>chmod +x fractal-compiler fractal-editor</code>
            </div>
          </div>
          <div className={styles.dlStepArrow}>→</div>
          <div className={styles.dlStep}>
            <span className={styles.dlStepN}>3</span>
            <div>
              <strong>Launch the editor</strong>
              <code>./fractal-editor</code>
            </div>
          </div>
        </div>

        <a
          href={`https://github.com/${REPO}/releases`}
          target="_blank"
          rel="noreferrer"
          className={styles.allReleases}
        >
          All releases on GitHub →
        </a>
      </section>

      {}
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
