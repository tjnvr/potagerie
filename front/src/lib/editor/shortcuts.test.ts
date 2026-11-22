// @vitest-environment happy-dom
import { describe, expect, test } from "vitest";
import { isMac, keysOf, shortcut } from "./shortcuts";

const key = (key: string, mods: { meta?: boolean; ctrl?: boolean; shift?: boolean; alt?: boolean } = {}, target: EventTarget | null = document.body) => ({
  key,
  metaKey: mods.meta ?? false,
  ctrlKey: mods.ctrl ?? false,
  shiftKey: mods.shift ?? false,
  altKey: mods.alt ?? false,
  target,
});

const MAC = true;
const PC = false;

describe("the platform", () => {
  test("macOS and iOS use Cmd, the others Ctrl", () => {
    for (const platform of ["MacIntel", "iPhone", "iPad"]) expect(isMac({ platform })).toBe(true);
    for (const platform of ["Win32", "Linux x86_64", ""]) expect(isMac({ platform })).toBe(false);
  });
});

describe("undo and redo shortcuts", () => {
  test("Cmd+Z undoes on macOS, Ctrl+Z elsewhere", () => {
    expect(shortcut(key("z", { meta: true }), MAC)).toEqual({ action: "undo" });
    expect(shortcut(key("z", { ctrl: true }), PC)).toEqual({ action: "undo" });
  });

  test("the modifier of the other platform does nothing", () => {
    expect(shortcut(key("z", { ctrl: true }), MAC)).toBeNull();
    expect(shortcut(key("y", { ctrl: true }), MAC)).toBeNull();
    expect(shortcut(key("z", { meta: true }), PC)).toBeNull();
    expect(shortcut(key("Z", { meta: true, shift: true }), PC)).toBeNull();
  });

  test("Shift+Cmd+Z redoes on macOS, Ctrl+Y and Shift+Ctrl+Z elsewhere", () => {
    // With Shift, browsers report the capital letter.
    expect(shortcut(key("Z", { meta: true, shift: true }), MAC)).toEqual({ action: "redo" });
    expect(shortcut(key("Z", { ctrl: true, shift: true }), PC)).toEqual({ action: "redo" });
    expect(shortcut(key("y", { ctrl: true }), PC)).toEqual({ action: "redo" });
    expect(shortcut(key("y", { meta: true }), MAC)).toBeNull();
  });

  test("other keys, Alt and both modifiers do nothing", () => {
    expect(shortcut(key("z"), MAC)).toBeNull();
    expect(shortcut(key("z", { ctrl: true, alt: true }), PC)).toBeNull();
    expect(shortcut(key("z", { ctrl: true, meta: true }), MAC)).toBeNull();
    expect(shortcut(key("x", { meta: true }), MAC)).toBeNull();
  });
});

describe("copy, paste and duplicate shortcuts", () => {
  test("C, V and D with Cmd on macOS, with Ctrl elsewhere", () => {
    for (const [mods, mac] of [[{ meta: true }, MAC], [{ ctrl: true }, PC]] as const) {
      expect(shortcut(key("c", mods), mac)).toEqual({ action: "copy" });
      expect(shortcut(key("v", mods), mac)).toEqual({ action: "paste" });
      expect(shortcut(key("d", mods), mac)).toEqual({ action: "duplicate" });
      // Caps Lock reports the capital letter.
      expect(shortcut(key("C", mods), mac)).toEqual({ action: "copy" });
    }
  });

  test("the other modifier, Shift, Alt or no modifier do nothing", () => {
    expect(shortcut(key("c", { ctrl: true }), MAC)).toBeNull();
    expect(shortcut(key("v", { meta: true }), PC)).toBeNull();
    expect(shortcut(key("c"), PC)).toBeNull();
    expect(shortcut(key("v", { meta: true, shift: true }), MAC)).toBeNull();
    expect(shortcut(key("d", { ctrl: true, alt: true }), PC)).toBeNull();
  });

  test("ignored while typing in a field", () => {
    for (const k of ["c", "v", "d"]) {
      expect(shortcut(key(k, { meta: true }, document.createElement("input")), MAC)).toBeNull();
      expect(shortcut(key(k, { ctrl: true }, document.createElement("textarea")), PC)).toBeNull();
    }
  });
});

describe("keys of the editor", () => {
  test("Escape cancels or deselects, Delete and Backspace delete", () => {
    expect(shortcut(key("Escape"), MAC)).toEqual({ action: "escape" });
    expect(shortcut(key("Delete"), PC)).toEqual({ action: "delete" });
    expect(shortcut(key("Backspace"), MAC)).toEqual({ action: "delete" });
  });

  test("the arrows move by one cell, with Shift by one metre", () => {
    expect(shortcut(key("ArrowLeft"), PC)).toEqual({ action: "move", dx: -1, dy: 0, metre: false });
    expect(shortcut(key("ArrowRight"), PC)).toEqual({ action: "move", dx: 1, dy: 0, metre: false });
    expect(shortcut(key("ArrowUp"), PC)).toEqual({ action: "move", dx: 0, dy: -1, metre: false });
    expect(shortcut(key("ArrowDown", { shift: true }), MAC)).toEqual({ action: "move", dx: 0, dy: 1, metre: true });
  });

  test("Tab selects the next bed, Shift+Tab the previous one", () => {
    expect(shortcut(key("Tab"), PC)).toEqual({ action: "next" });
    expect(shortcut(key("Tab", { shift: true }), MAC)).toEqual({ action: "previous" });
  });

  test("with a modifier they are left to the system", () => {
    expect(shortcut(key("Backspace", { meta: true }), MAC)).toBeNull();
    expect(shortcut(key("ArrowLeft", { ctrl: true }), PC)).toBeNull();
    expect(shortcut(key("ArrowLeft", { alt: true }), PC)).toBeNull();
    expect(shortcut(key("Tab", { ctrl: true }), PC)).toBeNull();
    expect(shortcut(key("Delete", { shift: true }), PC)).toBeNull();
  });
});

describe("while typing", () => {
  test("every shortcut is ignored in a field", () => {
    const keys = [key("z", { meta: true }), key("Escape"), key("Backspace"), key("ArrowLeft"), key("Tab")];
    for (const tag of ["input", "textarea", "select"]) {
      for (const k of keys) expect(shortcut({ ...k, target: document.createElement(tag) }, MAC)).toBeNull();
    }
    const editable = document.createElement("div");
    editable.contentEditable = "true";
    document.body.append(editable);
    expect(shortcut(key("z", { ctrl: true }, editable), PC)).toBeNull();
    expect(shortcut(key("Delete", {}, editable), PC)).toBeNull();
    editable.remove();
    expect(shortcut(key("z", { ctrl: true }, document.createElement("button")), PC)).toEqual({ action: "undo" });
  });
});

describe("keys shown in the tooltips", () => {
  test("the symbols of macOS, the names elsewhere", () => {
    expect(keysOf("undo", MAC)).toBe("⌘Z");
    expect(keysOf("redo", MAC)).toBe("⇧⌘Z");
    expect(keysOf("delete", MAC)).toBe("⌫");
    expect(keysOf("undo", PC)).toBe("Ctrl+Z");
    expect(keysOf("redo", PC)).toBe("Ctrl+Y");
    expect(keysOf("delete", PC)).toBe("Delete");
    expect(keysOf("escape", PC)).toBe("Esc");
    expect(keysOf("duplicate", MAC)).toBe("⌘D");
    expect(keysOf("duplicate", PC)).toBe("Ctrl+D");
    expect(keysOf("paste", MAC)).toBe("⌘V");
    expect(keysOf("paste", PC)).toBe("Ctrl+V");
  });
});
