import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import {
  ControleurGlisser,
  deposer,
  pixelsVersCm,
  positionDepuisBac,
  positionDeposee,
  redimensionner,
  type Geo,
} from "./glisser";

// A 1100 × 650 cm terrain with a margin of 22 cm, drawn in 560 × 340 pixels
// at (100, 50) on the screen.
const boite = { x: -22, y: -22, largeur: 1144, hauteur: 694 };
const rect = { left: 100, top: 50, width: 572, height: 347 };

describe("pixelsVersCm", () => {
  test("converts the corners and the centre", () => {
    expect(pixelsVersCm({ x: 100, y: 50 }, rect, boite)).toEqual({ x: -22, y: -22 });
    expect(pixelsVersCm({ x: 672, y: 397 }, rect, boite)).toEqual({ x: 1122, y: 672 });
    const centre = pixelsVersCm({ x: 386, y: 223.5 }, rect, boite);
    expect(centre.x).toBeCloseTo(550);
    expect(centre.y).toBeCloseTo(325);
  });
});

describe("snapping to cases", () => {
  test("a planche held by its offset lands on the grid", () => {
    expect(positionDeposee({ x: 480, y: 130 }, { x: 40, y: 20 }, 50)).toEqual({ x: 450, y: 100 });
    expect(positionDeposee({ x: 484, y: 135 }, { x: 40, y: 20 }, 25)).toEqual({ x: 450, y: 125 });
  });

  test("a planche from the tray is held by its centre", () => {
    // 300 × 120: the pointer at (500, 200) puts the top left at (350, 140) → (350, 150).
    expect(positionDepuisBac({ x: 500, y: 200 }, { longueur_cm: 300, largeur_cm: 120 }, 50)).toEqual({
      x: 350,
      y: 150,
    });
  });
});

describe("redimensionner", () => {
  const g: Geo = { x: 100, y: 100, longueur_cm: 300, largeur_cm: 120 };

  test("the east handle moves the length case by case", () => {
    expect(redimensionner(g, "e", { x: 540, y: 160 }, 50)).toEqual({ ...g, longueur_cm: 450 });
    expect(redimensionner(g, "e", { x: 90, y: 160 }, 50).longueur_cm).toBe(50);
  });

  test("the west handle keeps the right edge", () => {
    const r = redimensionner(g, "w", { x: 40, y: 160 }, 50);
    expect(r).toEqual({ ...g, x: 50, longueur_cm: 350 });
  });

  test("the width is free to the centimetre, with a minimum", () => {
    expect(redimensionner(g, "s", { x: 200, y: 233 }, 50).largeur_cm).toBe(133);
    expect(redimensionner(g, "s", { x: 200, y: 101 }, 50).largeur_cm).toBe(10);
    const n = redimensionner(g, "n", { x: 200, y: 77 }, 50);
    expect(n).toEqual({ ...g, y: 77, largeur_cm: 143 });
  });

  test("a corner handle moves both sides", () => {
    expect(redimensionner(g, "se", { x: 490, y: 250 }, 50)).toEqual({
      ...g,
      longueur_cm: 400,
      largeur_cm: 150,
    });
  });
});

describe("ControleurGlisser", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  test("a mouse drags after a few pixels, a still press is a click", () => {
    const c = new ControleurGlisser();
    c.appui("mouse", { x: 10, y: 10 });
    expect(c.deplacer({ x: 12, y: 11 })).toBe("rien");
    expect(c.deplacer({ x: 20, y: 10 })).toBe("debut");
    expect(c.deplacer({ x: 30, y: 10 })).toBe("glisse");
    expect(c.relacher()).toBe("fin");
    c.appui("mouse", { x: 10, y: 10 });
    expect(c.relacher()).toBe("clic");
  });

  test("a finger needs a long press before the planche follows it", () => {
    const armee = vi.fn();
    const c = new ControleurGlisser({ surArme: armee });
    c.appui("touch", { x: 10, y: 10 });
    vi.advanceTimersByTime(449);
    expect(armee).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(armee).toHaveBeenCalledOnce();
    expect(c.etat).toBe("arme");
    expect(c.deplacer({ x: 11, y: 10 })).toBe("debut");
    expect(c.deplacer({ x: 60, y: 10 })).toBe("glisse");
    expect(c.relacher()).toBe("fin");
  });

  test("a finger that moves before the long press is scrolling: dropped", () => {
    const armee = vi.fn();
    const c = new ControleurGlisser({ surArme: armee });
    c.appui("touch", { x: 10, y: 10 });
    vi.advanceTimersByTime(200);
    expect(c.deplacer({ x: 10, y: 40 })).toBe("abandon");
    vi.advanceTimersByTime(1000);
    expect(armee).not.toHaveBeenCalled();
    expect(c.etat).toBe("repos");
    expect(c.relacher()).toBe("rien");
  });

  test("a finger may tremble a little during the long press", () => {
    const c = new ControleurGlisser();
    c.appui("touch", { x: 10, y: 10 });
    expect(c.deplacer({ x: 14, y: 13 })).toBe("rien");
    vi.advanceTimersByTime(450);
    expect(c.etat).toBe("arme");
  });

  test("a quick tap is a click and cancels the timer", () => {
    const armee = vi.fn();
    const c = new ControleurGlisser({ surArme: armee });
    c.appui("touch", { x: 10, y: 10 });
    vi.advanceTimersByTime(100);
    expect(c.relacher()).toBe("clic");
    vi.advanceTimersByTime(1000);
    expect(armee).not.toHaveBeenCalled();
  });
});

describe("deposer", () => {
  const origine: Geo = { x: 50, y: 50, longueur_cm: 300, largeur_cm: 120 };

  test("the planche stays where it is dropped when Rust accepts", async () => {
    const envoyer = vi.fn().mockResolvedValue("ok");
    const cible = { ...origine, x: 400 };
    expect(await deposer(origine, cible, envoyer)).toEqual({ position: cible, reponse: "ok" });
    expect(envoyer).toHaveBeenCalledWith(cible);
  });

  test("the planche goes back to its place when Rust refuses", async () => {
    const envoyer = vi.fn().mockResolvedValue("refus");
    const issue = await deposer(origine, { ...origine, x: 100 }, envoyer);
    expect(issue).toEqual({ position: origine, reponse: "refus" });
  });

  test("a cancelled confirmation also brings it back", async () => {
    const envoyer = vi.fn().mockResolvedValue("annule");
    expect((await deposer(origine, { ...origine, longueur_cm: 100 }, envoyer)).position).toEqual(origine);
  });

  test("nothing is sent when it did not move", async () => {
    const envoyer = vi.fn();
    expect(await deposer(origine, { ...origine }, envoyer)).toEqual({ position: origine, reponse: "inchange" });
    expect(envoyer).not.toHaveBeenCalled();
  });

  test("a position before the terrain is refused without a call", async () => {
    const envoyer = vi.fn();
    const issue = await deposer(origine, { ...origine, x: -50 }, envoyer);
    expect(issue).toEqual({ position: origine, reponse: "avant-terrain" });
    expect(envoyer).not.toHaveBeenCalled();
  });
});
