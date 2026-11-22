// Keyboard shortcuts of the plan editor (S01 F-PLN-6). Each one only stands
// for a button: no action needs the keyboard (S01 §5).

export type Shortcut = "undo" | "redo";

type Key = Pick<KeyboardEvent, "key" | "metaKey" | "ctrlKey" | "shiftKey" | "altKey" | "target">;

/** A field being typed in keeps its own Cmd+Z. */
function typing(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
}

/** Cmd+Z / Ctrl+Z undoes, Shift+Cmd+Z / Shift+Ctrl+Z / Ctrl+Y redoes. */
export function shortcut(e: Key): Shortcut | null {
  if (!(e.metaKey || e.ctrlKey) || e.altKey || typing(e.target)) return null;
  const key = e.key.toLowerCase();
  if (key === "z") return e.shiftKey ? "redo" : "undo";
  if (key === "y" && e.ctrlKey && !e.shiftKey) return "redo";
  return null;
}
