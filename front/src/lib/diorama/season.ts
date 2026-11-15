// Seasonal mood of the diorama (S01 F-VIS-1): the grass and the backdrop
// follow the date, frost in winter, flowers in spring, golden tones in
// autumn. Presentation only, calendar seasons (D01 shows 20 June as summer).

import { monthOf } from "./days";

export type Season = "winter" | "spring" | "summer" | "autumn";

/** Meteorological seasons of the northern hemisphere: Dec-Feb, Mar-May, Jun-Aug, Sep-Nov. */
export function seasonOf(day: number): Season {
  const month = monthOf(day);
  if (month === 12 || month <= 2) return "winter";
  if (month <= 5) return "spring";
  if (month <= 8) return "summer";
  return "autumn";
}

export type Palette = {
  grass: string;
  grassGauche: string;
  grassRight: string;
  fond: string;
};

export const PALETTES: Record<Season, Palette> = {
  winter: { grass: "#c9dde0", grassGauche: "#a9c4ca", grassRight: "#97b3ba", fond: "#e4edf0" },
  spring: { grass: "#a6d68e", grassGauche: "#86bf74", grassRight: "#70ab63", fond: "#e3efdb" },
  summer: { grass: "#9ccb86", grassGauche: "#7fb36e", grassRight: "#6a9f5e", fond: "#e3ecdd" },
  autumn: { grass: "#cbc27e", grassGauche: "#b5a662", grassRight: "#9d9150", fond: "#f0e9d3" },
};
