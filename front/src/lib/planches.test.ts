import { describe, expect, test } from "vitest";
import type { PlancheView, TerrainView } from "./bindings";
import { anneesProposees, bilan, enM2, enMetres, lettreDe, nomParDefaut, positionLibre } from "./planches";

const terrain: TerrainView = { longueur_cm: 1100, largeur_cm: 650, case_cm: 50, allee_cm: 50 };

function planche(nom: string, x: number, y: number, longueur_cm = 300, largeur_cm = 120): PlancheView {
  return { id: nom, nom, planche_type: "planche", x, y, longueur_cm, largeur_cm, troncons: longueur_cm / 50, series_supprimees: [] };
}

describe("nomParDefaut", () => {
  test("starts at A", () => {
    expect(nomParDefaut("planche", [])).toBe("Planche A");
  });

  test("takes the first letter no planche ends with", () => {
    const planches = [planche("Planche A", 50, 50), planche("Les tomates", 400, 50), planche("Carré C", 50, 250)];
    expect(nomParDefaut("carre", planches)).toBe("Carré B");
  });
});

describe("lettreDe", () => {
  test("goes past Z", () => {
    expect([0, 25, 26, 27].map(lettreDe)).toEqual(["A", "Z", "AA", "AB"]);
  });
});

describe("positionLibre", () => {
  test("keeps the allée width from the edges", () => {
    expect(positionLibre(terrain, [], "planche")).toEqual({ x: 50, y: 50 });
  });

  test("keeps the allée width from the other planches", () => {
    // 50 + 300 + 50 of allée: the next one starts at x = 400.
    expect(positionLibre(terrain, [planche("A", 50, 50)], "planche")).toEqual({ x: 400, y: 50 });
  });

  test("goes to the next row when a row is full", () => {
    const rangee = [planche("A", 50, 50), planche("B", 400, 50), planche("C", 750, 50)];
    expect(positionLibre(terrain, rangee, "planche")).toEqual({ x: 50, y: 250 });
  });

  test("returns null when the terrain is full", () => {
    const petit: TerrainView = { longueur_cm: 300, largeur_cm: 200, case_cm: 50, allee_cm: 50 };
    expect(positionLibre(petit, [], "planche")).toBeNull();
  });
});

test("the season selector offers last year to two years ahead", () => {
  expect(anneesProposees(2027)).toEqual([2026, 2027, 2028, 2029]);
});

describe("bilan", () => {
  test("adds up the planches and keeps the rest for the allées", () => {
    const b = bilan(terrain, [planche("A", 50, 50), planche("B", 400, 50, 100, 100)]);
    expect(b).toEqual({ total: 715_000, cultivable: 36_000 + 10_000, allees: 669_000, nombre: 2 });
  });
});

test("metres and square metres read in French", () => {
  expect(enMetres(120)).toBe("1,2");
  expect(enMetres(300)).toBe("3");
  expect(enM2(36_000)).toBe("3,6");
});
