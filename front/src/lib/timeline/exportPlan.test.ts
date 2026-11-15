import { describe, expect, it } from "vitest";
import { yearsOfPeriod, scaleForPeriod, periodByDefault, periodVisible, problemPeriod } from "./exportPlan";
import { dayOf } from "../dates";

describe("export period", () => {
  it("defaults to the civil year of the cursor", () => {
    expect(periodByDefault("2027-06-20")).toEqual({ from: "2027-01-01", to: "2027-12-31" });
  });

  it("takes the days shown on screen, zoom and scroll included", () => {
    // From the middle of 2027-03-10, 90.2 days wide: 2027-03-10 to 2027-06-08.
    expect(periodVisible({ startDay: dayOf("2027-03-10") + 0.5, days: 90.2 })).toEqual({ from: "2027-03-10", to: "2027-06-08" });
    // A whole number of days ends the day before.
    expect(periodVisible({ startDay: dayOf("2027-01-01"), days: 365 })).toEqual({ from: "2027-01-01", to: "2027-12-31" });
    // Never more than the 3 years an export allows.
    const long = periodVisible({ startDay: dayOf("2027-01-01") + 0.3, days: 1095 });
    expect(problemPeriod(long.from, long.to)).toBeNull();
  });

  it("refuses an empty, reversed or too long period", () => {
    expect(problemPeriod("", "2027-12-31")).toBe("empty");
    expect(problemPeriod("2027-05-01", "2027-04-30")).toBe("reversed");
    expect(problemPeriod("2027-01-01", "2030-12-31")).toBe("too_long");
    expect(problemPeriod("2027-01-01", "2027-12-31")).toBeNull();
    expect(problemPeriod("2027-03-01", "2027-03-01")).toBeNull();
  });

  it("draws the period in a fixed width, bounded for a short one", () => {
    const year = scaleForPeriod("2027-01-01", "2027-12-31");
    expect(year.width).toBeCloseTo(1800, 5);
    expect(year.pxByDay).toBeCloseTo(1800 / 365, 5);
    expect(scaleForPeriod("2027-06-01", "2027-06-02").pxByDay).toBe(8);
  });

  it("lists the years it touches", () => {
    expect(yearsOfPeriod("2027-11-01", "2028-02-01")).toEqual([2027, 2028]);
  });
});
