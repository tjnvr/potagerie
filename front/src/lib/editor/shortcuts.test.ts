// @vitest-environment happy-dom
import { describe, expect, test } from "vitest";
import { shortcut } from "./shortcuts";

const key = (key: string, mods: { meta?: boolean; ctrl?: boolean; shift?: boolean; alt?: boolean } = {}, target: EventTarget | null = document.body) => ({
  key,
  metaKey: mods.meta ?? false,
  ctrlKey: mods.ctrl ?? false,
  shiftKey: mods.shift ?? false,
  altKey: mods.alt ?? false,
  target,
});

describe("undo and redo shortcuts", () => {
  test("Cmd+Z and Ctrl+Z undo", () => {
    expect(shortcut(key("z", { meta: true }))).toBe("undo");
    expect(shortcut(key("z", { ctrl: true }))).toBe("undo");
  });

  test("Shift+Cmd+Z, Shift+Ctrl+Z and Ctrl+Y redo", () => {
    // With Shift, browsers report the capital letter.
    expect(shortcut(key("Z", { meta: true, shift: true }))).toBe("redo");
    expect(shortcut(key("Z", { ctrl: true, shift: true }))).toBe("redo");
    expect(shortcut(key("y", { ctrl: true }))).toBe("redo");
  });

  test("other keys and plain letters do nothing", () => {
    expect(shortcut(key("z"))).toBeNull();
    expect(shortcut(key("y", { meta: true }))).toBeNull();
    expect(shortcut(key("z", { ctrl: true, alt: true }))).toBeNull();
    expect(shortcut(key("c", { meta: true }))).toBeNull();
  });

  test("ignored while typing in a field", () => {
    for (const tag of ["input", "textarea", "select"]) {
      expect(shortcut(key("z", { meta: true }, document.createElement(tag)))).toBeNull();
    }
    const editable = document.createElement("div");
    editable.contentEditable = "true";
    document.body.append(editable);
    expect(shortcut(key("z", { ctrl: true }, editable))).toBeNull();
    editable.remove();
    expect(shortcut(key("z", { ctrl: true }, document.createElement("button")))).toBe("undo");
  });
});
