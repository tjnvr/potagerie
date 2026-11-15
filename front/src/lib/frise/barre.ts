// Dragging a bar of the timeline (S01 F-PLA-2, S03 §2, S05 §4): the bar
// follows the pointer by whole days; on release Rust judges (S03 T12). A
// refusal brings the bar back to where it was. The gesture itself (mouse after
// a few pixels, finger after a long press) is the `ControleurGlisser` of the atelier.
import type { Reponse } from "../atelier/glisser";
import { decalageEnJours } from "./geometrie";

export type IssueDecalage =
  /** The bar did not move by a whole day: a click, nothing is sent. */
  | { reponse: "inchange"; jours: 0 }
  /** Rust accepted: the bar stays `jours` days further. */
  | { reponse: "ok"; jours: number }
  /** Rust refused (or the change was cancelled): the bar goes back. */
  | { reponse: "refus" | "annule"; jours: 0 };

/** End of a drag of `dxPx` pixels: how many days, and what Rust answered. */
export async function deposerBarre(
  dxPx: number,
  pxParJour: number,
  envoyer: (jours: number) => Promise<Reponse>,
): Promise<IssueDecalage> {
  const jours = decalageEnJours(dxPx, pxParJour);
  if (jours === 0) return { reponse: "inchange", jours: 0 };
  const reponse = await envoyer(jours);
  return reponse === "ok" ? { reponse: "ok", jours } : { reponse, jours: 0 };
}

/** Where the bar is drawn while it is dragged: its pixel offset snapped to
 *  whole days, so it jumps from day to day under the pointer. */
export function decalageAffiche(dxPx: number, pxParJour: number): number {
  return decalageEnJours(dxPx, pxParJour) * pxParJour;
}
