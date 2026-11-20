import { describe, expect, test } from "vitest";
import { CAMERA_HOME, MAX_ZOOM, MIN_ZOOM, pan, viewBox, wheelFactor, zoomAt, type Camera } from "./camera";

const FRAME: [number, number, number, number] = [-200, -100, 400, 200];

describe("camera", () => {
  test("home shows the whole frame", () => {
    expect(viewBox(FRAME, CAMERA_HOME)).toEqual(FRAME);
  });

  test("zooming on the centre keeps the centre", () => {
    const c = zoomAt(CAMERA_HOME, 2, [0.5, 0.5]);
    expect(c).toEqual({ k: 2, cx: 0.5, cy: 0.5 });
    expect(viewBox(FRAME, c)).toEqual([-100, -50, 200, 100]);
  });

  test("zooming on a point keeps that point under the pointer", () => {
    const c = zoomAt(CAMERA_HOME, 2, [0.75, 0.25]);
    const [x, y, w, h] = viewBox(FRAME, c);
    // The point at 75 % / 25 % of the frame sits at 75 % / 25 % of the window.
    expect((FRAME[0] + 0.75 * FRAME[2] - x) / w).toBeCloseTo(0.75, 9);
    expect((FRAME[1] + 0.25 * FRAME[3] - y) / h).toBeCloseTo(0.25, 9);
  });

  test("the zoom stays within its bounds", () => {
    expect(zoomAt(CAMERA_HOME, 0.2, [0.5, 0.5]).k).toBe(MIN_ZOOM);
    expect(zoomAt(CAMERA_HOME, 100, [0.5, 0.5]).k).toBe(MAX_ZOOM);
  });

  test("zooming back out to 1 recentres on the whole frame", () => {
    const c = zoomAt(zoomAt(CAMERA_HOME, 3, [0.9, 0.1]), 1 / 3, [0.2, 0.8]);
    expect(c.k).toBeCloseTo(1, 9);
    expect(c.cx).toBeCloseTo(0.5, 9);
    expect(c.cy).toBeCloseTo(0.5, 9);
  });

  test("panning moves the window, not past the edge of the frame", () => {
    const c: Camera = { k: 2, cx: 0.5, cy: 0.5 };
    expect(pan(c, 0.1, -0.1)).toEqual({ k: 2, cx: 0.6, cy: 0.4 });
    expect(pan(c, 5, -5)).toEqual({ k: 2, cx: 0.75, cy: 0.25 });
  });

  test("at zoom 1 there is nothing to pan", () => {
    expect(pan(CAMERA_HOME, 0.3, 0.3)).toEqual(CAMERA_HOME);
  });

  test("a wheel notch zooms in when scrolling up, out when down, and pinch steps are finer", () => {
    expect(wheelFactor(-100, 0, false)).toBeGreaterThan(1);
    expect(wheelFactor(100, 0, false)).toBeLessThan(1);
    expect(wheelFactor(-100, 0, false) * wheelFactor(100, 0, false)).toBeCloseTo(1, 9);
    // Lines (deltaMode 1) count as about 33 px each.
    expect(wheelFactor(-3, 1, false)).toBeCloseTo(wheelFactor(-100, 0, false), 1);
    // A trackpad pinch (ctrlKey) sends small deltas: they zoom faster per pixel.
    expect(wheelFactor(-10, 0, true)).toBeGreaterThan(wheelFactor(-10, 0, false));
  });

  test("a huge wheel delta does not jump past the bounds in one go", () => {
    expect(wheelFactor(-100000, 0, false)).toBeLessThanOrEqual(2);
    expect(wheelFactor(100000, 0, false)).toBeGreaterThanOrEqual(0.5);
  });
});
