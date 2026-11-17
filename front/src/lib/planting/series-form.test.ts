// The inconsistencies of the series form settled together (docs/POT-00_test-matrix.md, SER-05..08).
import { describe, expect, it } from "vitest";
import type { SuggestionView } from "../bindings";
import { endExclusive, lastDay } from "../dates";
import en from "../../../messages/en.json";
import fr from "../../../messages/fr.json";
import { countText } from "./catalogue";
import { applySuggestion, conflictText, endFromLastDay, formEmpty, resetManualEnd } from "./planting";

describe("SER-05 the end is read and typed as the last occupied day (S03 §1)", () => {
  it("converts between the stored exclusive end and the last day", () => {
    expect(lastDay("2027-06-07")).toBe("2027-06-06");
    expect(endExclusive("2027-06-06")).toBe("2027-06-07");
    expect(lastDay("2028-01-01")).toBe("2027-12-31");
  });

  it("a typed last day is stored one day later, unless it is the calculated end", () => {
    expect(endFromLastDay("2027-06-06", "2027-08-01")).toBe("2027-06-07");
    // Typing the last day of the calculated end is not « by hand »: 31 July = end 1 August.
    expect(endFromLastDay("2027-07-31", "2027-08-01")).toBeNull();
    expect(endFromLastDay("", "2027-08-01")).toBeNull();
    expect(endFromLastDay("2027-0", "2027-08-01")).toBeNull();
  });
});

describe("SER-06 an end that comes from a suggestion is not « edited by hand »", () => {
  const s = { crop_id: "lettuce", mode: "plants", date: "2027-06-07", manual_end: "2027-11-16" } as SuggestionView;
  it("flags the end as limited by the slot, until it is changed", () => {
    const f = applySuggestion(formEmpty(undefined, "b", 4, "2027-06-01"), s, { start: 1, end: 4 });
    expect(f.manualEnd).toBe("2027-11-16");
    expect(f.endFromSlot).toBe(true);
    expect(resetManualEnd(f).endFromSlot).toBe(false);
  });
  it("has a distinct message in both languages", () => {
    expect(en.planting_slot_end).not.toBe(en.planting_manual_end);
    expect(fr.planting_slot_end).toMatch(/créneau libre/);
    expect(fr.planting_slot_end).not.toMatch(/main/);
  });
});

describe("SER-07 the conflict names one segment in the singular", () => {
  const a = { crop: "Lettuce", from: "1 Jun", to: "6 Jun" };
  it("one segment", () => {
    expect(conflictText({ start: 4, end: 4 }, a)).toBe("Conflict with Lettuce on segment 4, from 1 Jun to 6 Jun.");
  });
  it("several segments", () => {
    expect(conflictText({ start: 3, end: 5 }, a)).toBe("Conflict with Lettuce on segments 3–5, from 1 Jun to 6 Jun.");
  });
  it("French", () => {
    expect(fr.planting_problem_conflict_one).toContain("le tronçon {start}");
    expect(fr.planting_problem_conflict).toContain("les tronçons");
  });
});

describe("SER-08 the catalogue count follows the list and one word is used", () => {
  it("counts the crops shown", () => {
    expect(countText(12)).toMatch(/^12 crops\./);
    expect(countText(1)).toMatch(/^1 crop\./);
  });
  it("French never says « légume » for a crop", () => {
    const text = Object.entries(fr).filter(([k]) => !k.startsWith("crop_") && !k.startsWith("$"));
    expect(text.filter(([, v]) => /légume/i.test(String(v))).map(([k]) => k)).toEqual([]);
  });
});
