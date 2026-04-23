import { NextRequest, NextResponse } from "next/server";
import { writeFile, unlink, mkdir, access } from "fs/promises";
import { join } from "path";
import { spawn } from "child_process";
import { fetch as fetchImpl } from "undici";

const REPO = "Pixelrick420/Fractal";
const CACHE_DIR = "/tmp/fractal_cache";
const COMPILER_NAME = "fractal-compiler";

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
      {
        output: "",
        compiled: false,
        error: err.message,
      },
      { status: 500 },
    );
  }

  const tempFile = join("/tmp", `fractal_demo_${Date.now()}.fr`);

  try {
    await writeFile(tempFile, code);
  } catch {
    return NextResponse.json(
      {
        output: "",
        compiled: false,
        error: "Cannot create temp file",
      },
      { status: 500 },
    );
  }

  const {
    stdout: rustSource,
    stderr: compileStderr,
    code: exitCode,
  } = await runProcess(compilerPath, ["--emit-rust", tempFile]);

  await unlink(tempFile).catch(() => {});

  if (exitCode !== 0 || !rustSource.trim()) {
    const match =
      compileStderr.match(/✗.*$/m) || compileStderr.match(/error.*$/im);
    return NextResponse.json({
      output: "",
      compiled: false,
      error: match
        ? match[0].trim()
        : compileStderr.trim() || `Compilation failed (exit ${exitCode})`,
    });
  }

  // Step 2: send Rust source to Wandbox to compile and run
  try {
    const wandboxResp = await fetchImpl(
      "https://wandbox.org/api/compile.json",
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          compiler: "rust-head",
          code: rustSource,
          options: "",
          stdin: "",
        }),
      },
    );

    if (!wandboxResp.ok) {
      throw new Error(`Wandbox returned ${wandboxResp.status}`);
    }

    const result = (await wandboxResp.json()) as {
      program_output?: string;
      program_error?: string;
      compiler_error?: string;
      status?: string;
    };

    if (result.compiler_error) {
      return NextResponse.json({
        output: "",
        compiled: false,
        error: result.compiler_error,
      });
    }

    return NextResponse.json({
      output: result.program_output?.trim() || "(no output)",
      compiled: true,
      error: result.program_error || null,
    });
  } catch (err: any) {
    return NextResponse.json({
      output: "",
      compiled: false,
      error: `Runner error: ${err.message}`,
    });
  }
}
