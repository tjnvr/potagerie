// Colours of the timeline (D01, gen.py: CROPCOL, mix). Plain hex values, not
// CSS variables: the exported SVG has no external CSS (S05 §4), so every colour
// is written into the drawing itself.

import { APPEARANCES } from "../appearances";

/** The palette of D01 (gen.py, C). */
export const PALETTE = {
  fond: "#FFFFFF",
  ink: "#1F3A2E",
  muted: "#4e6858",
  trait: "#D3DECD",
  accent: "#F2A65A",
  accentInk: "#5A3410",
  slotFond: "#FFF8EF",
  crit: "#C8443C",
  critFond: "#FBE4E2",
  absence: "#EEF2E6",
  selection: "#FDF1E4",
  note: "#3F78B0",
  noteInk: "#1B3F63",
} as const;

/** One colour per crop of the catalogue (D01). */
const COLORS_CROP: Record<string, string> = {
  tomato: "#E5533D",
  lettuce: "#92C65E",
  carrot: "#EF8A2E",
  green_bean: "#6DB24D",
  zucchini: "#3F8F4A",
  leek: "#6FB487",
};

/** A crop the catalogue does not know yet still gets a readable colour. */
const COLOR_BY_DEFAULT = "#8A9A7B";

export function colorCrop(id: string): string {
  const key = id.replaceAll("-", "_");
  return COLORS_CROP[key] ?? APPEARANCES[id.replaceAll("_", "-")]?.color ?? COLOR_BY_DEFAULT;
}

/** Mixes a colour toward white by `t` (0 = unchanged, 1 = white), as D01 does. */
export function melange(hex: string, t: number): string {
  const h = hex.replace("#", "");
  const channels = [0, 2, 4].map((i) => Number.parseInt(h.slice(i, i + 2), 16));
  const [r, g, b] = channels.map((v) => Math.round(v + (255 - v) * t));
  const two = (v: number | undefined) => (v ?? 0).toString(16).padStart(2, "0").toUpperCase();
  return `#${two(r)}${two(g)}${two(b)}`;
}
