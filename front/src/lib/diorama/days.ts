import { localeTag } from "../locale";
// Day numbers: days since 1970-01-01, UTC, so a civil date is an integer and
// the animation can move through the days with a fraction.

const MS_BY_DAY = 86_400_000;

/** `YYYY-MM-DD` as a day number. */
export function daySince(iso: string): number {
  const [a = 1970, m = 1, j = 1] = iso.split("-").map(Number);
  return Math.round(Date.UTC(a, m - 1, j) / MS_BY_DAY);
}

/** A day number (a fraction is dropped) as `YYYY-MM-DD`. */
export function isoSince(day: number): string {
  return new Date(Math.floor(day) * MS_BY_DAY).toISOString().slice(0, 10);
}

/** First and last day numbers of a calendar year. */
export function bornesYear(year: number): { start: number; end: number } {
  return { start: daySince(`${year}-01-01`), end: daySince(`${year}-12-31`) };
}

/** The month, 1 to 12, of a day number. */
export function monthOf(day: number): number {
  return new Date(Math.floor(day) * MS_BY_DAY).getUTCMonth() + 1;
}

/** A day number as « 20 June 2027 ». */
export function dateEnAllLetters(day: number): string {
  return new Date(Math.floor(day) * MS_BY_DAY).toLocaleDateString(localeTag(), {
    day: "numeric",
    month: "long",
    year: "numeric",
    timeZone: "UTC",
  });
}
