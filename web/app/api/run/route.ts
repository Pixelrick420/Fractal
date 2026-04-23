import { NextRequest, NextResponse } from "next/server";
/*turbopack-ignore: true*/
import { writeFile, unlink, mkdir, access } from "fs/promises";
/*turbopack-ignore: true*/
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
    
    const compilerPath = join/*turbopack-ignore: true*/(cacheDir, COMPILER_NAME);
  
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
  
  const release = await response.json() as { 
    assets?: Array<{ name: string; browser_download_url: string }> 
  };
  
  const asset = release.assets?.find(a => a.name === COMPILER_NAME);
  
  if (!asset) {
    throw new Error("Compiler not found in release");
  }
  
  const binaryUrl = asset.browser_download_url;
  const binaryResp = await fetchImpl(binaryUrl);
  
  if (!binaryResp.ok) {
    throw new Error(`Failed to download compiler: ${binaryResp.status}`);
  }
  
  const arrayBuffer = await binaryResp.arrayBuffer();
  const buffer = Buffer.from(arrayBuffer);
  
  await writeFile(compilerPath, buffer, { mode: 0o755 });
  
  return compilerPath;
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
    return NextResponse.json({ 
      output: "", 
      compiled: false,
      error: err.message 
    }, { status: 500 });
  }

  const tempDir = "/tmp";
  const timestamp = Date.now();
  const tempFile = join(tempDir, `fractal_demo_${timestamp}.fr`);
  
  try {
    await writeFile(tempFile, code);
  } catch {
    return NextResponse.json({ 
      output: "", 
      compiled: false,
      error: "Cannot create temp file" 
    }, { status: 500 });
  }
  
  return new Promise<NextResponse>((resolve) => {
    const compileProc = spawn(compilerPath, [tempFile], {
      stdio: ["pipe", "pipe", "pipe"],
    });
    
    let compileStdout = "";
    let compileStderr = "";
    
    compileProc.stdout?.on("data", (data) => {
      compileStdout += data.toString();
    });
    
    compileProc.stderr?.on("data", (data) => {
      compileStderr += data.toString();
    });
    
    compileProc.on("close", async (exitCode) => {
      try {
        await unlink(tempFile).catch(() => {});
      } catch {}
      
      const fullOutput = compileStdout + compileStderr;
      
      if (exitCode !== 0 || fullOutput.includes("error") || fullOutput.includes("Error")) {
        const match = fullOutput.match(/✗.*$/m) || fullOutput.match(/error.*$/mi);
        return resolve(NextResponse.json({ 
          output: "",
          compiled: false,
          error: match ? match[0].trim() : `Compilation failed (exit code ${exitCode})` 
        }, { status: 200 }));
      }
      
      const execFile = join(tempDir, `fractal_demo_${timestamp}`);
      
      try {
        const runProc = spawn(execFile, [], {
          stdio: ["pipe", "pipe", "pipe"],
        });
        
        let runStdout = "";
        let runStderr = "";
        
        runProc.stdout?.on("data", (data) => {
          runStdout += data.toString();
        });
        
        runProc.stderr?.on("data", (data) => {
          runStderr += data.toString();
        });
        
        runProc.on("close", async () => {
          try {
            await unlink(execFile).catch(() => {});
          } catch {}
          
          if (runStderr && !runStderr.includes("Compiling")) {
            return resolve(NextResponse.json({ 
              output: "",
              compiled: true,
              error: runStderr 
            }, { status: 200 }));
          }
          
          resolve(NextResponse.json({ 
            output: runStdout.trim(),
            compiled: true,
            error: null
          }, { status: 200 }));
        });
        
        runProc.on("error", async (err) => {
          try {
            await unlink(execFile).catch(() => {});
          } catch {}
          resolve(NextResponse.json({ 
            output: "",
            compiled: true,
            error: err.message 
          }, { status: 200 }));
        });
        
        setTimeout(() => {
          runProc.kill();
          resolve(NextResponse.json({ 
            output: "",
            compiled: true,
            error: "Execution timed out (5s limit)" 
          }, { status: 200 }));
        }, 5000);
        
      } catch (err: any) {
        resolve(NextResponse.json({ 
          output: "",
          compiled: true,
          error: err.message 
        }, { status: 200 }));
      }
    });
    
    compileProc.on("error", async (err) => {
      try {
        await unlink(tempFile).catch(() => {});
      } catch {}
      resolve(NextResponse.json({ 
        output: "",
        compiled: false,
        error: err.message 
      }, { status: 200 }));
    });
    
  });
}