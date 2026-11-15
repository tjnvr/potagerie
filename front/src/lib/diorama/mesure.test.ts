import { describe, expect, test } from "vitest";
import { imagesParSeconde, mesurer, respecteLeCritere } from "./mesure";

describe("mesurer", () => {
  test("too short a run gives nothing", () => {
    expect(mesurer(new Array(10).fill(16.7))).toBeNull();
  });

  test("steady 60 images per second", () => {
    const m = mesurer(new Array(300).fill(1000 / 60))!;
    expect(m.moyenne).toBeCloseTo(60, 6);
    expect(m.bas1pct).toBeCloseTo(60, 6);
    expect(m.p95ms).toBeCloseTo(16.67, 1);
    expect(respecteLeCritere(m)).toBe(true);
  });

  test("the first frames are ignored", () => {
    const m = mesurer([500, 500, 500, 500, 500, ...new Array(100).fill(10)])!;
    expect(m.images).toBe(100);
    expect(m.moyenne).toBeCloseTo(100, 6);
  });

  test("the slowest 1 % count even when the mean is good", () => {
    const durees = [...new Array(990).fill(8), ...new Array(15).fill(60)];
    const m = mesurer(durees)!;
    expect(m.moyenne).toBeGreaterThan(55);
    expect(m.bas1pct).toBeLessThan(30);
    expect(respecteLeCritere(m)).toBe(false);
  });
});

describe("imagesParSeconde", () => {
  test("from recent frame times", () => {
    expect(imagesParSeconde([20, 20, 20])).toBe(50);
    expect(imagesParSeconde([])).toBe(0);
  });
});
