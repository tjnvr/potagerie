import { localeTag } from "./locale";
// Civil dates `YYYY-MM-DD` (S03 §1) as day numbers, for the layout of the
// timeline only: pixels to days and back. No rule of S03 lives here (no
// duration, no conflict, no slot): every date that matters comes from Rust.

const MS_BY_DAY = 86_400_000;

/** Days since 1970-01-01 of a civil date. Never goes through a time zone. */
export function dayOf(iso: string): number {
  const [a = 1970, m = 1, j = 1] = iso.split("-").map(Number);
  return Math.round(Date.UTC(a, m - 1, j) / MS_BY_DAY);
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

/** The civil date `n` days after `iso`: stepping the cursor by a week. */
export function shiftDate(iso: string, days: number): string {
  return dateOf(dayOf(iso) + days);
}

/** A civil date as « 2 août », with the year when it is not `year`. */
export function dateAbbreviated(iso: string, year?: number): string {
  const day = new Date(`${iso}T12:00:00`);
  const options: Intl.DateTimeFormatOptions =
    day.getFullYear() === year ? { day: "numeric", month: "short" } : { day: "numeric", month: "short", year: "numeric" };
  return day.toLocaleDateString(localeTag(), options);
}

/** « lundi 2 août 2027 ». */
export function dateComplete(iso: string): string {
  return new Date(`${iso}T12:00:00`).toLocaleDateString(localeTag(), {
    weekday: "long",
    day: "numeric",
    month: "long",
    year: "numeric",
  });
}

/** The same month and day in another year, for moving the cursor from one
 *  season to another. February 29th becomes the 28th in a common year. */
export function sameDayYear(iso: string, year: number): string {
  const [, m = 1, j = 1] = iso.split("-").map(Number);
  const day = new Date(Date.UTC(year, m - 1, j));
  const dernier = day.getUTCMonth() !== m - 1 ? Date.UTC(year, m, 0) : day.getTime();
  return new Date(dernier).toISOString().slice(0, 10);
}
