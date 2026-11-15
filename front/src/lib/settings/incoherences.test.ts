import { expect, test } from "vitest";
import type { IncoherenceView } from "../bindings";
import { textIncoherence } from "./incoherences";

const base: IncoherenceView = {
  code: "",
  year: null,
  bed_name: null,
  other_bed_name: null,
  crop_id: null,
  other_crop_id: null,
  count: null,
};

test("too many beds names the year and the count", () => {
  const t = textIncoherence({ ...base, code: "too_many_beds", year: 2027, count: 11 });
  expect(t).toBe("2027 has 11 beds, 10 at most.");
});

test("an orphan planting names its crop, with or without a known bed", () => {
  const known = textIncoherence({
    ...base,
    code: "planting_orphan",
    year: 2027,
    bed_name: "The tomatoes",
    crop_id: "tomato",
  });
  expect(known).toBe('The planting of Tomato no longer has its bed "The tomatoes" in 2027.');
  const without = textIncoherence({ ...base, code: "planting_orphan", crop_id: "lettuce" });
  expect(without).toBe("The planting of Lettuce belongs to an unknown bed.");
});

test("an overlap names both crops and the bed", () => {
  const t = textIncoherence({
    ...base,
    code: "overlap",
    bed_name: "Roots",
    crop_id: "carrot",
    other_crop_id: "leek",
  });
  expect(t).toBe('On "Roots", Carrot and Leek take the same segments on the same dates.');
});

test("a placement problem names both beds", () => {
  const t = textIncoherence({
    ...base,
    code: "bed_overlaps",
    year: 2026,
    bed_name: "A",
    other_bed_name: "B",
  });
  expect(t).toBe('"A" overlaps "B" in 2026.');
});

test("an unknown code falls back to a generic line, an unknown crop keeps its id", () => {
  expect(textIncoherence({ ...base, code: "new" })).toBe("Another inconsistency was found in the file.");
  expect(textIncoherence({ ...base, code: "crop_unknown", crop_id: "mangue" })).toBe(
    'The crop "mangue" is not in the catalogue.',
  );
});
