import { describe, expect, it, vi } from "vitest";
import type { SlotView, BedView, PlantingView, YearView } from "../bindings";
import { dateOf, dayOf } from "../dates";
import { ControllerDrag, LONG_PRESS_DELAY_MS } from "../editor/drag";
import { shiftShown, deposerBar } from "./bar";
import {
  DAYS_MAX,
  DAYS_MIN,
  yearsVisible,
  barOf,
  centerOn,
  buildRows,
  shiftInDays,
  defiler,
  scaleCentered,
  stack,
  keepVisible,
  dayOfX,
  dayUnderLePointer,
  daysVisible,
  monthVisible,
  xOf,
  zoomer,
  rowAt,
} from "./geometry";

const scale = { startDay: dayOf("2027-01-01"), pxByDay: 2, width: 730 };

describe("pointer to date", () => {
  it("maps a pixel to the civil day under it", () => {
    expect(dateOf(dayUnderLePointer(scale, 100, 100))).toBe("2027-01-01");
    expect(dateOf(dayUnderLePointer(scale, 100 + 2 * 31, 100))).toBe("2027-02-01");
    // The middle of a day stays on that day.
    expect(dateOf(dayUnderLePointer(scale, 100 + 2 * 31 + 1, 100))).toBe("2027-02-01");
    expect(dateOf(dayUnderLePointer(scale, 100 + 2 * 31 - 1, 100))).toBe("2027-01-31");
  });

  it("goes back and forth between days and pixels", () => {
    const day = dayOf("2027-06-20");
    expect(dayOfX(scale, xOf(scale, day))).toBe(day);
  });

  it("reads the window left of the visible days", () => {
    expect(dateOf(dayUnderLePointer(scale, 50, 100))).toBe("2026-12-07");
  });
});

describe("scrolling and zoom", () => {
  it("follows the pointer: moving right shows earlier days", () => {
    const after = defiler(scale, 200);
    expect(after.startDay).toBe(scale.startDay - 100);
    expect(defiler(scale, -20).startDay).toBe(scale.startDay + 10);
  });

  it("does not move for no distance", () => {
    expect(defiler(scale, 0)).toEqual(scale);
  });

  it("centres the timeline on a day", () => {
    const day = dayOf("2027-06-20");
    const e = centerOn(scale, day);
    expect(e.startDay + daysVisible(e) / 2).toBe(day);
  });

  it("keeps a day in view with the least scroll", () => {
    const visible = dayOf("2027-06-01");
    expect(keepVisible(scale, visible)).toBe(scale);
    const loin = dayOf("2028-03-01");
    const e = keepVisible(scale, loin);
    expect(dayOfX(e, e.width)).toBeGreaterThan(loin);
    expect(e.startDay).toBeGreaterThan(scale.startDay);
    expect(e.pxByDay).toBe(scale.pxByDay);
    const before = keepVisible(scale, dayOf("2026-06-01"));
    expect(before.startDay).toBeLessThan(dayOf("2026-06-01"));
  });

  it("zooms around the day under the anchor and stays between 3 months and 3 years", () => {
    const anchor = 300;
    const dayBefore = dayOfX(scale, anchor);
    const e = zoomer(scale, 0.5, anchor);
    expect(daysVisible(e)).toBeCloseTo(daysVisible(scale) / 2);
    expect(dayOfX(e, anchor)).toBeCloseTo(dayBefore);
    expect(daysVisible(zoomer(scale, 0.0001, 0))).toBeCloseTo(DAYS_MIN);
    expect(daysVisible(zoomer(scale, 1000, 0))).toBeCloseTo(DAYS_MAX);
  });

  it("opens on 12 months", () => {
    const e = scaleCentered(1200, dayOf("2027-06-20"));
    expect(daysVisible(e)).toBe(365);
  });

  it("lists the years to load and the month columns", () => {
    expect(yearsVisible(scale)).toEqual([2027, 2028]);
    expect(yearsVisible({ ...scale, startDay: dayOf("2027-03-01") })).toEqual([2027, 2028]);
    expect(yearsVisible({ ...scale, startDay: dayOf("2027-01-01"), width: 600 })).toEqual([2027]);
    const month = monthVisible({ ...scale, width: 600 });
    expect(month).toHaveLength(10);
    expect(month[0]).toMatchObject({ year: 2027, month: 0, days: 31 });
    expect(month[1]).toMatchObject({ month: 1, days: 28 });
  });
});

describe("drag of a bar", () => {
  it("moves by whole days", () => {
    expect(shiftInDays(10, 4)).toBe(3);
    expect(shiftInDays(-5, 4)).toBe(-1);
    expect(shiftInDays(1, 4)).toBe(0);
    expect(shiftShown(10, 4)).toBe(12);
  });

  it("sends nothing for a click", async () => {
    const envoyer = vi.fn();
    expect(await deposerBar(1, 4, envoyer)).toEqual({ response: "unchanged", days: 0 });
    expect(envoyer).not.toHaveBeenCalled();
  });

  it("keeps the new place when Rust accepts", async () => {
    const envoyer = vi.fn().mockResolvedValue("ok");
    expect(await deposerBar(-9, 3, envoyer)).toEqual({ response: "ok", days: -3 });
    expect(envoyer).toHaveBeenCalledWith(-3);
  });

  it("brings the bar back when Rust refuses (S03 T12)", async () => {
    const envoyer = vi.fn().mockResolvedValue("refusal");
    expect(await deposerBar(-3, 3, envoyer)).toEqual({ response: "refusal", days: 0 });
    expect(await deposerBar(-3, 3, async () => "cancelled")).toEqual({ response: "cancelled", days: 0 });
  });
});

describe("long press on a bar", () => {
  function withTimer() {
    let rappel: (() => void) | null = null;
    return {
      timer: {
        start: (f: () => void) => ((rappel = f), 1),
        stop: () => (rappel = null),
      },
      trigger: () => rappel?.(),
    };
  }

  it("arms a finger after the delay, then drags", () => {
    const { timer, trigger } = withTimer();
    const armed = vi.fn();
    const c = new ControllerDrag({ onArmed: armed, timer, delay: LONG_PRESS_DELAY_MS });
    c.press("touch", { x: 10, y: 10 });
    expect(c.move({ x: 12, y: 10 })).toBe("nothing");
    trigger();
    expect(armed).toHaveBeenCalledOnce();
    expect(c.move({ x: 40, y: 10 })).toBe("start");
    expect(c.move({ x: 60, y: 10 })).toBe("dragged");
    expect(c.release()).toBe("end");
  });

  it("lets a finger that moves early scroll the timeline instead", () => {
    const { timer } = withTimer();
    const c = new ControllerDrag({ timer });
    c.press("touch", { x: 10, y: 10 });
    expect(c.move({ x: 60, y: 10 })).toBe("abandon");
    expect(c.release()).toBe("nothing");
  });

  it("starts a mouse drag after a few pixels, without waiting", () => {
    const c = new ControllerDrag();
    c.press("mouse", { x: 10, y: 10 });
    expect(c.move({ x: 20, y: 10 })).toBe("start");
  });
});

// ---------- Rows from the year views ----------

const BED: BedView = {
  id: "p1",
  name: "The tomatoes",
  bed_type: "bed",
  x: 50,
  y: 50,
  length_cm: 300,
  width_cm: 120,
  segments: 6,
  plantings_deleted: [],
};

function planting(id: string, crop: string, over: Partial<PlantingView> = {}): PlantingView {
  return {
    id,
    crop_id: crop,
    bed_id: "p1",
    segment_start: 1,
    segment_end: 3,
    mode: "indoor",
    date: "2027-03-10",
    sowing: "2027-03-10",
    planting_out: "2027-05-05",
    harvest_start: "2027-07-19",
    end: "2027-09-27",
    manual_end: false,
    pot_days: 56,
    rows: 1,
    plants_by_row: 2,
    appearance_today: { appearance: "nothing" },
    ...over,
  } as PlantingView;
}

function view(year: number, over: Partial<YearView> = {}): YearView {
  return {
    year,
    today: "2027-06-20",
    read_only: false,
    plot: { length_cm: 1100, width_cm: 650, cell_cm: 50, aisle_cm: 50 },
    version: 3,
    updated_at: null,
    beds: [BED],
    plantings: [],
    conflicts: [],
    history: [],
    slots: [],
    annotations: [],
    tasks: [],
    ...over,
  };
}

describe("bars and phases", () => {
  it("cuts a planting into pots, growth and harvest from the dates Rust sent", () => {
    const b = barOf(planting("a", "tomato"));
    expect(b.segments.map((s) => s.phase)).toEqual(["pots", "growth", "harvest"]);
    expect(dateOf(b.start)).toBe("2027-03-10");
    expect(dateOf(b.segments[1]!.start)).toBe("2027-05-05");
    expect(dateOf(b.segments[2]!.start)).toBe("2027-07-19");
    expect(dateOf(b.end)).toBe("2027-09-27");
  });

  it("has no pots for bought plants or sowing in the ground", () => {
    const plants = barOf(planting("a", "lettuce", { mode: "plants", sowing: null, planting_out: "2027-09-27", harvest_start: "2027-11-16", end: "2027-12-06", date: "2027-09-27" }));
    expect(plants.segments.map((s) => s.phase)).toEqual(["growth", "harvest"]);
    expect(dateOf(plants.start)).toBe("2027-09-27");
    const ground = barOf(planting("a", "carrot", { mode: "open_ground", sowing: "2027-03-15", planting_out: "2027-03-15", harvest_start: "2027-06-23", end: "2027-08-02" }));
    expect(ground.segments.map((s) => s.phase)).toEqual(["growth", "harvest"]);
  });
});

describe("stacking", () => {
  it("puts overlapping items on separate sub-rows and reuses a free one", () => {
    const lanes = stack([
      { id: "a", start: 0, end: 10 },
      { id: "b", start: 5, end: 15 },
      { id: "c", start: 10, end: 20 },
    ]);
    expect(lanes.get("a")).toBe(0);
    expect(lanes.get("b")).toBe(1);
    // c starts the day a ends: the half-open ranges do not overlap.
    expect(lanes.get("c")).toBe(0);
  });

  it("avoids the sub-rows that are already taken", () => {
    const lanes = stack([{ id: "x", start: 0, end: 10 }], [{ start: 5, end: 8, lane: 0 }]);
    expect(lanes.get("x")).toBe(1);
  });
});

describe("rows", () => {
  const slot: SlotView = {
    bed_id: "p1",
    segment_start: 1,
    segment_end: 6,
    start: "2027-09-27",
    end: "2027-11-15",
    suggestions: [],
  };

  it("makes one row per bed, with its bars and free slots", () => {
    const rows = buildRows(
      [view(2027, { plantings: [planting("a", "tomato"), planting("b", "lettuce", { segment_start: 4, segment_end: 6 })], slots: [slot] })],
      2027,
    );
    expect(rows).toHaveLength(1);
    const l = rows[0]!;
    expect(l.bars).toHaveLength(2);
    // The two bars overlap in time: two sub-rows. The slot starts after both.
    expect(l.lanes).toBe(2);
    expect(l.slots[0]!.lane).toBe(0);
    expect(dateOf(l.slots[0]!.end)).toBe("2027-11-16");
  });

  it("shows a planting that straddles two years once, and greys a missing bed", () => {
    const leek = planting("p", "leek", { planting_out: "2027-07-10", end: "2028-02-05", harvest_start: "2027-11-07" });
    const rows = buildRows([view(2027, { plantings: [leek] }), view(2028, { plantings: [leek], beds: [] })], 2027);
    expect(rows[0]!.bars).toHaveLength(1);
    expect(rows[0]!.absences).toEqual([{ start: dayOf("2028-01-01"), end: dayOf("2029-01-01") }]);
  });

  it("orders the rows by position on the plan", () => {
    const b: BedView = { ...BED, id: "p2", name: "B", x: 10, y: 50 };
    const c: BedView = { ...BED, id: "p3", name: "C", x: 0, y: 10 };
    const rows = buildRows([view(2027, { beds: [BED, b, c] })], 2027);
    expect(rows.map((l) => l.bed.id)).toEqual(["p3", "p2", "p1"]);
  });

  it("marks the plantings Rust reports in conflict", () => {
    const rows = buildRows(
      [view(2027, { plantings: [planting("a", "tomato")], conflicts: [{ a: "a", b: "z", segment_start: 1, segment_end: 1, start: "2027-06-01", end: "2027-06-02" }] })],
      2027,
    );
    expect(rows[0]!.bars[0]!.enConflict).toBe(true);
  });
});

describe("row under the pointer", () => {
  // Rows of 44 and 78 px under a 62 px header, with a 1 px line between them.
  const heights = [44, 78];
  it("finds the row a y falls in", () => {
    expect(rowAt(heights, 62, 62)).toBe(0);
    expect(rowAt(heights, 105, 62)).toBe(0);
    expect(rowAt(heights, 107, 62)).toBe(1);
    expect(rowAt(heights, 184, 62)).toBe(1);
  });
  it("is none in the header and below the last row", () => {
    expect(rowAt(heights, 10, 62)).toBe(-1);
    expect(rowAt(heights, 186, 62)).toBe(-1);
    expect(rowAt([], 70, 62)).toBe(-1);
  });
});
