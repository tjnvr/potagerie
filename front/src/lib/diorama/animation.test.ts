import { describe, expect, test } from "vitest";
import type { AppearanceChange } from "../bindings";
import {
  DURATION_GROWTH_DAYS,
  DAYS_BY_SECONDE,
  PALIER_DAYS,
  SYMBOL_ETIQUETTE,
  apparenceTo,
  chronology,
  stateTo,
  indexTo,
  dayOfPlayback,
  bounce,
} from "./animation";
import { daySince } from "./days";

// A tomato sown under cover on 2027-03-10, as Rust sends it (stages.rs test).
const changes: AppearanceChange[] = [
  { from: "2027-03-10", appearance: { appearance: "label" } },
  { from: "2027-05-05", appearance: { appearance: "plant", stage: "sowing" } },
  { from: "2027-05-17", appearance: { appearance: "plant", stage: "young_shoot" } },
  { from: "2027-06-12", appearance: { appearance: "plant", stage: "adult" } },
  { from: "2027-07-19", appearance: { appearance: "plant", stage: "harvest" } },
  { from: "2027-09-27", appearance: { appearance: "nothing" } },
];
const chrono = chronology(changes);
const j = daySince;

describe("apparenceTo", () => {
  test("nothing before the first change", () => {
    expect(indexTo(chrono, j("2027-03-09"))).toBe(-1);
    expect(apparenceTo(chrono, j("2027-03-09"))).toEqual({ appearance: "nothing" });
  });

  test("a change is in force from its day, not the day before", () => {
    expect(apparenceTo(chrono, j("2027-05-04"))).toEqual({ appearance: "label" });
    expect(apparenceTo(chrono, j("2027-05-05"))).toEqual({ appearance: "plant", stage: "sowing" });
    expect(apparenceTo(chrono, j("2027-09-26"))).toEqual({ appearance: "plant", stage: "harvest" });
    expect(apparenceTo(chrono, j("2027-09-27"))).toEqual({ appearance: "nothing" });
  });

  test("a fraction of a day belongs to the day", () => {
    expect(indexTo(chrono, j("2027-05-05") - 0.01)).toBe(0);
    expect(indexTo(chrono, j("2027-05-05") + 0.99)).toBe(1);
  });

  test("a planting without change is never drawn", () => {
    expect(stateTo(chronology([]), "tomato", j("2027-06-01"), true).symbol).toBeNull();
  });
});

describe("stateTo", () => {
  test("a label while sown under cover", () => {
    expect(stateTo(chrono, "tomato", j("2027-04-01"), true)).toEqual({ symbol: SYMBOL_ETIQUETTE, scale: 1 });
  });

  test("the symbol follows the crop and the stage", () => {
    expect(stateTo(chrono, "tomato", j("2027-08-01"), false).symbol).toBe("plant-tomato-harvest");
    expect(stateTo(chrono, "green-bean", j("2027-06-20"), false).symbol).toBe("plant-green-bean-adult");
    expect(stateTo(chrono, "green_bean", j("2027-06-20"), false).symbol).toBe("plant-green-bean-adult");
  });

  test("full size when not animated", () => {
    expect(stateTo(chrono, "tomato", j("2027-05-05"), false).scale).toBe(1);
  });

  test("comes out of the ground small, then settles at full size", () => {
    const start = stateTo(chrono, "tomato", j("2027-05-05"), true);
    expect(start.scale).toBeCloseTo(0.2, 5);
    const end = stateTo(chrono, "tomato", j("2027-05-05") + DURATION_GROWTH_DAYS, true);
    expect(end.scale).toBe(1);
  });

  test("a stage change starts from a larger size than a new plant", () => {
    const next = stateTo(chrono, "tomato", j("2027-06-12"), true);
    expect(next.scale).toBeCloseTo(0.75, 5);
  });

  test("bounces: goes over full size before settling", () => {
    const pics = [];
    for (let t = 0; t <= DURATION_GROWTH_DAYS; t += 0.25) {
      pics.push(stateTo(chrono, "tomato", j("2027-06-12") + t, true).scale);
    }
    expect(Math.max(...pics)).toBeGreaterThan(1);
    expect(Math.max(...pics)).toBeLessThan(1.15);
    expect(pics.at(-1)).toBe(1);
  });
});

describe("bounce", () => {
  test("starts at 0, ends at 1, overshoots in between", () => {
    expect(bounce(0)).toBe(0);
    expect(bounce(1)).toBe(1);
    expect(bounce(0.5)).toBeGreaterThan(1);
    expect(bounce(-1)).toBe(0);
    expect(bounce(2)).toBe(1);
  });
});

describe("dayOfPlayback", () => {
  const base = { start: 0, end: 365, speed: 1, reduced: false };

  test("a whole year in 15 seconds", () => {
    expect(DAYS_BY_SECONDE * 15).toBeCloseTo(365, 9);
    const end = dayOfPlayback({ ...base, elapsedMs: 15_000 });
    expect(end).toEqual({ day: 365, termine: true });
  });

  test("moves smoothly at half time", () => {
    const milieu = dayOfPlayback({ ...base, elapsedMs: 7_500 });
    expect(milieu.day).toBeCloseTo(182.5, 6);
    expect(milieu.termine).toBe(false);
  });

  test("x2 is twice as fast", () => {
    const fast = dayOfPlayback({ ...base, speed: 2, elapsedMs: 7_500 });
    expect(fast.day).toBeCloseTo(365, 6);
    expect(fast.termine).toBe(true);
  });

  test("starts from the cursor and stops at the end of the season", () => {
    const e = dayOfPlayback({ start: 300, end: 365, speed: 1, reduced: false, elapsedMs: 10_000 });
    expect(e).toEqual({ day: 365, termine: true });
  });

  test("reduced motion: steps of a week, no in-between day", () => {
    const vus = new Set<number>();
    for (let ms = 0; ms < 14_000; ms += 16) {
      vus.add(dayOfPlayback({ ...base, reduced: true, elapsedMs: ms }).day);
    }
    for (const day of vus) {
      expect(day % PALIER_DAYS === 0 || day === 365).toBe(true);
    }
    expect(vus.size).toBeGreaterThan(40);
  });
});
