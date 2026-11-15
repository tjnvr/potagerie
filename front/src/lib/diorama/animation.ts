// What a planting looks like at a date, and how the season plays.
//
// The stages and their dates come from the Rust view (`appearance_changes`,
// S03 §3). Nothing here knows a threshold: the only calculation is the
// interpolation of the animation (scale and bounce after a change) and the
// advance of the playback.

import { drawingOf } from "../appearances";
import type { Appearance, AppearanceChange } from "../bindings";
import { daySince } from "./days";

/** The changes of one planting with their dates as day numbers. */
export type Chronology = { days: number[]; appearances: Appearance[] };

export function chronology(changes: readonly AppearanceChange[]): Chronology {
  return {
    days: changes.map((c) => daySince(c.from)),
    appearances: changes.map((c) => c.appearance),
  };
}

/** Index of the last change that has happened on day `day` (a fraction is
 *  dropped), -1 before the first. */
export function indexTo(chrono: Chronology, day: number): number {
  const j = Math.floor(day);
  let low = 0;
  let top = chrono.days.length;
  while (low < top) {
    const milieu = (low + top) >> 1;
    if ((chrono.days[milieu] ?? 0) <= j) low = milieu + 1;
    else top = milieu;
  }
  return low - 1;
}

export function apparenceTo(chrono: Chronology, day: number): Appearance {
  const i = indexTo(chrono, day);
  return chrono.appearances[i] ?? { appearance: "nothing" };
}

/** The symbol and size of a planting on a day. `symbol` is null when nothing is drawn. */
export type State = { symbol: string | null; scale: number };

export const SYMBOL_ETIQUETTE = "etiquette";
const NOTHING: State = { symbol: null, scale: 1 };

/** Id of the symbol of a crop at a stage: `plant-tomato-adult`. */
export function symbolPlant(crop: string, stade: string): string {
  return `plant-${drawingOf(crop)}-${stade}`;
}

/** Days a plant takes to settle at its new size after a change. */
export const DURATION_GROWTH_DAYS = 10;

/** Ease out with a slight overshoot: the little bounce of the growth. */
export function bounce(t: number): number {
  if (t <= 0) return 0;
  if (t >= 1) return 1;
  const c1 = 1.70158;
  const c3 = c1 + 1;
  return 1 + c3 * Math.pow(t - 1, 3) + c1 * Math.pow(t - 1, 2);
}

/**
 * What to draw for `crop` on day `day`. With `animate`, a plant that has
 * just come out of the ground or changed stage grows to its size with a bounce;
 * without, it is at its size at once (scrubbing, reduced motion).
 */
export function stateTo(chrono: Chronology, crop: string, day: number, animate: boolean): State {
  const i = indexTo(chrono, day);
  if (i < 0) return NOTHING;
  const apparence = chrono.appearances[i];
  if (!apparence || apparence.appearance === "nothing") return NOTHING;
  if (apparence.appearance === "label") return { symbol: SYMBOL_ETIQUETTE, scale: 1 };
  const symbol = symbolPlant(crop, apparence.stage);
  if (!animate) return { symbol, scale: 1 };
  const t = (day - (chrono.days[i] ?? 0)) / DURATION_GROWTH_DAYS;
  if (t >= 1) return { symbol, scale: 1 };
  const before = chrono.appearances[i - 1]?.appearance ?? "nothing";
  // Out of the ground (or off the label) from small; from a stage to the next, a smaller step.
  const depart = before === "plant" ? 0.75 : 0.2;
  return { symbol, scale: depart + (1 - depart) * bounce(t) };
}

/** « Play la season »: about 15 s for a year (S01 F-VIS-4). */
export const DURATION_SEASON_S = 15;
export const DAYS_BY_SECONDE = 365 / DURATION_SEASON_S;
/** With reduced motion the season advances by steps of a week, no transition. */
export const PALIER_DAYS = 7;

export type EntreePlayback = {
  /** Day the playback (re)started from. */
  start: number;
  /** Time since then, in ms. */
  elapsedMs: number;
  /** 1 or 2. */
  speed: number;
  /** Last day of the season: the playback stops there. */
  end: number;
  reduced: boolean;
};

/** The day of the playback after `elapsedMs`, and whether it has reached the end. */
export function dayOfPlayback(e: EntreePlayback): { day: number; termine: boolean } {
  let advance = (e.elapsedMs / 1000) * DAYS_BY_SECONDE * e.speed;
  if (e.reduced) advance = Math.floor(advance / PALIER_DAYS) * PALIER_DAYS;
  const day = e.start + advance;
  return day >= e.end ? { day: e.end, termine: true } : { day, termine: false };
}
