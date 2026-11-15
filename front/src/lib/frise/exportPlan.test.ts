import { describe, expect, it } from "vitest";
import { anneesDePeriode, echellePourPeriode, periodeParDefaut, periodeVisible, problemePeriode } from "./exportPlan";
import { jourDe } from "../dates";

describe("export period", () => {
  it("defaults to the civil year of the cursor", () => {
    expect(periodeParDefaut("2027-06-20")).toEqual({ du: "2027-01-01", au: "2027-12-31" });
  });

  it("takes the days shown on screen, zoom and scroll included", () => {
    // From the middle of 2027-03-10, 90.2 days wide: 2027-03-10 to 2027-06-08.
    expect(periodeVisible({ debutJour: jourDe("2027-03-10") + 0.5, jours: 90.2 })).toEqual({ du: "2027-03-10", au: "2027-06-08" });
    // A whole number of days ends the day before.
    expect(periodeVisible({ debutJour: jourDe("2027-01-01"), jours: 365 })).toEqual({ du: "2027-01-01", au: "2027-12-31" });
    // Never more than the 3 years an export allows.
    const long = periodeVisible({ debutJour: jourDe("2027-01-01") + 0.3, jours: 1095 });
    expect(problemePeriode(long.du, long.au)).toBeNull();
  });

  it("refuses an empty, reversed or too long period", () => {
    expect(problemePeriode("", "2027-12-31")).toBe("vide");
    expect(problemePeriode("2027-05-01", "2027-04-30")).toBe("inversee");
    expect(problemePeriode("2027-01-01", "2030-12-31")).toBe("trop_longue");
    expect(problemePeriode("2027-01-01", "2027-12-31")).toBeNull();
    expect(problemePeriode("2027-03-01", "2027-03-01")).toBeNull();
  });

  it("draws the period in a fixed width, bounded for a short one", () => {
    const annee = echellePourPeriode("2027-01-01", "2027-12-31");
    expect(annee.largeur).toBeCloseTo(1800, 5);
    expect(annee.pxParJour).toBeCloseTo(1800 / 365, 5);
    expect(echellePourPeriode("2027-06-01", "2027-06-02").pxParJour).toBe(8);
  });

  it("lists the years it touches", () => {
    expect(anneesDePeriode("2027-11-01", "2028-02-01")).toEqual([2027, 2028]);
  });
});
