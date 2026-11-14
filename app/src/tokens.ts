// Colour tokens for direction A, "survey sheet" (UI_RESEARCH §4-5; ADR-0009 §9). The values
// here are the values on screen: main.ts writes them to CSS custom properties, and
// app/test/tokens.test.ts checks contrast (R3), evening chromaticity (R10) and hue reservation (R1).

export type Tokens = {
  ground: string; // pine-ink sheet
  surface: string; // raised panel
  line: string; // panel rules
  text: string;
  muted: string;
  contour: string; // ochre: grid and benchmark lines only, never text or fills
  corridor: string; // moss: the personal range
  trace: string; // the person's own values
  accent: string; // selection and primary action fill
  onAccent: string;
  advisory: string; // R1 blue (day only)
  caution: string; // R1 yellow
  warning: string; // R1 red
  onSignal: string; // text on caution/warning fills
};

export const day: Tokens = {
  ground: "#0f221c",
  surface: "#16302a",
  line: "#2c4a40",
  text: "#eee8d8",
  muted: "#b9c2ad",
  contour: "#a98a5e",
  corridor: "#5f8a52",
  trace: "#f4efe2",
  accent: "#cfe3b4",
  onAccent: "#0f221c",
  advisory: "#8cc4f0",
  caution: "#f0d84a",
  warning: "#ff8f80",
  onSignal: "#0f221c",
};

/** Pre-sleep rendering: no token bluer than a 2700 K source (sRGB ~ 255,169,87; B/max 0.34). */
export const evening: Tokens = {
  ground: "#0d0904",
  surface: "#1a1209",
  line: "#3a2b14",
  text: "#ffb858",
  muted: "#d39548",
  contour: "#8a6430",
  corridor: "#6f6a1f",
  trace: "#ffc458",
  accent: "#ffb858",
  onAccent: "#0d0904",
  advisory: "#ffb858", // R10 removes blue at night; advisories rely on shape and label (R2)
  caution: "#f0c23a",
  warning: "#ff6a3d",
  onSignal: "#0d0904",
};

/** Increased contrast (HIG accessibility; system "Increase contrast"): secondary text becomes primary
 *  text and rules take the old secondary colour, so every edge and label stands out. Hues are unchanged,
 *  so R1 and R10 still hold. */
export function moreContrast(t: Tokens): Tokens {
  return { ...t, muted: t.text, line: t.muted, contour: t.muted };
}

export function rgb(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

/** WCAG relative luminance and contrast ratio. */
export function contrast(a: string, b: string): number {
  const lum = (h: string) => {
    const [r, g, bl] = rgb(h).map((c) => {
      const s = c / 255;
      return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
    }) as [number, number, number];
    return 0.2126 * r + 0.7152 * g + 0.0722 * bl;
  };
  const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p) as [number, number];
  return (x + 0.05) / (y + 0.05);
}

export function hue(hex: string): number {
  const [r, g, b] = rgb(hex).map((c) => c / 255) as [number, number, number];
  const max = Math.max(r, g, b);
  const d = max - Math.min(r, g, b);
  if (d === 0) return 0;
  const h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return (h * 60 + 360) % 360;
}

export function toCss(t: Tokens): string {
  return Object.entries(t)
    .map(([k, v]) => `--${k.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`)}:${v}`)
    .join(";");
}
