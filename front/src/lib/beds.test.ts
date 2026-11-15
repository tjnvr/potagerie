import { describe, expect, test } from "vitest";
import type { BedView, PlotView } from "./bindings";
import { yearsProposees, summary, enM2, enMeters, letterOf, nameByDefault, positionFree } from "./beds";

const plot: PlotView = { length_cm: 1100, width_cm: 650, cell_cm: 50, aisle_cm: 50 };

function bed(name: string, x: number, y: number, length_cm = 300, width_cm = 120): BedView {
  return { id: name, name, bed_type: "bed", x, y, length_cm, width_cm, segments: length_cm / 50, plantings_deleted: [] };
}

describe("nameByDefault", () => {
  test("starts at A", () => {
    expect(nameByDefault("bed", [])).toBe("Bed A");
  });

  test("takes the first letter no bed ends with", () => {
    const beds = [bed("Bed A", 50, 50), bed("The tomatoes", 400, 50), bed("Square C", 50, 250)];
    expect(nameByDefault("square", beds)).toBe("Square B");
  });
});

describe("letterOf", () => {
  test("goes past Z", () => {
    expect([0, 25, 26, 27].map(letterOf)).toEqual(["A", "Z", "AA", "AB"]);
  });
});

describe("positionFree", () => {
  test("keeps the aisle width from the edges", () => {
    expect(positionFree(plot, [], "bed")).toEqual({ x: 50, y: 50 });
  });

  test("keeps the aisle width from the other beds", () => {
    // 50 + 300 + 50 of aisle: the next one starts at x = 400.
    expect(positionFree(plot, [bed("A", 50, 50)], "bed")).toEqual({ x: 400, y: 50 });
  });

  test("goes to the next row when a row is full", () => {
    const row = [bed("A", 50, 50), bed("B", 400, 50), bed("C", 750, 50)];
    expect(positionFree(plot, row, "bed")).toEqual({ x: 50, y: 250 });
  });

  test("returns null when the plot is full", () => {
    const petit: PlotView = { length_cm: 300, width_cm: 200, cell_cm: 50, aisle_cm: 50 };
    expect(positionFree(petit, [], "bed")).toBeNull();
  });
});

test("the season selector offers last year to two years ahead", () => {
  expect(yearsProposees(2027)).toEqual([2026, 2027, 2028, 2029]);
});

describe("summary", () => {
  test("adds up the beds and keeps the rest for the aisles", () => {
    const b = summary(plot, [bed("A", 50, 50), bed("B", 400, 50, 100, 100)]);
    expect(b).toEqual({ total: 715_000, cultivable: 36_000 + 10_000, aisles: 669_000, count: 2 });
  });
});

test("meters and square meters follow the app language", () => {
  expect(enMeters(120)).toBe("1.2");
  expect(enMeters(300)).toBe("3");
  expect(enM2(36_000)).toBe("3.6");
});
