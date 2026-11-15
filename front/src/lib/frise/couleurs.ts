// Colours of the timeline (D01, gen.py: CROPCOL, mix). Plain hex values, not
// CSS variables: the exported SVG has no external CSS (S05 §4), so every colour
// is written into the drawing itself.

/** The palette of D01 (gen.py, C). */
export const PALETTE = {
  fond: "#FFFFFF",
  encre: "#1F3A2E",
  discret: "#56705F",
  trait: "#D3DECD",
  accent: "#F2A65A",
  accentEncre: "#5A3410",
  creneauFond: "#FFF8EF",
  crit: "#C8443C",
  critFond: "#FBE4E2",
  absence: "#EEF2E6",
  selection: "#FDF1E4",
  note: "#3F78B0",
  noteEncre: "#1B3F63",
} as const;

/** One colour per crop of the catalogue (D01). */
const COULEURS_CULTURE: Record<string, string> = {
  tomate: "#E5533D",
  laitue: "#92C65E",
  carotte: "#EF8A2E",
  haricot_vert: "#6DB24D",
  courgette: "#3F8F4A",
  poireau: "#6FB487",
};

/** A crop the catalogue does not know yet still gets a readable colour. */
const COULEUR_PAR_DEFAUT = "#8A9A7B";

export function couleurCulture(id: string): string {
  return COULEURS_CULTURE[id] ?? COULEUR_PAR_DEFAUT;
}

/** Mixes a colour toward white by `t` (0 = unchanged, 1 = white), as D01 does. */
export function melange(hex: string, t: number): string {
  const h = hex.replace("#", "");
  const canaux = [0, 2, 4].map((i) => Number.parseInt(h.slice(i, i + 2), 16));
  const [r, g, b] = canaux.map((v) => Math.round(v + (255 - v) * t));
  const deux = (v: number | undefined) => (v ?? 0).toString(16).padStart(2, "0").toUpperCase();
  return `#${deux(r)}${deux(g)}${deux(b)}`;
}
