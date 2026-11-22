// Logic of the drag and drop of the editor (S05 §4): pointer to centimetres,
// snapping to cases, long press on touch, resizing with handles, and the
// return to the starting place when Rust refuses. No rule of S03 here: a
// position is only a proposal, Rust judges at the drop.

export type Point = { x: number; y: number };
export type Rect = { left: number; top: number; width: number; height: number };
/** The `viewBox` of the plan, in centimetres. */
export type Box = { x: number; y: number; width: number; height: number };
/** Where a bed is and how big, in centimetres. */
export type Geo = { x: number; y: number; length_cm: number; width_cm: number };
export type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

/** Pointer moves under this distance, in pixels, are a click, not a drag. */
export const THRESHOLD_MOUSE_PX = 4;
/** On touch, the finger may drift this far while waiting for the long press. */
export const TOLERANCE_PRESS_LONG_PX = 8;
export const LONG_PRESS_DELAY_MS = 450;
/** The narrowest width the handles can give: the same minimum as Rust (S03 §15),
 *  so a handle never proposes a width Rust would call invalid. */
export const WIDTH_MIN_CM = 10;

/** Centimetres of the plan per pixel of the screen. The SVG keeps the
 *  proportions of its `viewBox` (`xMidYMid meet`): the drawing fills the
 *  element on one axis only, and the same scale applies to both. */
export function cmPerPixel(rect: Rect, box: Box): number {
  return Math.max(box.width / Math.max(rect.width, 1), box.height / Math.max(rect.height, 1));
}

/** Pixels of the screen to centimetres of the plan. When the element is wider
 *  (or taller) than the drawing, the drawing is centred in it: the empty bands
 *  on the sides are not part of the plan. */
export function pixelsToCm(p: Point, rect: Rect, box: Box): Point {
  const k = cmPerPixel(rect, box);
  return {
    x: box.x + (p.x - rect.left - (rect.width - box.width / k) / 2) * k,
    y: box.y + (p.y - rect.top - (rect.height - box.height / k) / 2) * k,
  };
}

/** The nearest multiple of the case size. */
export function roundedCase(cm: number, step: number): number {
  return Math.round(cm / step) * step;
}

/** Top left of a bed carried by `shift` (where the pointer holds it),
 *  snapped to the grid of cases. */
export function positionDropped(pointer: Point, shift: Point, step: number): Point {
  return { x: roundedCase(pointer.x - shift.x, step), y: roundedCase(pointer.y - shift.y, step) };
}

/** A bed dropped from the tray is held by its centre. */
export function positionSincePlanter(
  pointer: Point,
  size: { length_cm: number; width_cm: number },
  step: number,
): Point {
  return positionDropped(pointer, { x: size.length_cm / 2, y: size.width_cm / 2 }, step);
}

/**
 * The geometry proposed while a handle is dragged. The length moves case by
 * case (a whole number of cases, at least one); the width is free to the
 * centimetre (S01 F-PLN-1).
 */
export function resize(g: Geo, handle: Handle, held: Point, step: number, gap: Point = { x: 0, y: 0 }): Geo {
  // `gap` is how far from its edge the handle was pressed: kept, the edge
  // moves with the pointer instead of jumping to it (S05 §19).
  const pointer = { x: held.x - gap.x, y: held.y - gap.y };
  const right = g.x + g.length_cm;
  const low = g.y + g.width_cm;
  let { x, y, length_cm, width_cm } = g;
  if (handle.includes("e")) {
    length_cm = Math.max(step, roundedCase(pointer.x - g.x, step));
  }
  if (handle.includes("w")) {
    length_cm = Math.max(step, roundedCase(right - pointer.x, step));
    x = right - length_cm;
  }
  if (handle.includes("s")) {
    width_cm = Math.max(WIDTH_MIN_CM, Math.round(pointer.y - g.y));
  }
  if (handle.includes("n")) {
    width_cm = Math.max(WIDTH_MIN_CM, Math.round(low - pointer.y));
    y = low - width_cm;
  }
  return { x, y, length_cm, width_cm };
}

/** Where a handle is drawn: a corner, or the middle of a side. */
export function handlePoint(g: Geo, handle: Handle): Point {
  const fx = handle.includes("w") ? 0 : handle.includes("e") ? 1 : 0.5;
  const fy = handle.includes("n") ? 0 : handle.includes("s") ? 1 : 0.5;
  return { x: g.x + fx * g.length_cm, y: g.y + fy * g.width_cm };
}

/** Across, in screen pixels, the circle that takes the press of a handle:
 *  at least 40 px on a touch screen (S01 §5), smaller with a mouse. */
export const HANDLE_HIT_TOUCH_PX = 44;
export const HANDLE_HIT_MOUSE_PX = 36;

/** A handle: its dot at (`x`, `y`) and the circle that takes the press,
 *  centred at (`cx`, `cy`), of radius `r`, all in centimetres. */
export type HandleZone = { name: Handle; x: number; y: number; cx: number; cy: number; r: number };

const HANDLES: Handle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];

/**
 * The handles of a bed and their hit circles, `hitPx` across on the screen
 * (`scale` centimetres per pixel). A circle reaches at most a quarter of the
 * bed into it, pushed outwards when the bed is small on the screen, so that
 * its middle stays free to move it; the middle handle of a side too short to
 * keep it apart from the corners is left out (S05 §19).
 */
export function handleZones(g: Geo, scale: number, hitPx: number): HandleZone[] {
  const r = (hitPx / 2) * scale;
  // Room for a middle circle between the two corner ones.
  const roomX = g.length_cm >= 4 * r;
  const roomY = g.width_cm >= 4 * r;
  return HANDLES.filter((h) => (h === "n" || h === "s" ? roomX : h === "e" || h === "w" ? roomY : true)).map(
    (name) => {
      const p = handlePoint(g, name);
      const sx = name.includes("w") ? -1 : name.includes("e") ? 1 : 0;
      const sy = name.includes("n") ? -1 : name.includes("s") ? 1 : 0;
      let ox = 0;
      let oy = 0;
      if (sx !== 0 && sy !== 0) {
        // A corner pushed out along its diagonal by `o`: the circle crosses the
        // two sides at sqrt(r² - o²) - o from the corner; that is set to `t`.
        const t = Math.min(r, g.length_cm / 4, g.width_cm / 4);
        const o = (-t + Math.sqrt(2 * r * r - t * t)) / 2;
        ox = o;
        oy = o;
      } else if (sx !== 0) {
        ox = r - Math.min(r, g.length_cm / 4);
      } else {
        oy = r - Math.min(r, g.width_cm / 4);
      }
      return { name, x: p.x, y: p.y, cx: p.x + sx * ox, cy: p.y + sy * oy, r };
    },
  );
}

/** Rust takes unsigned positions: a negative one is left of or above the
 *  plot, so it is refused as outside without a call. */
export function isBeforePlot(g: Pick<Geo, "x" | "y">): boolean {
  return g.x < 0 || g.y < 0;
}

export type TypePointer = "mouse" | "touch" | "pen";
export type StateGeste = "rest" | "waiting" | "armed" | "dragged";

type Timer = {
  start: (rappel: () => void, ms: number) => unknown;
  stop: (id: unknown) => void;
};

const timerReal: Timer = {
  start: (rappel, ms) => setTimeout(rappel, ms),
  stop: (id) => clearTimeout(id as ReturnType<typeof setTimeout>),
};

/**
 * When does a press become a drag? With a mouse (or a pen), as soon as the
 * pointer moves a few pixels. With a finger, only after a long press: a finger
 * that moves before that is scrolling the page, and the gesture is dropped
 * (S01 F-VIS-2, S05 §4).
 */
export class ControllerDrag {
  state: StateGeste = "rest";
  #depart: Point = { x: 0, y: 0 };
  #type: TypePointer = "mouse";
  #timerId: unknown = null;
  readonly #onArmed: () => void;
  readonly #delay: number;
  readonly #timers: Timer;

  constructor(options: { onArmed?: () => void; delay?: number; timer?: Timer } = {}) {
    this.#onArmed = options.onArmed ?? (() => {});
    this.#delay = options.delay ?? LONG_PRESS_DELAY_MS;
    this.#timers = options.timer ?? timerReal;
  }

  /** `immediate`: a finger drags like a mouse, with no long press (a resize
   *  handle, which a finger cannot mean to scroll, S05 §19). */
  press(type: TypePointer, p: Point, options: { immediate?: boolean } = {}): void {
    this.cancel();
    this.#type = options.immediate && type === "touch" ? "mouse" : type;
    this.#depart = p;
    this.state = "waiting";
    if (this.#type === "touch") {
      this.#timerId = this.#timers.start(() => {
        this.#timerId = null;
        if (this.state === "waiting") {
          this.state = "armed";
          this.#onArmed();
        }
      }, this.#delay);
    }
  }

  /** `start` once, when the drag starts; `dragged` while it goes on; `abandon`
   *  when a finger scrolls instead. */
  move(p: Point): "nothing" | "start" | "dragged" | "abandon" {
    const distance = Math.hypot(p.x - this.#depart.x, p.y - this.#depart.y);
    switch (this.state) {
      case "waiting":
        if (this.#type === "touch") {
          if (distance > TOLERANCE_PRESS_LONG_PX) {
            this.cancel();
            return "abandon";
          }
          return "nothing";
        }
        if (distance > THRESHOLD_MOUSE_PX) {
          this.state = "dragged";
          return "start";
        }
        return "nothing";
      case "armed":
        this.state = "dragged";
        return "start";
      case "dragged":
        return "dragged";
      default:
        return "nothing";
    }
  }

  /** `end` after a drag, `click` for a press without drag, `nothing` otherwise. */
  release(): "nothing" | "click" | "end" {
    const before = this.state;
    this.cancel();
    if (before === "dragged") return "end";
    if (before === "waiting" || before === "armed") return "click";
    return "nothing";
  }

  cancel(): void {
    if (this.#timerId !== null) {
      this.#timers.stop(this.#timerId);
      this.#timerId = null;
    }
    this.state = "rest";
  }
}

/** What the command answered. A refusal brings the bed back. */
export type Response = "ok" | "refusal" | "cancelled";

export type IssueDepot = { position: Geo; response: Response | "unchanged" | "before-plot" };

/**
 * End of a drag: where the bed stays. Nothing is sent if it did not
 * move; a position before the plot is refused without a call (Rust takes
 * unsigned numbers); otherwise the command judges, and a refusal, or a
 * cancelled confirmation, returns the bed to `origin`.
 */
export async function deposer(
  origin: Geo,
  target: Geo,
  envoyer: (g: Geo) => Promise<Response>,
): Promise<IssueDepot> {
  if (
    target.x === origin.x &&
    target.y === origin.y &&
    target.length_cm === origin.length_cm &&
    target.width_cm === origin.width_cm
  ) {
    return { position: origin, response: "unchanged" };
  }
  if (isBeforePlot(target)) {
    return { position: origin, response: "before-plot" };
  }
  const response = await envoyer(target);
  return { position: response === "ok" ? target : origin, response };
}
