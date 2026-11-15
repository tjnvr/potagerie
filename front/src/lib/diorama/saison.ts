// Seasonal mood of the diorama (S01 F-VIS-1): the grass and the backdrop
// follow the date, frost in winter, flowers in spring, golden tones in
// autumn. Presentation only, calendar seasons (D01 shows 20 June as summer).

import { moisDe } from "./jours";

export type Saison = "hiver" | "printemps" | "ete" | "automne";

/** Meteorological seasons of the northern hemisphere: Dec-Feb, Mar-May, Jun-Aug, Sep-Nov. */
export function saisonDe(jour: number): Saison {
  const mois = moisDe(jour);
  if (mois === 12 || mois <= 2) return "hiver";
  if (mois <= 5) return "printemps";
  if (mois <= 8) return "ete";
  return "automne";
}

export type Palette = {
  herbe: string;
  herbeGauche: string;
  herbeDroite: string;
  fond: string;
};

export const PALETTES: Record<Saison, Palette> = {
  hiver: { herbe: "#c9dde0", herbeGauche: "#a9c4ca", herbeDroite: "#97b3ba", fond: "#e4edf0" },
  printemps: { herbe: "#a6d68e", herbeGauche: "#86bf74", herbeDroite: "#70ab63", fond: "#e3efdb" },
  ete: { herbe: "#9ccb86", herbeGauche: "#7fb36e", herbeDroite: "#6a9f5e", fond: "#e3ecdd" },
  automne: { herbe: "#cbc27e", herbeGauche: "#b5a662", herbeDroite: "#9d9150", fond: "#f0e9d3" },
};
