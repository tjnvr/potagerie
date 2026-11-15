// The position in time, shared by the timeline (E05) and the garden view
// (diorama, playback). One store: the date of the cursor, a civil date
// `YYYY-MM-DD`. Whoever moves the cursor writes it; the others follow it.
// The timeline scrolls to keep it visible; the diorama draws that date.
//
//   import { cursor, allerA } from "../temps";
//   $cursor            // read, in a component
//   allerA("2027-06-20") // move it (playback: call it on each step)
import { get, writable } from "svelte/store";
import { todayLocal, shiftDate } from "./dates";

/** The date of the cursor. At startup, today (S01 F-VIS-3). */
export const cursor = writable<string>(todayLocal());

/** Counts the requests to bring the cursor back to the middle of the timeline
 *  (« Aujourd'hui », even when the cursor already is on that date). */
export const recentering = writable(0);

/** Puts the cursor on a date. */
export function allerA(date: string): void {
  cursor.set(date);
}

/** « Aujourd'hui » (S01 F-VIS-3): the cursor on today, and the timeline
 *  scrolled back to it. */
export function allerToday(): void {
  cursor.set(todayLocal());
  recentering.update((n) => n + 1);
}

/** The buttons ‹ and › : a week back or ahead (S01 F-VIS-3). */
export function avancerWeeks(weeks: number): void {
  cursor.set(shiftDate(get(cursor), 7 * weeks));
}

/** The days the timeline shows now, after zoom and scroll: the first day
 *  (days since 1970, may be fractional) and how many. `null` before the
 *  timeline is drawn. The export (E11) starts from it (S01 F-EXP-1). */
export const viewportTimeline = writable<{ startDay: number; days: number } | null>(null);

/** True while « Play la season » runs (S01 F-VIS-4). */
export const playbackEnProgress = writable<boolean>(false);
