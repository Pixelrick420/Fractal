import { NextRequest, NextResponse } from "next/server";
import { writeFile, unlink, mkdir, access } from "fs/promises";
import { join } from "path";
import { spawn } from "child_process";
import { fetch as fetchImpl } from "undici";

const REPO = "Pixelrick420/Fractal";
const CACHE_DIR = "/tmp/fractal_cache";
const COMPILER_NAME = "fractal-compiler";

// Strip ANSI escape codes from a string.
function stripAnsi(str: string): string {
  return str.replace(/[\x1b\u241b]\[[0-9;]*[a-zA-Z]/g, "");
}

// The Fractal compiler may prepend warning/diagnostic lines (with ANSI codes)
// to stdout before the actual Rust source. Drop everything before the first
// real Rust line so Godbolt doesn't choke on them.
function extractRustSource(raw: string): string {
  const cleaned = stripAnsi(raw);
  const lines = cleaned.split("\n");
  const firstCodeLine = lines.findIndex((l) =>
    /^\s*(fn |use |struct |enum |impl |const |static |pub |#\[|\/\/)/.test(l),
  );
  return firstCodeLine >= 0 ? lines.slice(firstCodeLine).join("\n") : cleaned;
}

async function ensureCompiler() {
  const cacheDir = CACHE_DIR;

  try {
    await access(cacheDir);
  } catch {
    await mkdir(cacheDir, { recursive: true });
  }

  const compilerPath = join(/*turbopack-ignore: true*/ cacheDir, COMPILER_NAME);

  try {
    await access(compilerPath);
    return compilerPath;
  } catch {}

  const releaseUrl = `https://api.github.com/repos/${REPO}/releases/latest`;
  const response = await fetchImpl(releaseUrl, {
    headers: { Accept: "application/vnd.github+json" },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch release: ${response.status}`);
  }

  const release = (await response.json()) as {
    assets?: Array<{ name: string; browser_download_url: string }>;
  };

  const asset = release.assets?.find((a) => a.name === COMPILER_NAME);

  if (!asset) {
    throw new Error("Compiler not found in release");
  }

  const binaryResp = await fetchImpl(asset.browser_download_url);

  if (!binaryResp.ok) {
    throw new Error(`Failed to download compiler: ${binaryResp.status}`);
  }

  const buffer = Buffer.from(await binaryResp.arrayBuffer());
  await writeFile(compilerPath, buffer, { mode: 0o755 });

  return compilerPath;
}

function runProcess(
  bin: string,
  args: string[],
): Promise<{ stdout: string; stderr: string; code: number }> {
  return new Promise((resolve) => {
    const proc = spawn(bin, args, { stdio: ["pipe", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    proc.stdout?.on("data", (d) => {
      stdout += d.toString();
    });
    proc.stderr?.on("data", (d) => {
      stderr += d.toString();
    });
    proc.on("close", (code) => resolve({ stdout, stderr, code: code ?? 1 }));
    proc.on("error", (e) => resolve({ stdout, stderr: e.message, code: 1 }));
  });
}

export async function POST(request: NextRequest) {
  const { code } = await request.json();

  if (!code || typeof code !== "string") {
    return NextResponse.json({ error: "No code provided" }, { status: 400 });
  }

  let compilerPath: string;

  try {
    compilerPath = await ensureCompiler();
  } catch (err: any) {
    return NextResponse.json(
      { output: "", compiled: false, error: err.message },
      { status: 500 },
    );
  }

  const tempFile = join("/tmp", `fractal_demo_${Date.now()}.fr`);

  try {
    await writeFile(tempFile, code);
  } catch {
    return NextResponse.json(
      { output: "", compiled: false, error: "Cannot create temp file" },
      { status: 500 },
    );
  }

  const {
    stdout: rawStdout,
    stderr: compileStderr,
    code: exitCode,
  } = await runProcess(compilerPath, ["--emit-rust", tempFile]);

  await unlink(tempFile).catch(() => {});

  if (exitCode !== 0 || !rawStdout.trim()) {
    // Return stderr with ANSI intact — the browser terminal renders colors.
    return NextResponse.json({
      output: "",
      compiled: false,
      error: compileStderr.trim() || `Compilation failed (exit ${exitCode})`,
    });
  }

  // Warnings land on stderr with ANSI codes — keep them to show in the
  // browser terminal, but they must NOT go to Godbolt.
  const warnings = compileStderr.trim();

  // Strip ANSI and any diagnostic preamble the compiler wrote to stdout
  // before the Rust source begins.
  const cleanRustSource = extractRustSource(rawStdout);

  try {
    const godboltResp = await fetchImpl(
      "https://godbolt.org/api/compiler/r1880/compile",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Accept: "application/json",
        },
        body: JSON.stringify({
          source: cleanRustSource,
          compiler: "r1880",
          options: {
            userArguments: "",
            executeParameters: {
              args: "",
              stdin: "",
            },
            compilerOptions: {},
            filters: {
              execute: true,
            },
            tools: [],
            libraries: [],
          },
          lang: "rust",
          allowStoreCodeDebug: false,
        }),
      },
    );

    if (!godboltResp.ok) {
      throw new Error(`Godbolt returned ${godboltResp.status}`);
    }

    const result = (await godboltResp.json()) as {
      stdout?: Array<{ text: string }>;
      stderr?: Array<{ text: string }>;
      execResult?: {
        stdout?: Array<{ text: string }>;
        stderr?: Array<{ text: string }>;
        code?: number;
      };
      code?: number;
    };

    const stdout = (result.execResult?.stdout ?? result.stdout ?? [])
      .map((l) => l.text)
      .join("\n")
      .trim();

    const buildErr = (result.stderr ?? [])
      .map((l) => l.text)
      .join("\n")
      .trim();

    if (buildErr && result.code !== 0) {
      return NextResponse.json({
        output: "",
        compiled: false,
        error: buildErr,
      });
    }

    // Prepend Fractal compiler warnings (with ANSI) above the program output.
    const finalOutput = warnings
      ? `${warnings}\n${stdout || "(no output)"}`
      : stdout || "(no output)";

    return NextResponse.json({
      output: finalOutput,
      compiled: true,
      error: null,
    });
  } catch (err: any) {
    return NextResponse.json({
      output: "",
      compiled: false,
      error: `Runner error: ${err.message}`,
    });
  }
}
