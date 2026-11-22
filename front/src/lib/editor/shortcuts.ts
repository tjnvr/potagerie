// Keyboard shortcuts of the plan editor (S01 F-PLN-6, S05 §20). Each one only
// stands for a button or a gesture: no action needs the keyboard (S01 §5).
import { m } from "../paraglide/messages.js";

export type Shortcut =
  | { action: "undo" | "redo" | "copy" | "paste" | "duplicate" | "escape" | "delete" | "next" | "previous" }
  /** By one cell along `dx` / `dy`, or by one metre with Shift. */
  | { action: "move"; dx: number; dy: number; metre: boolean };

type Key = Pick<KeyboardEvent, "key" | "metaKey" | "ctrlKey" | "shiftKey" | "altKey" | "target">;

/** macOS and iOS take Cmd as the modifier, the other platforms Ctrl. */
export function isMac(nav: Pick<Navigator, "platform"> = navigator): boolean {
  return /Mac|iPhone|iPad|iPod/.test(nav.platform);
}

/** A field being typed in keeps its own keys. */
function typing(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
}

const ARROWS: Record<string, [number, number]> = {
  ArrowLeft: [-1, 0],
  ArrowRight: [1, 0],
  ArrowUp: [0, -1],
  ArrowDown: [0, 1],
};

const WITH_MODIFIER: Record<string, "copy" | "paste" | "duplicate"> = { c: "copy", v: "paste", d: "duplicate" };

/** Cmd+Z undoes and Shift+Cmd+Z redoes on macOS; Ctrl+Z undoes and Ctrl+Y or
 *  Shift+Ctrl+Z redo elsewhere. C, V and D with the same modifier copy,
 *  paste and duplicate. Escape, Delete / Backspace, the arrows (Shift:
 *  one metre) and Tab / Shift+Tab, without a modifier. */
export function shortcut(e: Key, mac = isMac()): Shortcut | null {
  if (e.altKey || typing(e.target)) return null;
  const modifier = mac ? e.metaKey : e.ctrlKey;
  const other = mac ? e.ctrlKey : e.metaKey;
  if (other) return null;
  if (modifier) {
    const key = e.key.toLowerCase();
    if (key === "z") return { action: e.shiftKey ? "redo" : "undo" };
    if (key === "y" && !mac && !e.shiftKey) return { action: "redo" };
    // Copy, paste and duplicate a bed (S05 §21).
    if (!e.shiftKey && key in WITH_MODIFIER) return { action: WITH_MODIFIER[key]! };
    return null;
  }
  const arrow = ARROWS[e.key];
  if (arrow) return { action: "move", dx: arrow[0], dy: arrow[1], metre: e.shiftKey };
  if (e.key === "Tab") return { action: e.shiftKey ? "previous" : "next" };
  if (e.shiftKey) return null;
  if (e.key === "Escape") return { action: "escape" };
  if (e.key === "Delete" || e.key === "Backspace") return { action: "delete" };
  return null;
}

/** The keys of a shortcut as the tooltips show them: the symbols of macOS,
 *  the names of the keys elsewhere. */
export function keysOf(action: "undo" | "redo" | "paste" | "duplicate" | "delete" | "escape", mac = isMac()): string {
  switch (action) {
    case "paste":
      return mac ? "⌘V" : "Ctrl+V";
    case "duplicate":
      return mac ? "⌘D" : "Ctrl+D";
    case "undo":
      return mac ? "⌘Z" : "Ctrl+Z";
    case "redo":
      return mac ? "⇧⌘Z" : "Ctrl+Y";
    case "delete":
      return mac ? "⌫" : m.key_delete();
    case "escape":
      return m.key_escape();
  }
}

/** A tooltip: the name of the button and its keys. */
export function withKeys(label: string, action: Parameters<typeof keysOf>[0]): string {
  return m.shortcut_tooltip({ label, keys: keysOf(action) });
}
