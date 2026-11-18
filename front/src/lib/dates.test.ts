// Civil dates must not depend on the time zone of the device nor on a daylight
// saving change: every test runs once per zone, including zones that skipped a
// whole day (Samoa 2011-12-30, Kiritimati 1994-12-31).
import { afterAll, describe, expect, it } from "vitest";
import {
  dateAbbreviated,
  dateComplete,
  dateOf,
  dayOf,
  endExclusive,
  lastDay,
  premierJanuary,
  sameDayYear,
  shiftDate,
  todayLocal,
} from "./dates";
import { dateShort } from "./beds";
import { dateLong } from "./state.svelte";

const ZONES = ["UTC", "Pacific/Auckland", "America/Los_Angeles", "Pacific/Kiritimati", "Pacific/Apia", "Europe/Paris"];
const initial = process.env.TZ;
afterAll(() => {
  if (initial === undefined) delete process.env.TZ;
  else process.env.TZ = initial;
});

describe.each(ZONES)("civil dates in %s", (zone) => {
  const inZone = () => {
    process.env.TZ = zone;
  };

  it("day numbers: 1970, leap years, 2100 (not leap), new year", () => {
    inZone();
    expect(dayOf("1970-01-01")).toBe(0);
    expect(dateOf(0)).toBe("1970-01-01");
    for (const iso of ["2000-02-29", "2028-02-29", "2100-02-28", "2100-03-01", "2026-12-31", "2027-01-01", "2100-12-31"]) {
      expect(dateOf(dayOf(iso))).toBe(iso);
    }
    expect(dayOf("2028-03-01") - dayOf("2028-02-28")).toBe(2);
    expect(dayOf("2100-03-01") - dayOf("2100-02-28")).toBe(1);
  });

  it("years before 100 are not read as 19xx", () => {
    inZone();
    expect(dateOf(dayOf("0050-03-01"))).toBe("0050-03-01");
    expect(dayOf("0001-01-01")).toBeLessThan(dayOf("0099-12-31"));
    expect(premierJanuary(50)).toBe(dayOf("0050-01-01"));
    expect(dateOf(premierJanuary(50))).toBe("0050-01-01");
  });

  it("shifts over the 29th of February and the new year", () => {
    inZone();
    expect(shiftDate("2028-02-28", 1)).toBe("2028-02-29");
    expect(shiftDate("2028-02-29", 1)).toBe("2028-03-01");
    expect(shiftDate("2027-02-28", 1)).toBe("2027-03-01");
    expect(shiftDate("2027-12-31", 1)).toBe("2028-01-01");
    expect(shiftDate("2028-01-01", -1)).toBe("2027-12-31");
    expect(shiftDate("2028-02-29", 365)).toBe("2029-02-28");
  });

  it("last occupied day and exclusive end", () => {
    inZone();
    expect(endExclusive("2027-12-31")).toBe("2028-01-01");
    expect(lastDay("2028-01-01")).toBe("2027-12-31");
    expect(lastDay("2028-03-01")).toBe("2028-02-29");
    expect(lastDay("2100-03-01")).toBe("2100-02-28");
    expect(endExclusive("2028-02-29")).toBe("2028-03-01");
    expect(lastDay(endExclusive("1970-01-01"))).toBe("1970-01-01");
  });

  it("same day in another year", () => {
    inZone();
    expect(sameDayYear("2028-02-29", 2027)).toBe("2027-02-28");
    expect(sameDayYear("2027-02-28", 2028)).toBe("2028-02-28");
    expect(sameDayYear("2028-02-29", 2100)).toBe("2100-02-28");
    expect(sameDayYear("2028-12-31", 2029)).toBe("2029-12-31");
    expect(sameDayYear("2028-02-29", 50)).toBe("0050-02-28");
  });

  it("today is the local date, around midnight and the new year", () => {
    inZone();
    expect(todayLocal(new Date(2027, 11, 31, 23, 59, 59))).toBe("2027-12-31");
    expect(todayLocal(new Date(2028, 0, 1, 0, 0, 0))).toBe("2028-01-01");
    expect(todayLocal(new Date(2028, 1, 29, 0, 30))).toBe("2028-02-29");
  });

  it("formatted dates keep the day, even when the zone skipped it", () => {
    inZone();
    for (const text of [dateAbbreviated("2011-12-30", 2011), dateShort("2011-12-30", 2011), dateLong("2011-12-30"), dateComplete("2011-12-30")]) {
      expect(text).toMatch(/\b30\b/);
      expect(text).toMatch(/Dec/);
    }
    for (const text of [dateAbbreviated("1994-12-31", 1994), dateShort("1994-12-31"), dateLong("1994-12-31")]) {
      expect(text).toMatch(/\b31\b/);
    }
    for (const text of [dateAbbreviated("2028-02-29", 2028), dateComplete("2028-02-29"), dateLong("2028-02-29")]) {
      expect(text).toMatch(/\b29\b/);
      expect(text).toMatch(/Feb/);
    }
    expect(dateComplete("2011-12-30")).toMatch(/Fri/);
  });
});
