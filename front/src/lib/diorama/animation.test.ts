import { describe, expect, test } from "vitest";
import type { AppearanceChange } from "../bindings";
import {
  DUREE_CROISSANCE_JOURS,
  JOURS_PAR_SECONDE,
  PALIER_JOURS,
  SYMBOLE_ETIQUETTE,
  apparenceAu,
  chronologie,
  etatAu,
  indexAu,
  jourDeLecture,
  rebond,
} from "./animation";
import { jourDepuis } from "./jours";

// A tomato sown under cover on 2027-03-10, as Rust sends it (stages.rs test).
const changes: AppearanceChange[] = [
  { from: "2027-03-10", appearance: { appearance: "label" } },
  { from: "2027-05-05", appearance: { appearance: "plant", stage: "sowing" } },
  { from: "2027-05-17", appearance: { appearance: "plant", stage: "young_shoot" } },
  { from: "2027-06-12", appearance: { appearance: "plant", stage: "adult" } },
  { from: "2027-07-19", appearance: { appearance: "plant", stage: "harvest" } },
  { from: "2027-09-27", appearance: { appearance: "nothing" } },
];
const chrono = chronologie(changes);
const j = jourDepuis;

describe("apparenceAu", () => {
  test("nothing before the first change", () => {
    expect(indexAu(chrono, j("2027-03-09"))).toBe(-1);
    expect(apparenceAu(chrono, j("2027-03-09"))).toEqual({ appearance: "nothing" });
  });

  test("a change is in force from its day, not the day before", () => {
    expect(apparenceAu(chrono, j("2027-05-04"))).toEqual({ appearance: "label" });
    expect(apparenceAu(chrono, j("2027-05-05"))).toEqual({ appearance: "plant", stage: "sowing" });
    expect(apparenceAu(chrono, j("2027-09-26"))).toEqual({ appearance: "plant", stage: "harvest" });
    expect(apparenceAu(chrono, j("2027-09-27"))).toEqual({ appearance: "nothing" });
  });

  test("a fraction of a day belongs to the day", () => {
    expect(indexAu(chrono, j("2027-05-05") - 0.01)).toBe(0);
    expect(indexAu(chrono, j("2027-05-05") + 0.99)).toBe(1);
  });

  test("a serie without change is never drawn", () => {
    expect(etatAu(chronologie([]), "tomate", j("2027-06-01"), true).symbole).toBeNull();
  });
});

describe("etatAu", () => {
  test("a label while sown under cover", () => {
    expect(etatAu(chrono, "tomate", j("2027-04-01"), true)).toEqual({ symbole: SYMBOLE_ETIQUETTE, echelle: 1 });
  });

  test("the symbol follows the crop and the stage", () => {
    expect(etatAu(chrono, "tomate", j("2027-08-01"), false).symbole).toBe("plante-tomate-harvest");
    expect(etatAu(chrono, "haricot-vert", j("2027-06-20"), false).symbole).toBe("plante-haricot-vert-adult");
    expect(etatAu(chrono, "haricot_vert", j("2027-06-20"), false).symbole).toBe("plante-haricot-vert-adult");
  });

  test("full size when not animated", () => {
    expect(etatAu(chrono, "tomate", j("2027-05-05"), false).echelle).toBe(1);
  });

  test("comes out of the ground small, then settles at full size", () => {
    const debut = etatAu(chrono, "tomate", j("2027-05-05"), true);
    expect(debut.echelle).toBeCloseTo(0.2, 5);
    const fin = etatAu(chrono, "tomate", j("2027-05-05") + DUREE_CROISSANCE_JOURS, true);
    expect(fin.echelle).toBe(1);
  });

  test("a stage change starts from a larger size than a new plant", () => {
    const suivant = etatAu(chrono, "tomate", j("2027-06-12"), true);
    expect(suivant.echelle).toBeCloseTo(0.75, 5);
  });

  test("bounces: goes over full size before settling", () => {
    const pics = [];
    for (let t = 0; t <= DUREE_CROISSANCE_JOURS; t += 0.25) {
      pics.push(etatAu(chrono, "tomate", j("2027-06-12") + t, true).echelle);
    }
    expect(Math.max(...pics)).toBeGreaterThan(1);
    expect(Math.max(...pics)).toBeLessThan(1.15);
    expect(pics.at(-1)).toBe(1);
  });
});

describe("rebond", () => {
  test("starts at 0, ends at 1, overshoots in between", () => {
    expect(rebond(0)).toBe(0);
    expect(rebond(1)).toBe(1);
    expect(rebond(0.5)).toBeGreaterThan(1);
    expect(rebond(-1)).toBe(0);
    expect(rebond(2)).toBe(1);
  });
});

describe("jourDeLecture", () => {
  const base = { debut: 0, fin: 365, vitesse: 1, reduit: false };

  test("a whole year in 15 seconds", () => {
    expect(JOURS_PAR_SECONDE * 15).toBeCloseTo(365, 9);
    const fin = jourDeLecture({ ...base, ecouleMs: 15_000 });
    expect(fin).toEqual({ jour: 365, termine: true });
  });

  test("moves smoothly at half time", () => {
    const milieu = jourDeLecture({ ...base, ecouleMs: 7_500 });
    expect(milieu.jour).toBeCloseTo(182.5, 6);
    expect(milieu.termine).toBe(false);
  });

  test("x2 is twice as fast", () => {
    const rapide = jourDeLecture({ ...base, vitesse: 2, ecouleMs: 7_500 });
    expect(rapide.jour).toBeCloseTo(365, 6);
    expect(rapide.termine).toBe(true);
  });

  test("starts from the cursor and stops at the end of the season", () => {
    const e = jourDeLecture({ debut: 300, fin: 365, vitesse: 1, reduit: false, ecouleMs: 10_000 });
    expect(e).toEqual({ jour: 365, termine: true });
  });

  test("reduced motion: steps of a week, no in-between day", () => {
    const vus = new Set<number>();
    for (let ms = 0; ms < 14_000; ms += 16) {
      vus.add(jourDeLecture({ ...base, reduit: true, ecouleMs: ms }).jour);
    }
    for (const jour of vus) {
      expect(jour % PALIER_JOURS === 0 || jour === 365).toBe(true);
    }
    expect(vus.size).toBeGreaterThan(40);
  });
});
