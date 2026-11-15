// The position in time, shared by the timeline (E05) and the garden view
// (diorama, playback). One store: the date of the cursor, a civil date
// `YYYY-MM-DD`. Whoever moves the cursor writes it; the others follow it.
// The timeline scrolls to keep it visible; the diorama draws that date.
//
//   import { curseur, allerA } from "../temps";
//   $curseur            // read, in a component
//   allerA("2027-06-20") // move it (playback: call it on each step)
import { get, writable } from "svelte/store";
import { aujourdhuiLocal, decalerDate } from "./dates";

/** The date of the cursor. At startup, today (S01 F-VIS-3). */
export const curseur = writable<string>(aujourdhuiLocal());

/** Counts the requests to bring the cursor back to the middle of the timeline
 *  (« Aujourd'hui », even when the cursor already is on that date). */
export const recentrage = writable(0);

/** Puts the cursor on a date. */
export function allerA(date: string): void {
  curseur.set(date);
}

/** « Aujourd'hui » (S01 F-VIS-3): the cursor on today, and the timeline
 *  scrolled back to it. */
export function allerAujourdhui(): void {
  curseur.set(aujourdhuiLocal());
  recentrage.update((n) => n + 1);
}

/** The buttons ‹ and › : a week back or ahead (S01 F-VIS-3). */
export function avancerSemaines(semaines: number): void {
  curseur.set(decalerDate(get(curseur), 7 * semaines));
}
