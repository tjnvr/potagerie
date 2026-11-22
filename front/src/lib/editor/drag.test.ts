import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import {
  cmPerPixel,
  ControllerDrag,
  deposer,
  pixelsToCm,
  positionSincePlanter,
  positionDropped,
  resize,
  handlePoint,
  handleZones,
  HANDLE_HIT_MOUSE_PX,
  HANDLE_HIT_TOUCH_PX,
  type Geo,
  type Handle,
} from "./drag";

// A 1100 × 650 cm plot with a margin of 22 cm, drawn in 560 × 340 pixels
// at (100, 50) on the screen.
const box = { x: -22, y: -22, width: 1144, height: 694 };
const rect = { left: 100, top: 50, width: 572, height: 347 };

describe("pixelsToCm", () => {
  test("converts the corners and the centre", () => {
    expect(pixelsToCm({ x: 100, y: 50 }, rect, box)).toEqual({ x: -22, y: -22 });
    expect(pixelsToCm({ x: 672, y: 397 }, rect, box)).toEqual({ x: 1122, y: 672 });
    const centre = pixelsToCm({ x: 386, y: 223.5 }, rect, box);
    expect(centre.x).toBeCloseTo(550);
    expect(centre.y).toBeCloseTo(325);
  });

  // The plan has `max-height: 70vh`: in a low window the element is wider than
  // its drawing, which the SVG centres (`xMidYMid meet`) with empty bands on the
  // sides. 1144 × 694 cm in 972 × 448 px: 448 / 694 px per cm, the drawing is
  // 738.5 px wide, starting 116.75 px from the left of the element.
  test("follows the drawing when the element is wider than it (letterbox)", () => {
    const wide = { left: 52, top: 199, width: 972, height: 448 };
    const k = 448 / 694;
    const left = 52 + (972 - 1144 * k) / 2;
    expect(pixelsToCm({ x: left, y: 199 }, wide, box).x).toBeCloseTo(-22);
    expect(pixelsToCm({ x: left + 1144 * k, y: 647 }, wide, box)).toEqual({ x: expect.closeTo(1122), y: expect.closeTo(672) });
    // The cell 100 cm from the left of the plot, mid-height.
    const p = pixelsToCm({ x: left + (22 + 100) * k, y: 199 + (22 + 325) * k }, wide, box);
    expect(p.x).toBeCloseTo(100);
    expect(p.y).toBeCloseTo(325);
  });

  test("follows the drawing when the element is taller than it", () => {
    const tall = { left: 0, top: 0, width: 572, height: 600 };
    const k = 572 / 1144;
    const top = (600 - 694 * k) / 2;
    expect(pixelsToCm({ x: 0, y: top }, tall, box)).toEqual({ x: expect.closeTo(-22), y: expect.closeTo(-22) });
    expect(pixelsToCm({ x: 572, y: top + 694 * k }, tall, box)).toEqual({ x: expect.closeTo(1122), y: expect.closeTo(672) });
  });
});

describe("cmPerPixel", () => {
  test("is the scale of the drawing, the same on both axes", () => {
    expect(cmPerPixel(rect, box)).toBeCloseTo(2);
    expect(cmPerPixel({ left: 0, top: 0, width: 972, height: 448 }, box)).toBeCloseTo(694 / 448);
  });
});

describe("snapping to cases", () => {
  test("a bed held by its offset lands on the grid", () => {
    expect(positionDropped({ x: 480, y: 130 }, { x: 40, y: 20 }, 50)).toEqual({ x: 450, y: 100 });
    expect(positionDropped({ x: 484, y: 135 }, { x: 40, y: 20 }, 25)).toEqual({ x: 450, y: 125 });
  });

  test("a bed from the tray is held by its centre", () => {
    // 300 × 120: the pointer at (500, 200) puts the top left at (350, 140) → (350, 150).
    expect(positionSincePlanter({ x: 500, y: 200 }, { length_cm: 300, width_cm: 120 }, 50)).toEqual({
      x: 350,
      y: 150,
    });
  });
});

describe("resize", () => {
  const g: Geo = { x: 100, y: 100, length_cm: 300, width_cm: 120 };

  test("the east handle moves the length case by case", () => {
    expect(resize(g, "e", { x: 540, y: 160 }, 50)).toEqual({ ...g, length_cm: 450 });
    expect(resize(g, "e", { x: 90, y: 160 }, 50).length_cm).toBe(50);
  });

  test("the west handle keeps the right edge", () => {
    const r = resize(g, "w", { x: 40, y: 160 }, 50);
    expect(r).toEqual({ ...g, x: 50, length_cm: 350 });
  });

  test("the width is free to the centimetre, with a minimum", () => {
    expect(resize(g, "s", { x: 200, y: 233 }, 50).width_cm).toBe(133);
    expect(resize(g, "s", { x: 200, y: 101 }, 50).width_cm).toBe(10);
    const n = resize(g, "n", { x: 200, y: 77 }, 50);
    expect(n).toEqual({ ...g, y: 77, width_cm: 143 });
  });

  test("a corner handle moves both sides", () => {
    expect(resize(g, "se", { x: 490, y: 250 }, 50)).toEqual({
      ...g,
      length_cm: 400,
      width_cm: 150,
    });
  });

  // Each handle, pulled outwards: the edges it carries follow the pointer (a whole
  // number of cases for the length, the centimetre for the width), the other edges stay.
  // Edges of g: left 100, right 400, top 100, bottom 220.
  const pulled: [Handle, { x: number; y: number }, Geo][] = [
    ["n", { x: 0, y: 63.4 }, { x: 100, y: 63, length_cm: 300, width_cm: 157 }],
    ["s", { x: 0, y: 251.6 }, { x: 100, y: 100, length_cm: 300, width_cm: 152 }],
    ["e", { x: 474, y: 0 }, { x: 100, y: 100, length_cm: 350, width_cm: 120 }],
    ["w", { x: 26, y: 0 }, { x: 50, y: 100, length_cm: 350, width_cm: 120 }],
    ["ne", { x: 476, y: 41 }, { x: 100, y: 41, length_cm: 400, width_cm: 179 }],
    ["nw", { x: -12, y: 41 }, { x: 0, y: 41, length_cm: 400, width_cm: 179 }],
    ["se", { x: 476, y: 300 }, { x: 100, y: 100, length_cm: 400, width_cm: 200 }],
    ["sw", { x: -12, y: 300 }, { x: 0, y: 100, length_cm: 400, width_cm: 200 }],
  ];
  test.each(pulled)("the %s handle carries its edges and keeps the opposite ones", (h, p, expected) => {
    const r = resize(g, h, p, 50);
    expect(r).toEqual(expected);
    if (!h.includes("e") && !h.includes("w")) expect([r.x, r.x + r.length_cm]).toEqual([100, 400]);
    if (h.includes("e")) expect(r.x).toBe(100);
    if (h.includes("w")) expect(r.x + r.length_cm).toBe(400);
    if (!h.includes("n") && !h.includes("s")) expect([r.y, r.y + r.width_cm]).toEqual([100, 220]);
    if (h.includes("s")) expect(r.y).toBe(100);
    if (h.includes("n")) expect(r.y + r.width_cm).toBe(220);
  });

  // Each handle pushed past the opposite edge: the bed stops at one case long and
  // 10 cm wide, against the opposite edge, and never turns over.
  const crushed: [Handle, Geo][] = [
    ["n", { x: 100, y: 210, length_cm: 300, width_cm: 10 }],
    ["s", { x: 100, y: 100, length_cm: 300, width_cm: 10 }],
    ["e", { x: 100, y: 100, length_cm: 50, width_cm: 120 }],
    ["w", { x: 350, y: 100, length_cm: 50, width_cm: 120 }],
    ["ne", { x: 100, y: 210, length_cm: 50, width_cm: 10 }],
    ["nw", { x: 350, y: 210, length_cm: 50, width_cm: 10 }],
    ["se", { x: 100, y: 100, length_cm: 50, width_cm: 10 }],
    ["sw", { x: 350, y: 100, length_cm: 50, width_cm: 10 }],
  ];
  test.each(crushed)("the %s handle stops at the minimum size", (h, expected) => {
    const beyond = { x: h.includes("e") ? -500 : 900, y: h.includes("s") ? -500 : 900 };
    expect(resize(g, h, beyond, 50)).toEqual(expected);
  });

  test("the length goes to the nearest case: half a case is the turning point", () => {
    expect(resize(g, "e", { x: 424, y: 0 }, 50).length_cm).toBe(300);
    expect(resize(g, "e", { x: 425, y: 0 }, 50).length_cm).toBe(350);
    expect(resize(g, "w", { x: 76, y: 0 }, 50).length_cm).toBe(300);
    expect(resize(g, "w", { x: 75, y: 0 }, 50).length_cm).toBe(350);
  });

  test("the gap kept from the press: the edge moves with the pointer, without a jump", () => {
    // Pressed 12 cm right of and 9 cm below the south-east corner (400, 220).
    const gap = { x: 12, y: 9 };
    expect(resize(g, "se", { x: 412, y: 229 }, 50, gap)).toEqual(g);
    expect(resize(g, "se", { x: 512, y: 262 }, 50, gap)).toEqual({ ...g, length_cm: 400, width_cm: 153 });
    // Pressed 9 cm above the north edge, then 30 cm higher.
    expect(resize(g, "n", { x: 0, y: 61 }, 50, { x: 0, y: -9 })).toEqual({ ...g, y: 70, width_cm: 150 });
  });
});

describe("handlePoint", () => {
  test("is the corner or the middle of the side", () => {
    const g: Geo = { x: 100, y: 100, length_cm: 300, width_cm: 120 };
    expect(handlePoint(g, "nw")).toEqual({ x: 100, y: 100 });
    expect(handlePoint(g, "n")).toEqual({ x: 250, y: 100 });
    expect(handlePoint(g, "e")).toEqual({ x: 400, y: 160 });
    expect(handlePoint(g, "se")).toEqual({ x: 400, y: 220 });
    expect(handlePoint(g, "sw")).toEqual({ x: 100, y: 220 });
  });
});

describe("handleZones", () => {
  test("a touch target is at least 40 px across, a mouse one stays smaller", () => {
    expect(HANDLE_HIT_TOUCH_PX).toBeGreaterThanOrEqual(40);
    expect(HANDLE_HIT_MOUSE_PX).toBeLessThan(HANDLE_HIT_TOUCH_PX);
    // 2 cm per pixel: a 44 px target is 88 cm across.
    const zones = handleZones({ x: 0, y: 0, length_cm: 600, width_cm: 400 }, 2, HANDLE_HIT_TOUCH_PX);
    for (const z of zones) expect(z.r * 2).toBeCloseTo(88);
  });

  test("on a bed big on the screen, the 8 zones are centred on their handle", () => {
    const g: Geo = { x: 0, y: 0, length_cm: 600, width_cm: 400 };
    const zones = handleZones(g, 2, HANDLE_HIT_TOUCH_PX); // 300 × 200 px
    expect(zones.map((z) => z.name).sort()).toEqual(["e", "n", "ne", "nw", "s", "se", "sw", "w"]);
    for (const z of zones) {
      const p = handlePoint(g, z.name);
      expect({ x: z.x, y: z.y }).toEqual(p);
      expect(z.cx).toBeCloseTo(p.x);
      expect(z.cy).toBeCloseTo(p.y);
    }
  });

  // A 1.2 × 1 m square on a phone, at 10 cm per pixel: 12 × 10 px on the screen.
  test("on a bed small on the screen, the zones leave its middle free and keep the corners", () => {
    const g: Geo = { x: 500, y: 300, length_cm: 120, width_cm: 100 };
    const zones = handleZones(g, 10, HANDLE_HIT_TOUCH_PX);
    // The sides are too short to keep a middle handle apart from the corners.
    expect(zones.map((z) => z.name).sort()).toEqual(["ne", "nw", "se", "sw"]);
    // The handle itself is still in its zone.
    for (const z of zones) expect(Math.hypot(z.cx - z.x, z.cy - z.y)).toBeLessThanOrEqual(z.r + 1e-9);
    // No zone reaches the middle half of the bed: it can still be grabbed to move it.
    for (let i = 0; i <= 10; i++)
      for (let j = 0; j <= 10; j++) {
        const p = { x: 530 + (60 * i) / 10, y: 325 + (50 * j) / 10 };
        for (const z of zones) expect(Math.hypot(p.x - z.cx, p.y - z.cy)).toBeGreaterThan(z.r);
      }
  });

  test("a long and narrow bed keeps its north and south handles, not its east and west ones", () => {
    // 300 × 120 cm at 2 cm per pixel: 150 × 60 px, with 44 px targets.
    const zones = handleZones({ x: 0, y: 0, length_cm: 300, width_cm: 120 }, 2, HANDLE_HIT_TOUCH_PX);
    expect(zones.map((z) => z.name).sort()).toEqual(["n", "ne", "nw", "s", "se", "sw"]);
  });
});

describe("ControllerDrag", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  test("a mouse drags after a few pixels, a still press is a click", () => {
    const c = new ControllerDrag();
    c.press("mouse", { x: 10, y: 10 });
    expect(c.move({ x: 12, y: 11 })).toBe("nothing");
    expect(c.move({ x: 20, y: 10 })).toBe("start");
    expect(c.move({ x: 30, y: 10 })).toBe("dragged");
    expect(c.release()).toBe("end");
    c.press("mouse", { x: 10, y: 10 });
    expect(c.release()).toBe("click");
  });

  test("a finger needs a long press before the bed follows it", () => {
    const armed = vi.fn();
    const c = new ControllerDrag({ onArmed: armed });
    c.press("touch", { x: 10, y: 10 });
    vi.advanceTimersByTime(449);
    expect(armed).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(armed).toHaveBeenCalledOnce();
    expect(c.state).toBe("armed");
    expect(c.move({ x: 11, y: 10 })).toBe("start");
    expect(c.move({ x: 60, y: 10 })).toBe("dragged");
    expect(c.release()).toBe("end");
  });

  test("a finger that moves before the long press is scrolling: dropped", () => {
    const armed = vi.fn();
    const c = new ControllerDrag({ onArmed: armed });
    c.press("touch", { x: 10, y: 10 });
    vi.advanceTimersByTime(200);
    expect(c.move({ x: 10, y: 40 })).toBe("abandon");
    vi.advanceTimersByTime(1000);
    expect(armed).not.toHaveBeenCalled();
    expect(c.state).toBe("rest");
    expect(c.release()).toBe("nothing");
  });

  test("a finger may tremble a little during the long press", () => {
    const c = new ControllerDrag();
    c.press("touch", { x: 10, y: 10 });
    expect(c.move({ x: 14, y: 13 })).toBe("nothing");
    vi.advanceTimersByTime(450);
    expect(c.state).toBe("armed");
  });

  test("a finger on a handle drags at once, without a long press", () => {
    const armed = vi.fn();
    const c = new ControllerDrag({ onArmed: armed });
    c.press("touch", { x: 10, y: 10 }, { immediate: true });
    expect(c.move({ x: 12, y: 11 })).toBe("nothing");
    expect(c.move({ x: 10, y: 40 })).toBe("start");
    expect(c.move({ x: 10, y: 60 })).toBe("dragged");
    vi.advanceTimersByTime(1000);
    expect(armed).not.toHaveBeenCalled();
    expect(c.release()).toBe("end");
  });

  test("a quick tap is a click and cancels the timer", () => {
    const armed = vi.fn();
    const c = new ControllerDrag({ onArmed: armed });
    c.press("touch", { x: 10, y: 10 });
    vi.advanceTimersByTime(100);
    expect(c.release()).toBe("click");
    vi.advanceTimersByTime(1000);
    expect(armed).not.toHaveBeenCalled();
  });
});

describe("deposer", () => {
  const origin: Geo = { x: 50, y: 50, length_cm: 300, width_cm: 120 };

  test("the bed stays where it is dropped when Rust accepts", async () => {
    const envoyer = vi.fn().mockResolvedValue("ok");
    const target = { ...origin, x: 400 };
    expect(await deposer(origin, target, envoyer)).toEqual({ position: target, response: "ok" });
    expect(envoyer).toHaveBeenCalledWith(target);
  });

  test("the bed goes back to its place when Rust refuses", async () => {
    const envoyer = vi.fn().mockResolvedValue("refusal");
    const issue = await deposer(origin, { ...origin, x: 100 }, envoyer);
    expect(issue).toEqual({ position: origin, response: "refusal" });
  });

  test("a cancelled confirmation also brings it back", async () => {
    const envoyer = vi.fn().mockResolvedValue("cancelled");
    expect((await deposer(origin, { ...origin, length_cm: 100 }, envoyer)).position).toEqual(origin);
  });

  test("nothing is sent when it did not move", async () => {
    const envoyer = vi.fn();
    expect(await deposer(origin, { ...origin }, envoyer)).toEqual({ position: origin, response: "unchanged" });
    expect(envoyer).not.toHaveBeenCalled();
  });

  test("a position before the plot is refused without a call", async () => {
    const envoyer = vi.fn();
    const issue = await deposer(origin, { ...origin, x: -50 }, envoyer);
    expect(issue).toEqual({ position: origin, response: "before-plot" });
    expect(envoyer).not.toHaveBeenCalled();
  });
});
