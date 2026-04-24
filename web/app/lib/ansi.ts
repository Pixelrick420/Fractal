// lib/ansi.ts
export interface AnsiSpan {
  text: string;
  color?: string;
  bgColor?: string;
  bold?: boolean;
  italic?: boolean;
  dim?: boolean;
  underline?: boolean;
}

const ANSI_COLORS: Record<number, string> = {
  30: "#4a5568",
  31: "#ff7b72",
  32: "#3fb950",
  33: "#d29922",
  34: "#58a6ff",
  35: "#bc8cff",
  36: "#39c5cf",
  37: "#b1bac4",
  90: "#6e7681",
  91: "#ffa198",
  92: "#56d364",
  93: "#e3b341",
  94: "#79c0ff",
  95: "#d2a8ff",
  96: "#76e3ea",
  97: "#f0f6fc",
};

const ANSI_BG: Record<number, string> = {
  40: "#161b22",
  41: "#3d1f1f",
  42: "#1f3d1f",
  43: "#3d3d1f",
  44: "#1f2a3d",
  45: "#2d1f3d",
  46: "#1f3d3d",
  47: "#3d3d3d",
};

export function parseAnsi(raw: string): AnsiSpan[] {
  const spans: AnsiSpan[] = [];
  // Match real ESC byte (\x1b / \u001b) OR the visual symbol (\u241b) that
  // some JSON serialisers / terminals emit in place of the control character.
  const re = /[\x1b\u241b]\[([0-9;]*)m/g;
  let cur: AnsiSpan = { text: "" };
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = re.exec(raw)) !== null) {
    if (match.index > lastIndex) {
      const text = raw.slice(lastIndex, match.index);
      if (text) spans.push({ ...cur, text });
    }
    lastIndex = re.lastIndex;

    const codes = match[1].split(";").map(Number);
    for (const code of codes) {
      if (code === 0) {
        cur = { text: "" };
      } else if (code === 1) {
        cur.bold = true;
      } else if (code === 2) {
        cur.dim = true;
      } else if (code === 3) {
        cur.italic = true;
      } else if (code === 4) {
        cur.underline = true;
      } else if (ANSI_COLORS[code]) {
        cur.color = ANSI_COLORS[code];
      } else if (ANSI_BG[code]) {
        cur.bgColor = ANSI_BG[code];
      }
    }
  }

  if (lastIndex < raw.length) {
    const text = raw.slice(lastIndex);
    if (text) spans.push({ ...cur, text });
  }

  return spans;
}
