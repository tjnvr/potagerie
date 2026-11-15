import { describe, expect, test } from "vitest";
import { gaps, scalePlant, positionsPlants, type GeoBed, type ZonePlanting } from "./plants";

// « The tomatoes »: 4 m × 1.2 m at (200, 150), 50 cm tiles.
const bed: GeoBed = { x: 200, y: 150, length_cm: 400, width_cm: 120 };

function zone(start: number, end: number, rows: number, plants: number): ZonePlanting {
  return { segment_start: start, segment_end: end, rows, plants_by_row: plants };
}

describe("positionsPlants", () => {
  test("rows times plants per row, as the view says (S03 §6)", () => {
    expect(positionsPlants(bed, zone(1, 8, 2, 7), 50)).toHaveLength(14);
    expect(positionsPlants(bed, zone(1, 4, 3, 9), 50)).toHaveLength(27);
  });

  test("one row of 4 plants over the whole bed, evenly spread", () => {
    const plants = positionsPlants(bed, zone(1, 8, 1, 4), 50);
    // 400 cm / 4 = 100 cm apart, the first at 50 cm: x = 250, 350, 450, 550 cm.
    expect(plants.map((p) => p.gx)).toEqual([5, 7, 9, 11]);
    // The only row is in the middle of the width: 150 + 60 = 210 cm.
    expect(plants.every((p) => p.gy === 4.2)).toBe(true);
  });

  test("rows are spread across the width", () => {
    const plants = positionsPlants(bed, zone(1, 8, 3, 1), 50);
    expect(plants.map((p) => p.gy)).toEqual([(150 + 20) / 50, (150 + 60) / 50, (150 + 100) / 50]);
  });

  test("a zone covers only its segments", () => {
    // Segments 3 to 4 of a 50 cm grid: from 200 + 100 = 300 cm to 400 cm.
    const plants = positionsPlants(bed, zone(3, 4, 1, 2), 50);
    expect(plants.map((p) => p.gx)).toEqual([(300 + 25) / 50, (300 + 75) / 50]);
  });

  test("the segment size is the plot case size", () => {
    const plants = positionsPlants(bed, zone(2, 2, 1, 1), 100);
    // The second segment of a 100 cm grid: 300 to 400 cm, centre 350 cm.
    expect(plants).toEqual([{ gx: 7, gy: 4.2 }]);
  });

  test("plants closer than 15 cm are thinned, not piled up", () => {
    // 4 rows of 25 plants (S03 T4: carrots 4 cm apart) on 1 m: 6 per row, 8 rows fit but only 4 exist.
    const plants = positionsPlants(bed, zone(1, 2, 4, 25), 50);
    expect(plants).toHaveLength(4 * 6);
    const rows = new Set(plants.map((p) => p.gy));
    expect(rows.size).toBe(4);
  });

  test("never zero plants, even when the view has no layout (0 × 0)", () => {
    expect(positionsPlants(bed, zone(1, 1, 0, 0), 50)).toHaveLength(1);
  });
});

describe("scalePlant", () => {
  test("carrots a few centimetres apart are drawn small", () => {
    // Thinned to 6 plants every 16.7 cm, rows 30 cm apart: sqrt(16.7 x 30) / 35.
    expect(scalePlant(bed, zone(1, 2, 4, 25), 50)).toBeCloseTo(0.64, 2);
  });

  test("a zucchini alone on a metre is drawn large, but capped", () => {
    expect(scalePlant(bed, zone(1, 2, 1, 1), 50)).toBe(1.1);
  });

  test("a lettuce every 30 cm is about the reference size", () => {
    const k = scalePlant({ ...bed, width_cm: 120 }, zone(1, 8, 4, 13), 50);
    expect(k).toBeGreaterThan(0.7);
    expect(k).toBeLessThan(1);
  });

  test("gaps follow the zone and the rows", () => {
    expect(gaps(bed, zone(1, 4, 3, 5), 50)).toEqual({ spacing: 40, row: 40 });
  });

  test("gaps are those of the drawing, once thinned", () => {
    expect(gaps(bed, zone(1, 2, 4, 25), 50).spacing).toBeCloseTo(100 / 6, 6);
  });
});
