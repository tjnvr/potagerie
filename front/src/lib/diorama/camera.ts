// Zoom and pan of the garden view (S01 F-VIS-1): the camera only changes the
// SVG view box, so the drawing itself is never rebuilt. Positions are kept as
// fractions of the frame, so a quarter turn (another frame) keeps the camera.

export const MIN_ZOOM = 1;
export const MAX_ZOOM = 4;

/** `k`: zoom (1 = the whole frame); `cx`, `cy`: centre of the window, as a fraction of the frame. */
export type Camera = { k: number; cx: number; cy: number };

export const CAMERA_HOME: Camera = { k: 1, cx: 0.5, cy: 0.5 };

type Box = [number, number, number, number];

function clamp(v: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, v));
}

/** Keeps the window inside the frame. */
function within(c: Camera): Camera {
  const half = 1 / (2 * c.k);
  return { k: c.k, cx: clamp(c.cx, half, 1 - half), cy: clamp(c.cy, half, 1 - half) };
}

/** Zooms by `factor` around `point` (a fraction of the frame), which stays where it is on screen. */
export function zoomAt(c: Camera, factor: number, point: [number, number]): Camera {
  const k = clamp(c.k * factor, MIN_ZOOM, MAX_ZOOM);
  const r = c.k / k;
  return within({ k, cx: point[0] + (c.cx - point[0]) * r, cy: point[1] + (c.cy - point[1]) * r });
}

/** Moves the centre of the window by `dx`, `dy` (fractions of the frame). */
export function pan(c: Camera, dx: number, dy: number): Camera {
  return within({ k: c.k, cx: c.cx + dx, cy: c.cy + dy });
}

/** The view box shown for `frame` (the whole diorama) through the camera. */
export function viewBox(frame: Box, c: Camera): Box {
  const [x, y, w, h] = frame;
  const vw = w / c.k;
  const vh = h / c.k;
  return [x + c.cx * w - vw / 2, y + c.cy * h - vh / 2, vw, vh];
}

/**
 * The zoom factor of one wheel event: `deltaY` in pixels (`deltaMode` 0) or
 * lines (1); `pinch` for a trackpad pinch, which the browser sends as a wheel
 * with Ctrl and small deltas. One step is at most ×2.
 */
export function wheelFactor(deltaY: number, deltaMode: number, pinch: boolean): number {
  const px = deltaMode === 1 ? deltaY * 33 : deltaMode === 2 ? deltaY * 400 : deltaY;
  const exponent = clamp(-px * (pinch ? 0.01 : 0.002), -1, 1);
  return 2 ** exponent;
}
