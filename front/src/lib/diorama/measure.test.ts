import { describe, expect, test } from "vitest";
import { imagesBySeconde, measure, respectsLeCriterion } from "./measure";

describe("measure", () => {
  test("too short a run gives nothing", () => {
    expect(measure(new Array(10).fill(16.7))).toBeNull();
  });

  test("steady 60 images per second", () => {
    const m = measure(new Array(300).fill(1000 / 60))!;
    expect(m.average).toBeCloseTo(60, 6);
    expect(m.bas1pct).toBeCloseTo(60, 6);
    expect(m.p95ms).toBeCloseTo(16.67, 1);
    expect(respectsLeCriterion(m)).toBe(true);
  });

  test("the first frames are ignored", () => {
    const m = measure([500, 500, 500, 500, 500, ...new Array(100).fill(10)])!;
    expect(m.images).toBe(100);
    expect(m.average).toBeCloseTo(100, 6);
  });

  test("the slowest 1 % count even when the mean is good", () => {
    const durations = [...new Array(990).fill(8), ...new Array(15).fill(60)];
    const m = measure(durations)!;
    expect(m.average).toBeGreaterThan(55);
    expect(m.bas1pct).toBeLessThan(30);
    expect(respectsLeCriterion(m)).toBe(false);
  });
});

describe("imagesBySeconde", () => {
  test("from recent frame times", () => {
    expect(imagesBySeconde([20, 20, 20])).toBe(50);
    expect(imagesBySeconde([])).toBe(0);
  });
});
