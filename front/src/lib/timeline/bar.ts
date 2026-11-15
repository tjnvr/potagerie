// Dragging a bar of the timeline (S01 F-PLA-2, S03 §2, S05 §4): the bar
// follows the pointer by whole days; on release Rust judges (S03 T12). A
// refusal brings the bar back to where it was. The gesture itself (mouse after
// a few pixels, finger after a long press) is the `ControllerDrag` of the editor.
import type { Response } from "../editor/drag";
import { shiftInDays } from "./geometry";

export type IssueShift =
  /** The bar did not move by a whole day: a click, nothing is sent. */
  | { response: "unchanged"; days: 0 }
  /** Rust accepted: the bar stays `days` days further. */
  | { response: "ok"; days: number }
  /** Rust refused (or the change was cancelled): the bar goes back. */
  | { response: "refusal" | "cancelled"; days: 0 };

/** End of a drag of `dxPx` pixels: how many days, and what Rust answered. */
export async function deposerBar(
  dxPx: number,
  pxByDay: number,
  envoyer: (days: number) => Promise<Response>,
): Promise<IssueShift> {
  const days = shiftInDays(dxPx, pxByDay);
  if (days === 0) return { response: "unchanged", days: 0 };
  const response = await envoyer(days);
  return response === "ok" ? { response: "ok", days } : { response, days: 0 };
}

/** Where the bar is drawn while it is dragged: its pixel offset snapped to
 *  whole days, so it jumps from day to day under the pointer. */
export function shiftShown(dxPx: number, pxByDay: number): number {
  return shiftInDays(dxPx, pxByDay) * pxByDay;
}
