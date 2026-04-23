import { NextResponse } from "next/server";
import { unlink } from "fs/promises";
import { join } from "path";

export async function GET() {
  const compilerPath = join("/tmp/fractal_cache", "fractal-compiler");
  try {
    await unlink(compilerPath);
    return NextResponse.json({ ok: true, message: "Compiler cache cleared" });
  } catch (err: any) {
    return NextResponse.json({ ok: false, message: err.message });
  }
}
