import { localeTag } from "./locale";
// Civil dates `YYYY-MM-DD` (S03 §1) as day numbers, for the layout of the
// timeline only: pixels to days and back. No rule of S03 lives here (no
// duration, no conflict, no slot): every date that matters comes from Rust.

const MS_BY_DAY = 86_400_000;

/** Days since 1970-01-01 of a civil date. Never goes through a time zone. */
export function dayOf(iso: string): number {
  const [a = 1970, m = 1, j = 1] = iso.split("-").map(Number);
  return Math.round(utcMs(a, m, j) / MS_BY_DAY);
}

/** Milliseconds of a civil date at 00:00 UTC. Unlike `Date.UTC`, years 0 to 99
 *  stay themselves instead of becoming 1900 to 1999. */
function utcMs(year: number, month: number, day: number, hour = 0): number {
  const d = new Date(0);
  d.setUTCFullYear(year, month - 1, day);
  d.setUTCHours(hour);
  return d.getTime();
}

/** The civil date of a day number. */
export function dateOf(day: number): string {
  return new Date(day * MS_BY_DAY).toISOString().slice(0, 10);
}

export function yearOf(iso: string): number {
  return Number(iso.slice(0, 4));
}

/** January 1st of `year`, as a day number. */
export function premierJanuary(year: number): number {
  return dayOf(`${String(year).padStart(4, "0")}-01-01`);
}

/** The local date of the device (S03 §1: « aujourd'hui » is the local date). */
export function todayLocal(now: Date = new Date()): string {
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const j = String(now.getDate()).padStart(2, "0");
  return `${now.getFullYear()}-${m}-${j}`;
}

/** Milliseconds from `now` to the next local midnight (when « today » changes). */
export function msUntilMidnight(now: Date = new Date()): number {
  const next = new Date(now.getFullYear(), now.getMonth(), now.getDate() + 1);
  return next.getTime() - now.getTime();
}

/** The civil date `n` days after `iso`: stepping the cursor by a week. */
export function shiftDate(iso: string, days: number): string {
  return dateOf(dayOf(iso) + days);
}

/** The end of a planting is stored exclusive (S03 §1, `[planting out, end[`) but
 *  everything the user reads or types is the LAST OCCUPIED DAY (S03 §1, display
 *  convention): this is the day before. */
export function lastDay(endExclusive: string): string {
  return shiftDate(endExclusive, -1);
}

/** The stored (exclusive) end of a planting whose last occupied day is `last`. */
export function endExclusive(last: string): string {
  return shiftDate(last, 1);
}

/** Formats a civil date in its own calendar, never through the time zone of
 *  the device: a zone that skipped a day (Samoa, 2011-12-30) would shift it. */
export function formatCivil(iso: string, options: Intl.DateTimeFormatOptions): string {
  const [a = 1970, m = 1, j = 1] = iso.split("-").map(Number);
  return new Date(utcMs(a, m, j, 12)).toLocaleDateString(localeTag(), { ...options, timeZone: "UTC" });
}

/** A civil date as « 2 August », with the year when it is not `year`. */
export function dateAbbreviated(iso: string, year?: number): string {
  return formatCivil(
    iso,
    yearOf(iso) === year ? { day: "numeric", month: "short" } : { day: "numeric", month: "short", year: "numeric" },
  );
}

/** « Monday 2 August 2027 ». */
export function dateComplete(iso: string): string {
  return formatCivil(iso, { weekday: "long", day: "numeric", month: "long", year: "numeric" });
}

/** The same month and day in another year, for moving the cursor from one
 *  season to another. February 29th becomes the 28th in a common year. */
export function sameDayYear(iso: string, year: number): string {
  const [, m = 1, j = 1] = iso.split("-").map(Number);
  const ms = utcMs(year, m, j);
  const lastOfMonth = new Date(ms).getUTCMonth() !== m - 1 ? utcMs(year, m + 1, 0) : ms;
  return dateOf(Math.round(lastOfMonth / MS_BY_DAY));
}
