import { describe, expect, it } from "vitest";
import en from "../../messages/en.json";
import fr from "../../messages/fr.json";

const placeholders = (t: string) => [...t.matchAll(/\{(\w+)\}/g)].map((m) => m[1] ?? "").sort();

describe("translations", () => {
  const keys = (o: object) => Object.keys(o).filter((k) => !k.startsWith("$"));

  it("French has exactly the keys of English", () => {
    expect(keys(fr).sort()).toEqual(keys(en).sort());
  });

  it("each message has the same placeholders in both languages", () => {
    for (const k of keys(en)) {
      expect(placeholders((fr as Record<string, string>)[k] ?? ""), k).toEqual(
        placeholders((en as Record<string, string>)[k] ?? ""),
      );
    }
  });
});
