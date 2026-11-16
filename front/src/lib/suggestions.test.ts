import { describe, expect, it } from "vitest";
import type { SuggestionView } from "./bindings";
import { rankSuggestions } from "./suggestions";

function suggestion(crop_id: string, score_tenths: number, recent_family = false): SuggestionView {
  return {
    crop_id,
    mode: "open_ground",
    date: "2027-05-01",
    planting_out: "2027-05-01",
    harvest_start: "2027-08-01",
    end: "2027-11-16",
    manual_end: null,
    score_tenths,
    recent_family,
  };
}

describe("rankSuggestions", () => {
  it("sorts the ties with the names shown, and keeps 3", () => {
    const ranked = rankSuggestions([
      suggestion("zucchini", 7),
      suggestion("tomato", 7),
      suggestion("carrot", 7),
      suggestion("leek", 10),
      suggestion("lettuce", 7),
    ]);
    // English names (the test language): Carrot, Lettuce, Tomato after the leek.
    expect(ranked.map((s) => s.crop_id)).toEqual(["leek", "carrot", "lettuce"]);
  });

  it("puts the family of the last planting after the others of the same score", () => {
    const ranked = rankSuggestions([suggestion("carrot", 7, true), suggestion("tomato", 7), suggestion("zucchini", 7)]);
    expect(ranked.map((s) => s.crop_id)).toEqual(["tomato", "zucchini", "carrot"]);
  });
});
