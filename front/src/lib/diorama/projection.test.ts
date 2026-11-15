import { describe, expect, test } from "vitest";
import { TH, TW, cadre, dimensions, enUnites, pivoter, pivoterRect, project, type Quarter } from "./projection";
import { dateEnAllLetters, isoSince, daySince } from "./days";
import { seasonOf } from "./season";

describe("project", () => {
  test("the origin is the top corner of the plot", () => {
    expect(project(0, 0)).toEqual([0, 0]);
  });

  test("one tile along x goes down and right by half a diamond", () => {
    expect(project(1, 0)).toEqual([TW / 2, TH / 2]);
  });

  test("one tile along y goes down and left", () => {
    expect(project(0, 1)).toEqual([-TW / 2, TH / 2]);
  });

  test("height raises the point", () => {
    expect(project(2, 3, 7)).toEqual([-TW / 2, (5 * TH) / 2 - 7]);
  });

  test("a tile is 50 cm", () => {
    expect(enUnites(300)).toBe(6);
  });
});

describe("rotation of the view", () => {
  const l = 22;
  const h = 13;

  test("four quarter turns come back", () => {
    let p: [number, number] = [3, 5];
    let dl = l;
    let dh = h;
    for (let i = 0; i < 4; i++) {
      p = pivoter(p, 1, dl, dh);
      [dl, dh] = [dh, dl];
    }
    expect(p).toEqual([3, 5]);
  });

  test("the plot corners stay the corners of the turned plot", () => {
    for (const quarter of [0, 1, 2, 3] as Quarter[]) {
      const d = dimensions(quarter, l, h);
      for (const coin of [[0, 0], [l, 0], [0, h], [l, h]] as [number, number][]) {
        const [x, y] = pivoter(coin, quarter, l, h);
        expect([0, d.l]).toContain(x);
        expect([0, d.h]).toContain(y);
      }
    }
  });

  test("a rectangle keeps its area when turned", () => {
    const r = pivoterRect({ x: 2, y: 3, l: 6, h: 2 }, 1, l, h);
    expect(r.l * r.h).toBe(12);
    expect(r).toEqual({ x: h - 5, y: 2, l: 2, h: 6 });
  });
});

describe("cadre", () => {
  test("holds the corners of the plot, its thickness and the plants above the back row", () => {
    const [x, y, w, height] = cadre(22, 13);
    expect(x).toBeLessThan(project(0, 13)[0]);
    expect(x + w).toBeGreaterThan(project(22, 0)[0]);
    expect(y).toBeLessThan(project(0, 0)[1] - 60);
    expect(y + height).toBeGreaterThan(project(22, 13, -30)[1]);
  });
});

describe("days", () => {
  test("day numbers round-trip", () => {
    expect(isoSince(daySince("2027-06-20"))).toBe("2027-06-20");
    expect(daySince("2027-03-01") - daySince("2027-02-01")).toBe(28);
    expect(isoSince(daySince("2028-02-29") + 0.9)).toBe("2028-02-29");
  });

  test("a date in words", () => {
    expect(dateEnAllLetters(daySince("2027-06-20"))).toBe("20 June 2027");
  });

  test("calendar seasons, 20 June is summer (D01)", () => {
    expect(seasonOf(daySince("2027-01-15"))).toBe("winter");
    expect(seasonOf(daySince("2027-04-01"))).toBe("spring");
    expect(seasonOf(daySince("2027-06-20"))).toBe("summer");
    expect(seasonOf(daySince("2027-10-31"))).toBe("autumn");
    expect(seasonOf(daySince("2027-12-01"))).toBe("winter");
  });
});
