// What the export screen (E11) decides before drawing: the default period,
// whether the chosen one can be exported, and the scale of the drawing.
// The drawing itself comes from `TimelineSvg`, the business from Rust.
import { yearOf, dateOf, dayOf } from "../dates";
import { DAYS_MAX, type Scale } from "./geometry";

/** Pixels of the exported drawing for a year: wide enough to read once printed on A3. */
const WIDTH_TARGET_PX = 1800;
const PX_BY_DAY_MIN = 1.5;
const PX_BY_DAY_MAX = 8;

/** The civil year of the cursor, the default of the period (S01 F-EXP-1). */
export function periodByDefault(cursor: string): { from: string; to: string } {
  const year = yearOf(cursor);
  return { from: `${year}-01-01`, to: `${year}-12-31` };
}

/** The period the timeline shows on screen: every day at least partly
 *  visible, at most the 3 years an export allows (S01 F-EXP-1). */
export function periodVisible(viewport: { startDay: number; days: number }): { from: string; to: string } {
  const start = Math.floor(viewport.startDay);
  const end = Math.min(Math.ceil(viewport.startDay + viewport.days) - 1, start + DAYS_MAX - 1);
  return { from: dateOf(start), to: dateOf(Math.max(end, start)) };
}

export type ProblemPeriod = "empty" | "reversed" | "too_long";

/** Why a period cannot be exported, or `null`. At most 3 years, as the timeline. */
export function problemPeriod(from: string, to: string): ProblemPeriod | null {
  if (!from || !to) return "empty";
  if (to < from) return "reversed";
  if (dayOf(to) - dayOf(from) + 1 > DAYS_MAX) return "too_long";
  return null;
}

/** The scale that draws `from` to `to` (included) in a fixed width. */
export function scaleForPeriod(from: string, to: string): Scale {
  const days = dayOf(to) - dayOf(from) + 1;
  const pxByDay = Math.min(PX_BY_DAY_MAX, Math.max(PX_BY_DAY_MIN, WIDTH_TARGET_PX / days));
  return { startDay: dayOf(from), pxByDay, width: days * pxByDay };
}

/** The years a period touches, to load their views. */
export function yearsOfPeriod(from: string, to: string): number[] {
  const start = yearOf(from);
  return Array.from({ length: yearOf(to) - start + 1 }, (_, i) => start + i);
}
