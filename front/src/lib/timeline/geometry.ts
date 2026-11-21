// Layout of the timeline (S01 F-VIS-2, S05 §4): pixels to days and back,
// scrolling, zoom, the bars and their sub-rows. Pure functions, tested.
//
// Nothing here is a rule of S03. The dates of a bar (planting out, harvest,
// end), the conflicts, the free slots and the suggestions all come from Rust
// in the year view; this file only decides where to draw them.
import type { SlotView, BedView, PlantingView, YearView } from "../bindings";
import { dayOf, premierJanuary } from "../dates";

/** The visible part of the timeline: from `startDay` on, `pxByDay` pixels
 *  per day, `width` pixels wide. */
export type Scale = { startDay: number; pxByDay: number; width: number };

/** 12 months at first; from 3 months to 3 years by pinching or with the wheel (F-VIS-2). */
export const DAYS_BY_DEFAULT = 365;
/** On a phone width the timeline opens on 6 months: 12 would not be readable (decision of S05 v0.9). */
export const DAYS_BY_DEFAULT_NARROW = 180;
export const DAYS_MIN = 90;
export const DAYS_MAX = 1095;
/** Pointer moves under this distance, in pixels, are a click, not a scroll. */
export const THRESHOLD_DEFILEMENT_PX = 4;

export function daysVisible(e: Scale): number {
  return e.width / e.pxByDay;
}

/** The pixel of a day, which may be fractional. */
export function xOf(e: Scale, day: number): number {
  return (day - e.startDay) * e.pxByDay;
}

/** The day under a pixel, which may be fractional. */
export function dayOfX(e: Scale, x: number): number {
  return e.startDay + x / e.pxByDay;
}

/** The civil day under a pixel: the pointer to a date. `x` is relative to the
 *  left of the drawing area, not to the window. */
export function dayUnderLePointer(e: Scale, clientX: number, gauche: number): number {
  return Math.floor(dayOfX(e, clientX - gauche));
}

/** A scale showing `daysVisible` days with `centreDay` in the middle. */
export function scaleCentered(width: number, centreDay: number, countOfDays = DAYS_BY_DEFAULT): Scale {
  const pxByDay = width / countOfDays;
  return { startDay: centreDay - countOfDays / 2, pxByDay, width };
}

/** The finger or mouse moved by `dxPx` since the start of the scroll: the
 *  timeline follows it, so moving right shows earlier days. */
export function defiler(depart: Scale, dxPx: number): Scale {
  return { ...depart, startDay: depart.startDay - dxPx / depart.pxByDay };
}

/** Zoom by `factor` (> 1 widens the view onto more days), keeping the day
 *  under `anchoringX` where it is. Clamped from 3 months to 3 years. */
export function zoomer(e: Scale, factor: number, anchoringX: number): Scale {
  const count = Math.min(DAYS_MAX, Math.max(DAYS_MIN, daysVisible(e) * factor));
  const pxByDay = e.width / count;
  const anchor = dayOfX(e, anchoringX);
  return { width: e.width, pxByDay, startDay: anchor - anchoringX / pxByDay };
}

/** The scale with `day` in the middle, same zoom. */
export function centerOn(e: Scale, day: number): Scale {
  return { ...e, startDay: day - daysVisible(e) / 2 };
}

/** Scrolls as little as needed to bring `day` into view, with a margin of a
 *  tenth of the width. Unchanged if it is already visible. */
export function keepVisible(e: Scale, day: number): Scale {
  const marge = daysVisible(e) * 0.1;
  if (day < e.startDay + marge) return { ...e, startDay: day - marge };
  if (day > e.startDay + daysVisible(e) - marge) return { ...e, startDay: day - daysVisible(e) + marge };
  return e;
}

/** A bar dragged by `dxPx` moves by whole days (S03 §2: the shift is in days). */
export function shiftInDays(dxPx: number, pxByDay: number): number {
  return Math.round(dxPx / pxByDay);
}

/** The years the visible days touch: the views to load. */
export function yearsVisible(e: Scale): number[] {
  const start = new Date(Math.floor(e.startDay) * 86_400_000).getUTCFullYear();
  const end = new Date(Math.ceil(e.startDay + daysVisible(e)) * 86_400_000).getUTCFullYear();
  return Array.from({ length: end - start + 1 }, (_, i) => start + i);
}

/** The first day of each month that touches the visible days, for the columns. */
export type Month = { day: number; year: number; month: number; days: number };

export function monthVisible(e: Scale): Month[] {
  const premier = new Date(Math.floor(e.startDay) * 86_400_000);
  let year = premier.getUTCFullYear();
  let month = premier.getUTCMonth();
  const limit = e.startDay + daysVisible(e);
  const sortie: Month[] = [];
  for (;;) {
    const day = Math.round(Date.UTC(year, month, 1) / 86_400_000);
    if (day >= limit) break;
    const next = month === 11 ? [year + 1, 0] : [year, month + 1];
    const days = Math.round(Date.UTC(next[0]!, next[1]!, 1) / 86_400_000) - day;
    sortie.push({ day, year, month, days });
    [year, month] = next as [number, number];
  }
  return sortie;
}

// ---------- Bars, slots and rows ----------

export type Phase = "pots" | "growth" | "harvest";
export type Segment = { phase: Phase; start: number; end: number };

/** A planting as a bar: day numbers, `end` not included. */
export type Bar = {
  id: string;
  cropId: string;
  bedId: string;
  mode: PlantingView["mode"];
  start: number;
  end: number;
  segments: Segment[];
  zone: [number, number];
  manualEnd: boolean;
  enConflict: boolean;
};

/** The phases of S03 §2, read from the dates Rust sends: pots from sowing to
 *  planting out (mode indoor only), growth to the start of harvest, harvest. */
export function barOf(s: PlantingView, enConflict = false): Bar {
  const m = dayOf(s.planting_out);
  const r = dayOf(s.harvest_start);
  const f = dayOf(s.end);
  const sowing = s.mode === "indoor" && s.sowing ? dayOf(s.sowing) : m;
  const segments: Segment[] = [];
  if (sowing < m) segments.push({ phase: "pots", start: sowing, end: m });
  if (r > m) segments.push({ phase: "growth", start: m, end: r });
  segments.push({ phase: "harvest", start: r, end: f });
  return {
    id: s.id,
    cropId: s.crop_id,
    bedId: s.bed_id,
    mode: s.mode,
    start: Math.min(sowing, m),
    end: f,
    segments,
    zone: [s.segment_start, s.segment_end],
    manualEnd: s.manual_end,
    enConflict,
  };
}

export type Slot = {
  key: string;
  bedId: string;
  segments: [number, number];
  start: number;
  /** Day after the last free day. */
  end: number;
  source: SlotView;
};

export function slotOf(c: SlotView): Slot {
  return {
    key: `${c.bed_id}/${c.segment_start}-${c.segment_end}/${c.start}/${c.end}`,
    bedId: c.bed_id,
    segments: [c.segment_start, c.segment_end],
    start: dayOf(c.start),
    end: dayOf(c.end) + 1,
    source: c,
  };
}

/** Greedy sub-rows: items that overlap in time are stacked (F-VIS-2). Items
 *  already placed (`fixes`) keep their sub-row. Returns the sub-row of each id. */
export function stack(
  elements: readonly { id: string; start: number; end: number }[],
  fixes: readonly { start: number; end: number; lane: number }[] = [],
): Map<string, number> {
  const occupied: { start: number; end: number; lane: number }[] = [...fixes];
  const lanes = new Map<string, number>();
  for (const e of [...elements].sort((a, b) => a.start - b.start || a.end - b.end)) {
    let lane = 0;
    while (occupied.some((o) => o.lane === lane && o.start < e.end && e.start < o.end)) lane++;
    lanes.set(e.id, lane);
    occupied.push({ start: e.start, end: e.end, lane });
  }
  return lanes;
}

/** The index of the row under `y` (pixels from the top of the drawing), or
 *  -1 in the header or below the rows. The rows have the given `heights`,
 *  start at `top`, and are one pixel apart, as `TimelineSvg` draws them. */
export function rowAt(heights: readonly number[], y: number, top: number): number {
  let start = top;
  for (const [i, height] of heights.entries()) {
    if (y >= start && y < start + height + 1) return i;
    start += height + 1;
  }
  return -1;
}

export type Row = {
  bed: BedView;
  bars: (Bar & { lane: number })[];
  slots: (Slot & { lane: number })[];
  /** Day ranges where the bed does not exist yet or any more (greyed). */
  absences: { start: number; end: number }[];
  lanes: number;
};

/**
 * One row per bed, from the year views loaded around the visible days.
 * The rows follow the position on the plan (top to bottom, then left to
 * right); a bed that does not exist in a loaded year is greyed for it.
 * `reference` is the year whose layout names the row.
 */
export function buildRows(views: readonly YearView[], reference: number): Row[] {
  const sorted = [...views].sort((a, b) => a.year - b.year);
  const beds = new Map<string, BedView>();
  for (const view of sorted) {
    for (const p of view.beds) {
      if (view.year <= reference || !beds.has(p.id)) beds.set(p.id, p);
    }
  }
  const plantings = new Map<string, PlantingView>();
  const conflicts = new Set<string>();
  const slots = new Map<string, Slot>();
  for (const view of sorted) {
    for (const s of view.plantings) plantings.set(s.id, s);
    for (const c of view.conflicts) {
      conflicts.add(c.a);
      conflicts.add(c.b);
    }
    for (const c of view.slots) {
      const slot = slotOf(c);
      slots.set(slot.key, slot);
    }
  }

  const rows: Row[] = [];
  for (const bed of beds.values()) {
    const bars = [...plantings.values()]
      .filter((s) => s.bed_id === bed.id)
      .map((s) => barOf(s, conflicts.has(s.id)));
    const lanesBars = stack(bars);
    const posees = bars.map((b) => ({ start: b.start, end: b.end, lane: lanesBars.get(b.id) ?? 0 }));
    const free = [...slots.values()].filter((c) => c.bedId === bed.id);
    const lanesSlots = stack(
      free.map((c) => ({ id: c.key, start: c.start, end: c.end })),
      posees,
    );
    const absences = sorted
      .filter((view) => !view.beds.some((p) => p.id === bed.id))
      .map((view) => ({ start: premierJanuary(view.year), end: premierJanuary(view.year + 1) }));
    const all = [...lanesBars.values(), ...lanesSlots.values()];
    rows.push({
      bed,
      bars: bars.map((b) => ({ ...b, lane: lanesBars.get(b.id) ?? 0 })),
      slots: free.map((c) => ({ ...c, lane: lanesSlots.get(c.key) ?? 0 })),
      absences,
      lanes: all.length ? Math.max(...all) + 1 : 1,
    });
  }
  return rows.sort((a, b) => a.bed.y - b.bed.y || a.bed.x - b.bed.x);
}
